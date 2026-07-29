use ind_auth::{decode_asset_cookie_secret, sign_asset_cookie};
use ind_domain::ClientType;
use ind_test_support::{TestPersonalAccessToken, test_app::TEST_ASSET_COOKIE_SECRET_HEX};
use reqwest::StatusCode;
use serde_json::json;

use super::common::{SaveScenario, assert_json_response, document_id_from_response};
use super::resource_route_permission_support::{RouteCase, RoutePermissionFixture};

const AI_READ_ROUTES: &[RouteCase] = &[
    RouteCase::get("/api/v1/mila/status"),
    RouteCase::get("/api/v1/mila/config"),
    RouteCase::get("/api/v1/mila/presets"),
    RouteCase::get("/api/v1/mila/sessions"),
    RouteCase::get("/api/v1/mila/sessions/bad/messages"),
    RouteCase::get("/api/v1/tts/voice-personas"),
    RouteCase::get("/api/v1/documents/bad/playback?kind=tts"),
];

const AI_WRITE_ROUTES: &[RouteCase] = &[
    RouteCase::post("/api/v1/mila/config"),
    RouteCase::post("/api/v1/mila/presets"),
    RouteCase::patch("/api/v1/mila/presets/bad"),
    RouteCase::delete("/api/v1/mila/presets/bad"),
    RouteCase::post("/api/v1/mila/sessions"),
    RouteCase::delete("/api/v1/mila/sessions/bad"),
    RouteCase::patch("/api/v1/documents/bad/playback"),
];

const AI_WRITE_AND_AI_USE_ROUTES: &[RouteCase] = &[RouteCase::post("/api/v1/tts/voice-personas")];

const AI_USE_ROUTES: &[RouteCase] = &[RouteCase::post("/api/v1/mila/config/test")];

const AI_WRITE_AI_USE_AND_LIBRARY_READ_ROUTES: &[RouteCase] =
    &[RouteCase::post("/api/v1/mila/config/reindex")];

const AI_USE_AND_LIBRARY_READ_ROUTES: &[RouteCase] = &[
    RouteCase::get("/api/v1/mila/stream"),
    RouteCase::post("/api/v1/documents/bad/tts/sessions"),
];

const AI_READ_AND_LIBRARY_READ_ROUTES: &[RouteCase] = &[
    RouteCase::get("/api/v1/documents/bad/tts/chunks/chunk?session_id=bad"),
    RouteCase::get(
        "/api/v1/documents/bad/tts/timestamp?session_id=bad&chunk_id=chunk&element_index=0",
    ),
    RouteCase::get("/api/v1/assets/documents/bad/tts/bad/chunk.mp3"),
];

const OBSIDIAN_SYNC_ROUTES: &[RouteCase] = &[
    RouteCase::post("/api/v1/export/obsidian/runs"),
    RouteCase::get("/api/v1/export/obsidian/runs/bad"),
    RouteCase::get("/api/v1/export/obsidian/artifacts/bad"),
    RouteCase::post("/api/v1/export/obsidian/runs/bad/ack"),
    RouteCase::post("/api/v1/export/obsidian/refresh"),
    RouteCase::post("/api/v1/export/obsidian/rename"),
];

#[tokio::test]
async fn ai_routes_enforce_read_write_and_use_permissions_independently() {
    let fixture = RoutePermissionFixture::new().await;

    fixture
        .assert_pat_matrix("ai:read", "ai:use", AI_READ_ROUTES)
        .await;
    fixture
        .assert_pat_matrix("ai:write", "ai:read", AI_WRITE_ROUTES)
        .await;
    fixture
        .assert_pat_matrix("ai:use", "ai:write", AI_USE_ROUTES)
        .await;
}

#[tokio::test]
async fn document_backed_ai_use_requires_ai_use_and_library_read() {
    let fixture = RoutePermissionFixture::new().await;
    fixture
        .assert_pat_composite_matrix(
            &["library:read", "ai:use"],
            &[&["library:read"], &["ai:use"]],
            AI_USE_AND_LIBRARY_READ_ROUTES,
        )
        .await;
}

#[tokio::test]
async fn voice_persona_creation_requires_ai_write_and_ai_use() {
    let fixture = RoutePermissionFixture::new().await;
    fixture
        .assert_pat_composite_matrix(
            &["ai:write", "ai:use"],
            &[&["ai:write"], &["ai:use"]],
            AI_WRITE_AND_AI_USE_ROUTES,
        )
        .await;
}

#[tokio::test]
async fn mila_config_reindex_requires_ai_write_ai_use_and_library_read() {
    let fixture = RoutePermissionFixture::new().await;
    fixture
        .assert_pat_composite_matrix(
            &["ai:write", "ai:use", "library:read"],
            &[
                &["ai:write", "ai:use"],
                &["ai:write", "library:read"],
                &["ai:use", "library:read"],
            ],
            AI_WRITE_AI_USE_AND_LIBRARY_READ_ROUTES,
        )
        .await;
}

#[tokio::test]
async fn document_backed_tts_reads_require_ai_read_and_library_read() {
    let fixture = RoutePermissionFixture::new().await;
    fixture
        .assert_pat_composite_matrix(
            &["library:read", "ai:read"],
            &[&["library:read"], &["ai:read"]],
            AI_READ_AND_LIBRARY_READ_ROUTES,
        )
        .await;
}

