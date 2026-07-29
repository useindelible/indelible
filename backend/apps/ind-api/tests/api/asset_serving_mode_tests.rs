//! Asset serving contract: every URL in an API response body points at the API
//! origin regardless of `asset_serving_mode`. The mode only changes how the
//! asset proxy endpoints answer: `passthrough` streams the bytes, `presigned`
//! redirects to a presigned S3 URL. Uploads always go through the API.

use ind_test_support::{TestAppOptions, spawn_app, spawn_app_with_options};
use reqwest::StatusCode;
use reqwest::multipart::{Form, Part};
use serde_json::Value;

use super::common::{SaveScenario, assert_json_response, document_id_from_response};

const PNG_BYTES: &[u8] = b"\x89PNG\r\n\x1a\ntiny-avatar-fixture";

fn assert_body_free_of_s3_endpoint(body: &Value, s3_endpoint: &str, context: &str) {
    let raw = body.to_string();
    assert!(
        !raw.contains(s3_endpoint),
        "{context} leaked the S3 endpoint {s3_endpoint}: {raw}"
    );
}

fn no_redirect_client() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn passthrough_asset_urls_stay_on_the_api_origin() {
    let scenario = SaveScenario::new().await;
    let saved = scenario
        .extension_reader_save("https://example.com/passthrough-contract")
        .await;
    let document_id = document_id_from_response(&saved);

    let asset = assert_json_response(
        scenario
            .web_client()
            .get(&format!(
                "/api/v1/documents/{document_id}/assets/readable_html"
            ))
            .await,
        StatusCode::OK,
    )
    .await;

    let download_url = asset["download_url"].as_str().expect("download_url");
    let expected = format!(
        "{}/api/v1/assets/documents/{document_id}/readable_html",
        scenario.app.address
    );
    assert_eq!(download_url, expected);

    let download = scenario
        .app
        .client()
        .get(download_url)
        .bearer_auth(&scenario.web.token)
        .send()
        .await
        .expect("download asset");
    assert_eq!(download.status(), StatusCode::OK);
    let text = download.text().await.expect("asset body");
    assert!(text.contains("Integration Reader Article"));

    let entry_id = saved["library_entry_id"].as_str().expect("entry id");
    let extension_asset = assert_json_response(
        scenario
            .extension_client()
            .get(&format!(
                "/api/v1/extension/entries/{entry_id}/assets/readable_html"
            ))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        extension_asset["download_url"].as_str().expect("url"),
        expected
    );
}

#[tokio::test]
async fn response_bodies_never_leak_the_s3_endpoint_in_either_mode() {
    for mode in ["passthrough", "presigned"] {
        let app = spawn_app_with_options(TestAppOptions {
            asset_serving_mode: mode,
            ..TestAppOptions::default()
        })
        .await;
        let s3_endpoint = app.s3_endpoint().to_string();
        let session = app.create_web_session().await;
        let extension = app.create_extension_session(&session.user).await;
        let client = app.authed_client(&session);

        let saved = assert_json_response(
            app.authed_client(&extension)
                .post_json(
                    "/api/v1/extension/reader-save",
                    &serde_json::json!({
                        "url": format!("https://example.com/leak-sweep-{mode}"),
                        "title": "Leak Sweep",
                        "reader_html": super::common::SIMPLE_READER_HTML,
                        "word_count": 10,
                        "reading_time_minutes": 1,
                        "language": "en"
                    }),
                )
                .await,
            StatusCode::ACCEPTED,
        )
        .await;
        assert_body_free_of_s3_endpoint(&saved, &s3_endpoint, "reader-save response");
        let document_id = document_id_from_response(&saved);

        let avatar_form = Form::new().part(
            "file",
            Part::bytes(PNG_BYTES.to_vec())
                .file_name("avatar.png")
                .mime_str("image/png")
                .expect("valid mime"),
        );
        let profile = assert_json_response(
            client
                .post_multipart("/api/v1/me/avatar", avatar_form)
                .await,
            StatusCode::OK,
        )
        .await;
        assert_body_free_of_s3_endpoint(&profile, &s3_endpoint, "avatar upload response");

        for path in [
            format!("/api/v1/documents/{document_id}"),
            format!("/api/v1/documents/{document_id}/assets/readable_html"),
            "/api/v1/me".to_string(),
        ] {
            let body = assert_json_response(client.get(&path).await, StatusCode::OK).await;
            assert_body_free_of_s3_endpoint(&body, &s3_endpoint, &format!("GET {path} ({mode})"));
        }
    }
}

