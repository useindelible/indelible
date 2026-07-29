#[tokio::test]
async fn same_readwise_origin_retry_repairs_assets_tags_progress_and_jobs() {
    let s = ReadwiseScenario::new().await;
    let id = "repair01";
    let data = csv(&[(
        "Repairable Article",
        "https://example.com/repairable",
        id,
        "['repair']",
        0.5,
        "later",
        "False",
    )]);
    let zip = archive(&[(
        "Library/Repairable Article (repair01).html",
        b"<article>Repair me</article>",
    )]);
    s.import(Some(&data), Some(&zip), None).await;
    let document_id = s.document_for_origin(id).await;
    let entry_id: uuid::Uuid = sqlx::query_scalar(
        "SELECT id FROM library_entries WHERE user_id = $1 AND document_id = $2",
    )
    .bind(s.user_id.into_uuid())
    .bind(document_id.into_uuid())
    .fetch_one(s.db.pool())
    .await
    .unwrap();
    for statement in [
        "DELETE FROM archive_assets WHERE document_id = $1",
        "DELETE FROM user_document_state WHERE document_id = $1",
    ] {
        sqlx::query(statement)
            .bind(document_id.into_uuid())
            .execute(s.db.pool())
            .await
            .unwrap();
    }
    sqlx::query("DELETE FROM library_entry_tags WHERE library_entry_id = $1")
        .bind(entry_id)
        .execute(s.db.pool())
        .await
        .unwrap();

    let retry = s.import(Some(&data), Some(&zip), None).await;
    assert_eq!(job_counts(&s, retry).await, (0, 1, 0));
    let repaired: (i64, i64, i64) = sqlx::query_as(
        "SELECT \
          (SELECT count(*) FROM archive_assets WHERE document_id = $1 AND asset_kind = 'readable_html'), \
          (SELECT count(*) FROM library_entry_tags WHERE library_entry_id = $2), \
          (SELECT count(*) FROM user_document_state WHERE document_id = $1 AND progress_percent = 50)",
    )
    .bind(document_id.into_uuid())
    .bind(entry_id)
    .fetch_one(s.db.pool())
    .await
    .unwrap();
    assert_eq!(repaired, (1, 1, 1));
    let report: serde_json::Value =
        sqlx::query_scalar("SELECT provider_report FROM import_jobs WHERE id = $1")
            .bind(retry.into_uuid())
            .fetch_one(s.db.pool())
            .await
            .unwrap();
    assert_eq!(report["search_reindex_jobs_enqueued"], 1);
    assert_eq!(report["embedding_jobs_enqueued"], 1);
}
#[tokio::test]
async fn cross_source_duplicate_claims_archive_without_phantom_item() {
    let s = ReadwiseScenario::new().await;
    let existing = SavedDocumentFactory::new(s.user_id)
        .with_document_type(DocumentType::Video)
        .with_url("https://youtube.com/watch?v=weeI1G46q0o")
        .insert(s.db.pool())
        .await;
    let data = csv(&[(
        "Duplicate video",
        "https://www.youtube.com/watch?v=weeI1G46q0o&list=tracking",
        "phantom01",
        "[]",
        0.0,
        "new",
        "True",
    )]);
    let zip = archive(&[(
        "Library/Duplicate video (phantom01).html",
        b"<html>snapshot</html>",
    )]);
    let job = s.import(Some(&data), Some(&zip), None).await;
    assert_eq!(job_counts(&s, job).await, (0, 1, 0));
    let report: serde_json::Value =
        sqlx::query_scalar("SELECT provider_report FROM import_jobs WHERE id = $1")
            .bind(job.into_uuid())
            .fetch_one(s.db.pool())
            .await
            .unwrap();
    assert_eq!(report["zip_files_unmatched"], 0);
    let ids: Vec<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM documents WHERE user_id = $1")
        .bind(s.user_id.into_uuid())
        .fetch_all(s.db.pool())
        .await
        .unwrap();
    assert_eq!(ids, vec![existing.document_id.into_uuid()]);
}