#[tokio::test]
async fn extension_jwt_is_not_relaxed_for_general_ai_or_tts_routes() {
    let fixture = RoutePermissionFixture::new().await;
    let owner = fixture.app.create_web_session().await;
    let extension = fixture
        .app
        .create_client_session(&owner.user, ClientType::Extension);

    for case in [AI_READ_ROUTES[0], AI_READ_AND_LIBRARY_READ_ROUTES[2]] {
        let response = fixture.request(&extension, case).await;
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "Extension JWT must not reach {} {}",
            case.method,
            case.path,
        );
    }
}

#[tokio::test]
async fn obsidian_sync_remains_additive_with_unrelated_permissions() {
    let fixture = RoutePermissionFixture::new().await;
    let combined = fixture
        .mint_token_with_permissions(
            "combined obsidian automation",
            &["library:read", "obsidian:sync"],
        )
        .await;
    let unrelated = fixture
        .mint_token_with_permissions("non-obsidian automation", &["library:read", "ai:use"])
        .await;

    for case in OBSIDIAN_SYNC_ROUTES {
        let allowed = fixture.request(&combined, *case).await;
        assert!(
            !matches!(
                allowed.status(),
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
            ),
            "combined PAT must reach {} {}, got {}",
            case.method,
            case.path,
            allowed.status(),
        );
        let denied = fixture.request(&unrelated, *case).await;
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn document_assets_accept_library_pat_extension_jwt_and_asset_cookie() {
    let scenario = SaveScenario::new().await;
    let saved = scenario
        .extension_reader_save("https://example.com/task-7-document-asset-policy")
        .await;
    let document_id = document_id_from_response(&saved);
    let path = format!("/api/v1/assets/documents/{document_id}/readable_html");
    let metadata_path = format!("/api/v1/documents/{document_id}/assets/readable_html");

    let created = assert_json_response(
        scenario
            .web_client()
            .post_json(
                "/api/v1/tokens",
                &json!({"name": "document asset reader", "permissions": ["library:read"]}),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let library_pat = TestPersonalAccessToken::new(created["raw_token"].as_str().expect("raw PAT"));
    let pat_response = scenario.app.authed_client(&library_pat).get(&path).await;
    assert_eq!(pat_response.status(), StatusCode::OK);
    let pat_metadata = scenario
        .app
        .authed_client(&library_pat)
        .get(&metadata_path)
        .await;
    assert_eq!(pat_metadata.status(), StatusCode::OK);

    let unrelated = assert_json_response(
        scenario
            .web_client()
            .post_json(
                "/api/v1/tokens",
                &json!({"name": "non-library token", "permissions": ["ai:use"]}),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let unrelated_pat =
        TestPersonalAccessToken::new(unrelated["raw_token"].as_str().expect("raw unrelated PAT"));
    let unrelated_response = scenario.app.authed_client(&unrelated_pat).get(&path).await;
    assert_eq!(unrelated_response.status(), StatusCode::FORBIDDEN);

    let extension_response = scenario.extension_client().get(&path).await;
    assert_eq!(extension_response.status(), StatusCode::OK);
    assert!(
        extension_response
            .text()
            .await
            .expect("readable HTML body")
            .contains("Integration Reader Article")
    );
    let extension_metadata = scenario.extension_client().get(&metadata_path).await;
    assert_eq!(extension_metadata.status(), StatusCode::OK);

    let cookie = sign_asset_cookie(
        &scenario.web.user.id,
        &decode_asset_cookie_secret(TEST_ASSET_COOKIE_SECRET_HEX)
            .expect("test asset cookie secret is hex"),
    );
    let cookie_response = scenario
        .app
        .client()
        .get(format!("{}{}", scenario.app.address, path))
        .header(reqwest::header::COOKIE, format!("ind_asset={cookie}"))
        .send()
        .await
        .expect("cookie document asset request");
    assert_eq!(cookie_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn avatar_assets_reject_pat_and_extension_but_accept_cookie() {
    let fixture = RoutePermissionFixture::new().await;
    let owner = fixture.app.create_web_session().await;
    let path = format!("/api/v1/assets/{}/avatars/missing.png", owner.user.id);
    let extension = fixture
        .app
        .create_client_session(&owner.user, ClientType::Extension);
    let library_pat = fixture.mint_token("avatar reader", "library:read").await;

    for response in [
        fixture.app.authed_client(&extension).get(&path).await,
        fixture.app.authed_client(&library_pat).get(&path).await,
    ] {
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    for client_type in [
        ClientType::Web,
        ClientType::Ios,
        ClientType::Android,
        ClientType::Desktop,
        ClientType::Cli,
    ] {
        let session = fixture.app.create_client_session(&owner.user, client_type);
        let response = fixture.app.authed_client(&session).get(&path).await;
        assert!(
            !matches!(
                response.status(),
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
            ),
            "{client_type:?} JWT must reach the avatar handler, got {}",
            response.status(),
        );
    }

    let cookie = sign_asset_cookie(
        &owner.user.id,
        &decode_asset_cookie_secret(TEST_ASSET_COOKIE_SECRET_HEX)
            .expect("test asset cookie secret is hex"),
    );
    let cookie_response = fixture
        .app
        .client()
        .get(format!("{}{}", fixture.app.address, path))
        .header(reqwest::header::COOKIE, format!("ind_asset={cookie}"))
        .send()
        .await
        .expect("cookie avatar request");
    assert!(
        !matches!(
            cookie_response.status(),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ),
        "valid asset cookie must reach the avatar handler, got {}",
        cookie_response.status(),
    );
}
