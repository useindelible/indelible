use ind_application::repos::prepared_content::PreparedContentProvider;
use ind_ingest::AssetBackedPreparedContentProvider;

async fn seed_readable(s: &ReadwiseScenario, document_id: DocumentId, html: &str) {
    seed_asset(
        s,
        document_id,
        "readable_html",
        "readable.html",
        "text/html",
        html.as_bytes(),
    )
    .await;
}

async fn seed_asset(
    s: &ReadwiseScenario,
    document_id: DocumentId,
    asset_kind: &str,
    filename: &str,
    content_type: &str,
    bytes: &[u8],
) -> String {
    let key = format!("documents/{}/{document_id}/{filename}", s.user_id);
    s.ctx
        .object_storage
        .as_ref()
        .unwrap()
        .upload(&key, content_type, bytes::Bytes::copy_from_slice(bytes))
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO archive_assets \
         (id, document_id, asset_kind, s3_key, s3_bucket, content_type, size_bytes, created_at, status) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, now(), 'completed')",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(document_id.into_uuid())
    .bind(asset_kind)
    .bind(&key)
    .bind(s.db.bucket())
    .bind(content_type)
    .bind(bytes.len() as i64)
    .execute(s.db.pool())
    .await
    .unwrap();
    key
}

fn prepared_content_provider(s: &ReadwiseScenario) -> AssetBackedPreparedContentProvider {
    AssetBackedPreparedContentProvider::new(
        s.ctx.document_repo.clone(),
        s.ctx.document_asset_repo.clone(),
        s.ctx.mila_config_repo.clone(),
        s.ctx.object_storage.clone(),
    )
}

fn graphics_only_pdf() -> Vec<u8> {
    let content = "q 0 0 100 100 re f Q";
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (index, object) in [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /Contents 4 0 R >>".to_string(),
        format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        ),
    ]
    .into_iter()
    .enumerate()
    {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
    }
    let xref = pdf.len();
    pdf.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n");
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!("trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
    );
    pdf
}

async fn seed_epub_text(s: &ReadwiseScenario, document_id: DocumentId, text: &str) {
    let toc = br#"{"metadata":{"title":"Test EPUB","author":null,"publisher":null,"language":null,"isbn":null,"total_chapters":0,"total_words":0,"estimated_pages":0},"toc":[]}"#;
    seed_asset(
        s,
        document_id,
        "epub",
        "epub_toc.json",
        "application/json",
        toc,
    )
    .await;
    seed_asset(
        s,
        document_id,
        "extracted_text",
        "extracted.txt",
        "text/plain",
        text.as_bytes(),
    )
    .await;
}

