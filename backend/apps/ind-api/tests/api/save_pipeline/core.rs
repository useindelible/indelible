use reqwest::StatusCode;

use super::common::{
    SaveScenario, assert_json_response, document_available_assets, document_id_from_response,
};

#[tokio::test]
async fn extension_quick_save_prepares_readable_document() {
    let scenario = SaveScenario::new().await;

    let created = scenario
        .extension_quick_save("https://example.com/save-pipeline/quick-save")
        .await;
    let document_id = document_id_from_response(&created);
    assert!(
        created["reader_url"]
            .as_str()
            .expect("reader_url")
            .ends_with(&format!("/reader/{document_id}")),
        "extension saves must open the document reader, got {}",
        created["reader_url"]
    );

    let doc = scenario.get_document(&document_id).await;
    assert_eq!(doc["saved"], true);
    assert!(
        scenario
            .pending_job_count_by_type("feed.prepare_document")
            .await
            >= 1,
        "quick save must enqueue document readable preparation"
    );
}

#[tokio::test]
async fn extension_check_url_and_context_resolve_document_only_save() {
    let scenario = SaveScenario::new().await;
    let url = "https://example.com/save-pipeline/extension-check";

    let saved = scenario.extension_reader_save(url).await;
    let library_entry_id = saved["library_entry_id"]
        .as_str()
        .expect("canonical library_entry_id");
    assert!(
        library_entry_id.starts_with("lib_"),
        "extension save returns a canonical library entry id, got {library_entry_id}"
    );
    let document_id = document_id_from_response(&saved);
    let reader_url = saved["reader_url"].as_str().expect("reader_url");

    let check = scenario
        .extension_client()
        .get(&format!("/api/v1/extension/check-url?url={url}"))
        .await;
    let check_body = assert_json_response(check, StatusCode::OK).await;
    assert_eq!(check_body["exists"], true);
    assert_eq!(check_body["library_entry_id"], library_entry_id);
    assert_eq!(check_body["document_id"], document_id);
    assert_eq!(check_body["reader_url"], reader_url);

    let context = scenario
        .extension_client()
        .get(&format!("/api/v1/extension/entries/{library_entry_id}"))
        .await;
    let context_body = assert_json_response(context, StatusCode::OK).await;
    assert_eq!(context_body["library_entry_id"], library_entry_id);
    assert_eq!(context_body["document_id"], document_id);
    assert_eq!(context_body["title"], "Reader Save Article");
    assert_eq!(context_body["reader_url"], reader_url);

    let patched = scenario
        .extension_client()
        .patch_json(
            &format!("/api/v1/extension/entries/{library_entry_id}"),
            &serde_json::json!({
                "triage_state": "later",
                "is_favorite": true
            }),
        )
        .await;
    let patched_body = assert_json_response(patched, StatusCode::OK).await;
    assert_eq!(patched_body["library_entry_id"], library_entry_id);
    assert_eq!(patched_body["document_id"], document_id);
    assert_eq!(patched_body["triage_state"], "later");
    assert_eq!(patched_body["is_favorite"], true);

    let asset = scenario
        .extension_client()
        .get(&format!(
            "/api/v1/extension/entries/{library_entry_id}/assets/readable_html"
        ))
        .await;
    let asset = assert_json_response(asset, StatusCode::OK).await;
    assert_eq!(asset["document_id"], document_id);
    let download = scenario
        .app
        .client()
        .get(asset["download_url"].as_str().expect("download URL"))
        .send()
        .await
        .expect("download readable HTML");
    assert_eq!(download.status(), StatusCode::OK);
    assert!(download.text().await.unwrap().contains("Integration Reader Article"));
}

#[tokio::test]
async fn extension_full_archive_stores_readable_and_monolith_document_assets() {
    let scenario = SaveScenario::new().await;

    let created = scenario
        .extension_full_archive("https://example.com/save-pipeline/full-archive")
        .await;
    let document_id = document_id_from_response(&created);

    assert_eq!(
        scenario
            .pending_job_count_by_type("document.attach_provided_content")
            .await,
        2,
        "full archive commits monolith + readable attach drivers in the save tx"
    );
    assert_eq!(
        scenario
            .pending_job_count_by_type("feed.prepare_document")
            .await,
        1,
        "default screenshot archival enqueues server preparation for derived assets"
    );

    let doc = scenario.get_document(&document_id).await;
    assert_eq!(doc["saved"], true);
    assert_eq!(doc["readable_ready"], true);
    let assets = document_available_assets(&doc);
    assert!(assets.contains(&"readable_html".to_string()));
    assert!(
        assets.contains(&"monolith".to_string()),
        "full archive must attach the monolith capture, got {assets:?}"
    );
    scenario
        .assert_document_asset_downloadable(&document_id, "monolith")
        .await;
}

#[tokio::test]
async fn extension_check_url_annotations_share_the_saved_document_contract() {
    let scenario = SaveScenario::new().await;
    let saved = scenario
        .extension_reader_save("https://example.com/save-pipeline/extension-annotations")
        .await;
    let entry_id = saved["library_entry_id"].as_str().unwrap();
    let client = scenario.extension_client();
    let highlights_path = format!("/api/v1/extension/entries/{entry_id}/highlights");

    assert_eq!(
        client
            .post_json(
                &highlights_path,
                &serde_json::json!({"color": "yellow", "text_content": "missing locator"}),
            )
            .await
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let highlight = assert_json_response(
        client
            .post_json(
                &highlights_path,
                &serde_json::json!({
                    "color": "yellow",
                    "text_content": "Integration Reader",
                    "locator": {"type": "html", "start_offset": 0, "end_offset": 18}
                }),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let listed = assert_json_response(client.get(&highlights_path).await, StatusCode::OK).await;
    assert_eq!(listed["count"], 1);
    assert_eq!(listed["highlights"][0]["id"], highlight["id"]);

    let note_path = format!("/api/v1/extension/entries/{entry_id}/note");
    let note = assert_json_response(
        client
            .put_json(
                &note_path,
                &serde_json::json!({"body": "Extension-owned document note"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(note["body"], "Extension-owned document note");
    let tags = assert_json_response(
        client
            .put_json(
                &format!("/api/v1/extension/entries/{entry_id}/tags"),
                &serde_json::json!({"tags": [" Rust ", "Architecture", "rust"]}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(tags["tags"], serde_json::json!(["architecture", "rust"]));
    let context = assert_json_response(
        client
            .get(&format!("/api/v1/extension/entries/{entry_id}"))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(context["note"]["body"], note["body"]);
    assert_eq!(context["tags"].as_array().unwrap().len(), 2);

    let cleared = assert_json_response(
        client
            .put_json(&note_path, &serde_json::json!({"body": ""}))
            .await,
        StatusCode::OK,
    )
    .await;
    assert!(cleared.is_null());
}