#[tokio::test]
async fn epub_pdf_and_malformed_archive_asset_semantics_are_preserved() {
    let s = ReadwiseScenario::new().await;
    let data = csv(&[
        (
            "Book",
            "private://read/book01",
            "book01",
            "[]",
            0.0,
            "new",
            "False",
        ),
        (
            "Paper",
            "https://example.com/paper.pdf",
            "pdf01",
            "[]",
            0.0,
            "archive",
            "False",
        ),
        (
            "Broken",
            "https://example.com/broken.epub",
            "bad01",
            "[]",
            0.0,
            "new",
            "False",
        ),
    ]);
    let zip = archive(&[
        ("Library/Book (book01).epub", b"epub bytes"),
        ("Library/Paper (pdf01).pdf", b"%PDF-1.4 bytes"),
        ("Library/Broken (bad01).epub", b"not an epub"),
    ]);
    let job = s.import(Some(&data), Some(&zip), None).await;
    assert_eq!(job_counts(&s, job).await, (3, 0, 0));
    for (id, expected) in [
        ("book01", vec![("original_upload", "application/epub+zip")]),
        (
            "pdf01",
            vec![
                ("original_upload", "application/pdf"),
                ("pdf", "application/pdf"),
            ],
        ),
        ("bad01", vec![("original_upload", "application/epub+zip")]),
    ] {
        let document_id = s.document_for_origin(id).await;
        let assets: Vec<(String, String)> = sqlx::query_as(
            "SELECT asset_kind, content_type FROM archive_assets WHERE document_id = $1 ORDER BY asset_kind",
        )
        .bind(document_id.into_uuid())
        .fetch_all(s.db.pool())
        .await
        .unwrap();
        assert_eq!(
            assets,
            expected
                .into_iter()
                .map(|(a, b)| (a.into(), b.into()))
                .collect::<Vec<_>>()
        );
    }
    let embeds: i64 =
        sqlx::query_scalar("SELECT count(*) FROM job_outbox WHERE job_type = 'document.ai.embed'")
            .fetch_one(s.db.pool())
            .await
            .unwrap();
    assert_eq!(embeds, 0, "non-readable uploads do not enqueue embedding");
}

#[tokio::test]
async fn opml_and_youtube_inputs_route_to_their_authoritative_pipelines() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let s = ReadwiseScenario::new().await;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/feed.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<feed xmlns="http://www.w3.org/2005/Atom"><title>Feed</title><id>x</id><updated>2024-01-15T00:00:00Z</updated></feed>"#,
        ))
        .mount(&server)
        .await;
    let opml = format!(
        r#"<opml version="2.0"><body><outline type="rss" xmlUrl="{}/feed.xml" text="Feed"/></body></opml>"#,
        server.uri()
    );
    let opml_job = s.import(None, None, Some(opml.as_bytes())).await;
    let report: serde_json::Value =
        sqlx::query_scalar("SELECT provider_report FROM import_jobs WHERE id = $1")
            .bind(opml_job.into_uuid())
            .fetch_one(s.db.pool())
            .await
            .unwrap();
    assert_eq!(report["opml_feeds_created"], 1);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM documents WHERE user_id = $1")
        .bind(s.user_id.into_uuid())
        .fetch_one(s.db.pool())
        .await
        .unwrap();
    assert_eq!(count, 0);

    let data = csv(&[(
        "Video",
        "https://www.youtube.com/watch?v=lrdmnAn9gxk",
        "video01",
        "[]",
        0.0,
        "new",
        "True",
    )]);
    let zip = archive(&[("Library/Video (video01).html", b"watch snapshot")]);
    let video_job = s.import(Some(&data), Some(&zip), None).await;
    let document_id = s.document_for_origin("video01").await;
    let (kind, youtube_jobs, readable_assets): (String, i64, i64) = sqlx::query_as(
        "SELECT d.document_type, \
          (SELECT count(*) FROM job_outbox WHERE job_type = 'document.youtube_ingest' AND payload->>'document_id' = $2), \
          (SELECT count(*) FROM archive_assets WHERE document_id = d.id AND asset_kind = 'readable_html') \
         FROM documents d WHERE d.id = $1",
    )
    .bind(document_id.into_uuid())
    .bind(document_id.to_string())
    .fetch_one(s.db.pool())
    .await
    .unwrap();
    assert_eq!(
        (kind.as_str(), youtube_jobs, readable_assets),
        ("video", 1, 0)
    );
    let video_report: serde_json::Value =
        sqlx::query_scalar("SELECT provider_report FROM import_jobs WHERE id = $1")
            .bind(video_job.into_uuid())
            .fetch_one(s.db.pool())
            .await
            .unwrap();
    assert_eq!(video_report["zip_files_unmatched"], 0);
}
