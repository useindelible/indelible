use reqwest::StatusCode;
use serde_json::json;

use ind_test_support::spawn_app;

use super::common::assert_json_response;

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