#[tokio::test]
async fn presigned_mode_redirects_asset_streams_to_s3() {
    let app = spawn_app_with_options(TestAppOptions {
        asset_serving_mode: "presigned",
        ..TestAppOptions::default()
    })
    .await;
    let session = app.create_web_session().await;
    let extension = app.create_extension_session(&session.user).await;

    let saved = assert_json_response(
        app.authed_client(&extension)
            .post_json(
                "/api/v1/extension/reader-save",
                &serde_json::json!({
                    "url": "https://example.com/presigned-redirect",
                    "title": "Redirect Article",
                    "reader_html": super::common::SIMPLE_READER_HTML,
                    "word_count": 10,
                    "reading_time_minutes": 1,
                    "language": "en"
                }),
            )
            .await,
        StatusCode::ACCEPTED,
    )
    .await;
    let document_id = document_id_from_response(&saved);

    let response = no_redirect_client()
        .get(format!(
            "{}/api/v1/assets/documents/{document_id}/readable_html",
            app.address
        ))
        .bearer_auth(&session.token)
        .send()
        .await
        .expect("proxy request");
    assert_eq!(response.status(), StatusCode::FOUND);
    let location = response
        .headers()
        .get(http::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("redirect Location header");
    assert!(
        location.starts_with(app.s3_endpoint()),
        "expected presigned redirect to {}, got {location}",
        app.s3_endpoint()
    );

    // The redirect target must actually serve the bytes without API credentials.
    let bytes = app
        .client()
        .get(location)
        .send()
        .await
        .expect("follow presigned redirect")
        .error_for_status()
        .expect("presigned URL serves object")
        .text()
        .await
        .expect("object body");
    assert!(bytes.contains("Integration Reader Article"));
}

#[tokio::test]
async fn passthrough_mode_streams_asset_bytes_directly() {
    let scenario = SaveScenario::new().await;
    let saved = scenario
        .extension_reader_save("https://example.com/passthrough-streams")
        .await;
    let document_id = document_id_from_response(&saved);

    let response = no_redirect_client()
        .get(format!(
            "{}/api/v1/assets/documents/{document_id}/readable_html",
            scenario.app.address
        ))
        .bearer_auth(&scenario.web.token)
        .send()
        .await
        .expect("proxy request");
    assert_eq!(response.status(), StatusCode::OK);
    let text = response.text().await.expect("asset body");
    assert!(text.contains("Integration Reader Article"));
}

#[tokio::test]
async fn avatar_upload_goes_through_the_api_and_serves_from_it() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let form = Form::new().part(
        "file",
        Part::bytes(PNG_BYTES.to_vec())
            .file_name("avatar.png")
            .mime_str("image/png")
            .expect("valid mime"),
    );
    let profile = assert_json_response(
        client.post_multipart("/api/v1/me/avatar", form).await,
        StatusCode::OK,
    )
    .await;

    let avatar_url = profile["avatar_url"].as_str().expect("avatar_url");
    let expected_prefix = format!("{}/api/v1/assets/{}/avatars/", app.address, session.user.id);
    assert!(
        avatar_url.starts_with(&expected_prefix),
        "avatar_url {avatar_url} should start with {expected_prefix}"
    );

    let download = app
        .client()
        .get(avatar_url)
        .bearer_auth(&session.token)
        .send()
        .await
        .expect("fetch avatar");
    assert_eq!(download.status(), StatusCode::OK);
    let bytes = download.bytes().await.expect("avatar bytes");
    assert_eq!(bytes.as_ref(), PNG_BYTES);

    let me = assert_json_response(client.get("/api/v1/me").await, StatusCode::OK).await;
    assert_eq!(me["avatar_url"].as_str().expect("avatar_url"), avatar_url);
}

#[tokio::test]
async fn avatar_upload_rejects_unsupported_content_types() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let form = Form::new().part(
        "file",
        Part::bytes(b"GIF89a".to_vec())
            .file_name("avatar.gif")
            .mime_str("image/gif")
            .expect("valid mime"),
    );
    let response = client.post_multipart("/api/v1/me/avatar", form).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn avatar_upload_rejects_oversized_files() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let form = Form::new().part(
        "file",
        Part::bytes(vec![0u8; 2 * 1024 * 1024 + 1])
            .file_name("avatar.png")
            .mime_str("image/png")
            .expect("valid mime"),
    );
    let response = client.post_multipart("/api/v1/me/avatar", form).await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn presigned_avatar_upload_url_endpoint_is_gone() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let response = client
        .post_json(
            "/api/v1/me/avatar/upload-url",
            &serde_json::json!({ "content_type": "image/png" }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