async fn seed_pdf(s: &ReadwiseScenario, document_id: DocumentId, text: Option<&str>) {
    let pdf = graphics_only_pdf();
    let pdf_key = seed_asset(s, document_id, "pdf", "reader.pdf", "application/pdf", &pdf).await;
    sqlx::query(
        "INSERT INTO archive_assets \
         (id, document_id, asset_kind, s3_key, s3_bucket, content_type, size_bytes, created_at, status) \
         VALUES ($1, $2, 'original_upload', $3, $4, 'application/pdf', $5, now(), 'completed')",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(document_id.into_uuid())
    .bind(pdf_key)
    .bind(s.db.bucket())
    .bind(pdf.len() as i64)
    .execute(s.db.pool())
    .await
    .unwrap();
    if let Some(text) = text {
        seed_asset(
            s,
            document_id,
            "extracted_text",
            "extracted.txt",
            "text/plain",
            text.as_bytes(),
        )
        .await;
    }
}

#[tokio::test]
async fn prepared_text_provider_returns_supported_root_text_and_rejects_unreadable_assets() {
    let s = ReadwiseScenario::new().await;
    let article = SavedDocumentFactory::new(s.user_id)
        .with_title("Article")
        .insert(s.db.pool())
        .await;
    seed_readable(
        &s,
        article.document_id,
        "<article>Canonical article text</article>",
    )
    .await;
    let epub = SavedDocumentFactory::new(s.user_id)
        .with_title("EPUB")
        .with_document_type(DocumentType::Book)
        .insert(s.db.pool())
        .await;
    seed_epub_text(&s, epub.document_id, "Canonical EPUB text").await;
    let pdf = SavedDocumentFactory::new(s.user_id)
        .with_title("PDF")
        .with_document_type(DocumentType::Pdf)
        .insert(s.db.pool())
        .await;
    seed_pdf(&s, pdf.document_id, Some("Canonical PDF text")).await;
    let empty = SavedDocumentFactory::new(s.user_id)
        .with_title("Empty")
        .insert(s.db.pool())
        .await;
    seed_readable(
        &s,
        empty.document_id,
        "<article><img src=\"cover.png\"></article>",
    )
    .await;
    let failed = SavedDocumentFactory::new(s.user_id)
        .with_title("Failed")
        .insert(s.db.pool())
        .await;
    seed_asset(
        &s,
        failed.document_id,
        "readable_html",
        "readable.html",
        "text/html",
        b"<article>must not escape failed state</article>",
    )
    .await;
    sqlx::query(
        "UPDATE archive_assets SET status = 'failed' \
         WHERE document_id = $1 AND asset_kind = 'readable_html'",
    )
    .bind(failed.document_id.into_uuid())
    .execute(s.db.pool())
    .await
    .unwrap();
    let graphics_only = SavedDocumentFactory::new(s.user_id)
        .with_title("Graphics-only PDF")
        .with_document_type(DocumentType::Pdf)
        .insert(s.db.pool())
        .await;
    seed_pdf(&s, graphics_only.document_id, None).await;

    let provider = prepared_content_provider(&s);
    for (document_id, expected) in [
        (article.document_id, Some("Canonical article text")),
        (epub.document_id, Some("Canonical EPUB text")),
        (pdf.document_id, Some("Canonical PDF text")),
        (empty.document_id, None),
        (failed.document_id, None),
        (graphics_only.document_id, None),
    ] {
        assert_eq!(
            provider
                .load_readable_text_for_document(document_id)
                .await
                .unwrap()
                .as_deref(),
            expected,
            "unexpected prepared text for {document_id}"
        );
    }
}

#[tokio::test]
async fn obsidian_export_emits_pdf_and_epub_companions_without_counting_unreadable_pdf_as_exported()
{
    let s = ReadwiseScenario::new().await;
    let article = SavedDocumentFactory::new(s.user_id)
        .with_title("Ready article")
        .insert(s.db.pool())
        .await;
    seed_readable(
        &s,
        article.document_id,
        "<article>Obsidian article companion</article>",
    )
    .await;
    let epub = SavedDocumentFactory::new(s.user_id)
        .with_title("Ready EPUB")
        .with_document_type(DocumentType::Book)
        .insert(s.db.pool())
        .await;
    seed_epub_text(&s, epub.document_id, "Obsidian EPUB companion").await;
    let pdf = SavedDocumentFactory::new(s.user_id)
        .with_title("Ready PDF")
        .with_document_type(DocumentType::Pdf)
        .insert(s.db.pool())
        .await;
    seed_pdf(&s, pdf.document_id, Some("Obsidian PDF companion")).await;
    let unreadable_pdf = SavedDocumentFactory::new(s.user_id)
        .with_title("Graphics-only PDF")
        .with_document_type(DocumentType::Pdf)
        .insert(s.db.pool())
        .await;
    seed_pdf(&s, unreadable_pdf.document_id, None).await;

    let connection_id = IntegrationConnectionId::new();
    sqlx::query(
        "INSERT INTO integration_connections \
         (id, user_id, provider, config, status, created_at, updated_at) \
         VALUES ($1, $2, 'obsidian', $3, 'active', now(), now())",
    )
    .bind(connection_id.into_uuid())
    .bind(s.user_id.into_uuid())
    .bind(serde_json::json!({
        "obsidian_export": { "export_all_reader_documents": true }
    }))
    .execute(s.db.pool())
    .await
    .unwrap();
    let run_id = uuid::Uuid::now_v7();
    let forced: Vec<uuid::Uuid> = Vec::new();
    sqlx::query(
        "INSERT INTO obsidian_export_runs \
         (id, connection_id, user_id, status, requested_by_user, auto, parent_folder_deleted, force_item_ids, created_at, updated_at) \
         VALUES ($1, $2, $3, 'pending', true, false, false, $4, now(), now())",
    )
    .bind(run_id)
    .bind(connection_id.into_uuid())
    .bind(s.user_id.into_uuid())
    .bind(&forced)
    .execute(s.db.pool())
    .await
    .unwrap();
    integrations::dispatch_envelope(
        &s.ctx,
        GenericJobEnvelope {
            outbox_id: JobOutboxId::new(),
            job_type: "integration.obsidian.sync_connection".into(),
            payload: serde_json::json!({
                "connection_id": connection_id.to_string(),
                "user_id": s.user_id.to_string(),
                "requested_by_user": true,
                "run_id": run_id,
            }),
            dedupe_key: None,
        },
    )
    .await
    .unwrap();

    let (status, total_documents, documents_exported): (String, i32, i32) = sqlx::query_as(
        "SELECT status, total_documents, documents_exported \
         FROM obsidian_export_runs WHERE id = $1",
    )
    .bind(run_id)
    .fetch_one(s.db.pool())
    .await
    .unwrap();
    assert_eq!(
        (status.as_str(), total_documents, documents_exported),
        ("artifact_ready", 4, 0),
        "all four candidates stay visible until the client reports delivery"
    );

    let bytes: Vec<u8> = sqlx::query_scalar(
        "SELECT bytes FROM obsidian_export_artifacts WHERE run_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(run_id)
    .fetch_one(s.db.pool())
    .await
    .unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let manifest: serde_json::Value =
        serde_json::from_reader(zip.by_name("artifact.json").unwrap()).unwrap();
    let entries = manifest["entries"].as_array().unwrap();
    assert_eq!(
        entries.len(),
        3,
        "article, EPUB, and PDF should emit; graphics-only PDF should not"
    );

    let companion = |id: LibraryEntryId| {
        entries
            .iter()
            .find(|entry| entry["subject_id"] == id.to_string())
            .and_then(|entry| entry["full_document_text"].as_str())
    };
    assert!(
        companion(article.library_entry_id)
            .unwrap()
            .contains("Obsidian article companion")
    );
    assert!(
        companion(epub.library_entry_id)
            .unwrap()
            .contains("Obsidian EPUB companion")
    );
    assert!(
        companion(pdf.library_entry_id)
            .unwrap()
            .contains("Obsidian PDF companion")
    );
    assert!(companion(unreadable_pdf.library_entry_id).is_none());
}

#[tokio::test]
async fn obsidian_worker_smoke_preserves_summary_precedence_and_disambiguates_paths() {
    let s = ReadwiseScenario::new().await;
    let mut subjects = Vec::new();
    for (index, excerpt) in [
        (0, Some("fallback")),
        (1, Some("excerpt summary")),
        (2, None),
    ] {
        let mut factory = SavedDocumentFactory::new(s.user_id)
            .with_title("Same title")
            .with_url(format!("https://source-{index}.example/item"));
        if let Some(excerpt) = excerpt {
            factory = factory.with_excerpt(excerpt);
        }
        let saved = factory.insert(s.db.pool()).await;
        seed_readable(&s, saved.document_id, &format!("<p>body {index}</p>")).await;
        subjects.push(saved);
    }
    sqlx::query(
        "INSERT INTO ai_outputs (id, document_id, output_type, content, created_at) \
         VALUES ($1, $2, 'summary', $3, now())",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(subjects[0].document_id.into_uuid())
    .bind(serde_json::Value::String("stored summary".into()))
    .execute(s.db.pool())
    .await
    .unwrap();
    let connection_id = IntegrationConnectionId::new();
    sqlx::query(
        "INSERT INTO integration_connections (id, user_id, provider, config, status, created_at, updated_at) \
         VALUES ($1, $2, 'obsidian', '{}', 'active', now(), now())",
    )
    .bind(connection_id.into_uuid())
    .bind(s.user_id.into_uuid())
    .execute(s.db.pool())
    .await
    .unwrap();
    let run_id = uuid::Uuid::now_v7();
    let forced = subjects
        .iter()
        .map(|item| item.library_entry_id.into_uuid())
        .collect::<Vec<_>>();
    sqlx::query(
        "INSERT INTO obsidian_export_runs \
         (id, connection_id, user_id, status, requested_by_user, auto, parent_folder_deleted, force_item_ids, created_at, updated_at) \
         VALUES ($1, $2, $3, 'pending', true, false, false, $4, now(), now())",
    )
    .bind(run_id)
    .bind(connection_id.into_uuid())
    .bind(s.user_id.into_uuid())
    .bind(&forced)
    .execute(s.db.pool())
    .await
    .unwrap();
    integrations::dispatch_envelope(
        &s.ctx,
        GenericJobEnvelope {
            outbox_id: JobOutboxId::new(),
            job_type: "integration.obsidian.sync_connection".into(),
            payload: serde_json::json!({
                "connection_id": connection_id.to_string(),
                "user_id": s.user_id.to_string(),
                "requested_by_user": true,
                "run_id": run_id,
            }),
            dedupe_key: None,
        },
    )
    .await
    .unwrap();
    let bytes: Vec<u8> = sqlx::query_scalar(
        "SELECT bytes FROM obsidian_export_artifacts WHERE run_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(run_id)
    .fetch_one(s.db.pool())
    .await
    .unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let manifest: serde_json::Value =
        serde_json::from_reader(zip.by_name("artifact.json").unwrap()).unwrap();
    let entries = manifest["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    let paths = entries
        .iter()
        .map(|entry| entry["file_path"].as_str().unwrap())
        .collect::<HashSet<_>>();
    assert_eq!(paths.len(), 3);
    let content = |id: LibraryEntryId| {
        entries
            .iter()
            .find(|entry| entry["subject_id"] == id.to_string())
            .unwrap()["full_content"]
            .as_str()
            .unwrap()
    };
    assert!(content(subjects[0].library_entry_id).contains("stored summary"));
    assert!(!content(subjects[0].library_entry_id).contains("fallback"));
    assert!(content(subjects[1].library_entry_id).contains("excerpt summary"));
    assert!(!content(subjects[2].library_entry_id).contains("- Summary:"));
}
