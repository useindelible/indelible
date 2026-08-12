use reqwest::StatusCode;
use serde_json::json;

use ind_test_support::spawn_app;

use super::common::{assert_json_response, dispatch_pending_jobs};

#[tokio::test]
async fn youtube_watch_and_short_link_share_one_library_identity() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let first = assert_json_response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": "https://www.youtube.com/watch?v=abc123&list=playlist&t=45"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let alias = assert_json_response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": "https://youtu.be/abc123?si=shared&t=90"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;

    assert_eq!(alias["document_id"], first["document_id"]);
    assert_eq!(alias["library_entry_id"], first["library_entry_id"]);
    assert_eq!(alias["document_type"], "video");

    let identities: Vec<(String, String)> =
        sqlx::query_as("SELECT canonical_url, document_type FROM documents WHERE user_id = $1")
            .bind(session.user.id.into_uuid())
            .fetch_all(app.pool())
            .await
            .unwrap();
    assert_eq!(
        identities,
        vec![(
            "https://youtube.com/watch?v=abc123".to_string(),
            "video".to_string()
        )]
    );

    let extension = app.create_extension_session(&session.user).await;
    let check = assert_json_response(
        app.authed_client(&extension)
            .get("/api/v1/extension/check-url?url=https%3A%2F%2Fyoutu.be%2Fabc123%3Fsi%3Dshared")
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(check["exists"], true);
    assert_eq!(check["document_id"], first["document_id"]);
    assert_eq!(check["library_entry_id"], first["library_entry_id"]);
}

#[tokio::test]
async fn youtube_shorts_save_as_the_canonical_video_identity() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let short = assert_json_response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": "https://www.youtube.com/shorts/short123/?feature=share"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let watch = assert_json_response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": "https://youtube.com/watch?v=short123"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;

    assert_eq!(short["document_type"], "video");
    assert_eq!(
        short["canonical_url"],
        "https://youtube.com/watch?v=short123"
    );
    assert_eq!(watch["document_id"], short["document_id"]);
    assert_eq!(watch["library_entry_id"], short["library_entry_id"]);

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM documents WHERE user_id = $1")
        .bind(session.user.id.into_uuid())
        .fetch_one(app.pool())
        .await
        .unwrap();
    assert_eq!(count, 1);

    assert_eq!(
        dispatch_pending_jobs(&app, "feed.prepare_document").await,
        1
    );
    let youtube_jobs: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM job_outbox WHERE job_type = 'document.youtube_ingest' \
         AND payload->>'document_id' = $1",
    )
    .bind(short["document_id"].as_str().unwrap())
    .fetch_one(app.pool())
    .await
    .unwrap();
    assert_eq!(youtube_jobs, 1);

    let readable_assets: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM archive_assets WHERE document_id = $1 \
         AND asset_kind = 'readable_html'",
    )
    .bind(
        short["document_id"]
            .as_str()
            .unwrap()
            .trim_start_matches("doc_")
            .parse::<uuid::Uuid>()
            .unwrap(),
    )
    .fetch_one(app.pool())
    .await
    .unwrap();
    assert_eq!(readable_assets, 0);
}
