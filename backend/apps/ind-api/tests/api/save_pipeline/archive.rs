/// A provided-content reader save must commit the attach-provided-content driver inside the save
/// transaction (not a fire-and-forget post-commit write), and running that one job must attach the
/// readable asset and enqueue search reindex + the content-gated embed.
#[tokio::test]
async fn reader_save_commits_attach_driver_and_drives_index_and_embed() {
    let scenario = SaveScenario::new().await;
    let url = "https://example.com/save-pipeline/durable-attach";
    let created = scenario
        .extension_reader_save(url)
        .await;
    let document_id = document_id_from_response(&created);
    scenario.extension_reader_save(url).await;

    assert_eq!(
        scenario.total_job_count_by_type("document.attach_provided_content").await,
        1,
        "duplicate save must converge on one durable attach driver"
    );
    assert_eq!(
        scenario
            .pending_job_count_by_type("search.reindex_document")
            .await,
        0
    );
    let doc_before_worker = scenario.get_document(&document_id).await;
    assert_eq!(
        doc_before_worker["readable_ready"], true,
        "provided readable HTML must be attached before the save response is useful to the extension"
    );

    let processed = scenario
        .run_pending_jobs_of_type("document.attach_provided_content")
        .await;
    assert_eq!(processed, 1);

    let doc = scenario.get_document(&document_id).await;
    assert_eq!(doc["readable_ready"], true);
    assert!(document_available_assets(&doc).contains(&"readable_html".to_string()));
    assert_eq!(
        scenario
            .pending_job_count_by_type("search.reindex_document")
            .await,
        1,
        "the readable attach drives a search reindex"
    );
    assert_eq!(
        scenario
            .pending_job_count_by_type("document.ai.embed")
            .await,
        1,
        "the readable attach drives the content-gated embed for the engaged document"
    );
    let assets: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM archive_assets \
         WHERE document_id = $1 AND asset_kind = 'readable_html' AND status = 'completed'",
    )
    .bind(uuid_from_doc(&document_id))
    .fetch_one(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(assets, 1);
}

fn uuid_from_doc(document_id: &str) -> uuid::Uuid {
    uuid::Uuid::parse_str(document_id.trim_start_matches("doc_")).expect("doc uuid")
}
