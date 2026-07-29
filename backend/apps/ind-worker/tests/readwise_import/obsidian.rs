async fn seed_readable(s: &ReadwiseScenario, document_id: DocumentId, html: &str) {
    let key = format!("documents/{}/{document_id}/readable.html", s.user_id);
    s.ctx
        .object_storage
        .as_ref()
        .unwrap()
        .upload(
            &key,
            "text/html",
            bytes::Bytes::copy_from_slice(html.as_bytes()),
        )
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO archive_assets \
         (id, document_id, asset_kind, s3_key, s3_bucket, content_type, size_bytes, created_at, status) \
         VALUES ($1, $2, 'readable_html', $3, $4, 'text/html', $5, now(), 'completed')",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(document_id.into_uuid())
    .bind(key)
    .bind(s.db.bucket())
    .bind(html.len() as i64)
    .execute(s.db.pool())
    .await
    .unwrap();
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
