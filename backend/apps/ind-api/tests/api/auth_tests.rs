use ind_test_support::{TestApp, TestAppOptions, spawn_app_with_options};

use super::common::assert_json_response;
use reqwest::StatusCode;

#[tokio::test]
async fn registration_login_and_profile_access_follow_the_public_auth_contract() {
    let app = TestApp::new().await;
    for path in ["/health", "/api/health"] {
        assert_eq!(app.get(path).await.status(), 200, "{path}");
    }
    let registered = app
        .post_json_anon(
            "/api/v1/auth/register",
            &serde_json::json!({
                "email": "login@example.com",
                "password": "SecureP@ss123!",
                "display_name": "Login User"
            }),
        )
        .await;
    assert_eq!(registered.status(), 201);
    let body: serde_json::Value = registered.json().await.unwrap();
    assert!(body["access_token"].is_string());
    assert!(body["id"].is_string());
    assert_eq!(body["email"], "login@example.com");

    let login = app
        .post_json_anon(
            "/api/v1/auth/login",
            &serde_json::json!({
                "email": "login@example.com",
                "password": "SecureP@ss123!"
            }),
        )
        .await;
    assert_eq!(login.status(), 200);
    let login: serde_json::Value = login.json().await.unwrap();
    assert!(login["access_token"].is_string());
    let rejected = app
        .post_json_anon(
            "/api/v1/auth/login",
            &serde_json::json!({
                "email": "login@example.com",
                "password": "WrongPassword1!"
            }),
        )
        .await;
    assert_eq!(rejected.status(), 401);
    assert_eq!(app.get("/api/v1/me").await.status(), 401);
    let session = app.create_web_session().await;
    let profile = app.authed_client(&session).get("/api/v1/me").await;
    assert_eq!(profile.status(), 200);
    assert_eq!(
        profile.json::<serde_json::Value>().await.unwrap()["id"],
        session.user.id.to_string()
    );
}

#[tokio::test]
async fn scoped_api_token_lifecycle_crosses_route_middleware_and_persistence() {
    let app = TestApp::new().await;
    let session = app.create_web_session().await;
    let account = app.authed_client(&session);

    let created = account
        .post_json(
            "/api/v1/tokens",
            &serde_json::json!({"name": "automation", "scopes": ["read"]}),
        )
        .await;
    assert_eq!(created.status(), 201);
    let token: serde_json::Value = created.json().await.unwrap();
    let id = token["id"].as_str().unwrap();
    let raw = token["raw_token"].as_str().unwrap();
    assert!(raw.starts_with("ind_"));
    assert_eq!(token["prefix"].as_str().unwrap(), &raw[..8]);

    let list: serde_json::Value = account.get("/api/v1/tokens").await.json().await.unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 1);
    let response = app
        .client()
        .get(format!("{}/api/v1/me", app.address))
        .bearer_auth(raw)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    assert_eq!(
        account
            .delete(&format!("/api/v1/tokens/{id}"))
            .await
            .status(),
        204
    );
    let response = app
        .client()
        .get(format!("{}/api/v1/me", app.address))
        .bearer_auth(raw)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn first_run_provider_and_concurrent_signup_admit_exactly_one_owner() {
    let app = spawn_app_with_options(TestAppOptions {
        allow_signups: false,
        ..TestAppOptions::default()
    })
    .await;
    let before: serde_json::Value = app
        .get("/api/v1/auth/providers")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(
        (
            before["setup_required"].as_bool(),
            before["signups_enabled"].as_bool()
        ),
        (Some(true), Some(true))
    );

    let bodies: Vec<serde_json::Value> = (0..5)
        .map(|i| {
            serde_json::json!({
                "email": format!("race{i}@example.com"),
                "password": "SecureP@ss123!",
                "display_name": format!("Racer {i}")
            })
        })
        .collect();

    let responses = futures::future::join_all(
        bodies
            .iter()
            .map(|body| app.post_json_anon("/api/v1/auth/register", body)),
    )
    .await;

    let created = responses.iter().filter(|r| r.status() == 201).count();
    let forbidden = responses.iter().filter(|r| r.status() == 403).count();
    assert_eq!((created, forbidden), (1, 4));

    let user_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(app.pool())
        .await
        .unwrap();
    assert_eq!(user_count, 1);
    let after: serde_json::Value = app
        .get("/api/v1/auth/providers")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(
        (
            after["setup_required"].as_bool(),
            after["signups_enabled"].as_bool()
        ),
        (Some(false), Some(false))
    );
}

#[tokio::test]
async fn cli_refresh_family_routes_rotate_list_and_revoke() {
    let app = TestApp::new().await;
    let register = app
        .client()
        .post(format!("{}/api/v1/auth/register", app.address))
        .header("x-client-type", "cli")
        .json(&serde_json::json!({
            "email": "refresh-family@example.com",
            "password": "SecureP@ss123!",
            "display_name": "Refresh Family"
        }))
        .send()
        .await
        .unwrap();
    let registered = assert_json_response(register, StatusCode::CREATED).await;
    let access = registered["access_token"].as_str().unwrap();
    let first_refresh = registered["refresh_token"].as_str().unwrap();

    let families = app
        .client()
        .get(format!("{}/api/v1/auth/refresh-tokens", app.address))
        .bearer_auth(access)
        .send()
        .await
        .unwrap();
    let families = assert_json_response(families, StatusCode::OK).await;
    assert_eq!(families["tokens"].as_array().unwrap().len(), 1);
    assert_eq!(families["tokens"][0]["client_type"], "cli");

    let refresh = |token: &str| {
        app.client()
            .post(format!("{}/api/v1/auth/refresh", app.address))
            .header("x-client-type", "cli")
            .json(&serde_json::json!({"refresh_token": token}))
            .send()
    };
    let rotated = assert_json_response(refresh(first_refresh).await.unwrap(), StatusCode::OK).await;
    let second_refresh = rotated["refresh_token"].as_str().unwrap();
    assert_ne!(second_refresh, first_refresh);
    let all = app
        .client()
        .delete(format!("{}/api/v1/auth/refresh-tokens", app.address))
        .bearer_auth(rotated["access_token"].as_str().unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(all.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        refresh(second_refresh).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn account_profile_password_and_deletion_mutations_preserve_the_public_contract() {
    let app = TestApp::new().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let profile = assert_json_response(
        client
            .patch_json(
                "/api/v1/me",
                &serde_json::json!({
                    "display_name": "Account Boundary",
                    "locale": "fr-FR",
                    "timezone": "Africa/Lagos",
                    "theme": "dark"
                }),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(profile["display_name"], "Account Boundary");
    assert_eq!(profile["locale"], "fr-FR");
    assert_eq!(profile["timezone"], "Africa/Lagos");
    assert_eq!(profile["theme"], "dark");
    assert_eq!(profile["has_password"], false);
    assert_eq!(
        client
            .patch_json("/api/v1/me", &serde_json::json!({"theme": "neon"}))
            .await
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    assert_json_response(
        client
            .post_json(
                "/api/v1/me/password",
                &serde_json::json!({
                    "current_password": "",
                    "new_password": "ReplacementP@ss123!"
                }),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let login = assert_json_response(
        app.post_json_anon(
            "/api/v1/auth/login",
            &serde_json::json!({
                "email": session.user.email,
                "password": "ReplacementP@ss123!"
            }),
        )
        .await,
        StatusCode::OK,
    )
    .await;
    assert!(login["access_token"].is_string());
    assert_eq!(
        client
            .post_json(
                "/api/v1/me/password",
                &serde_json::json!({
                    "current_password": "wrong",
                    "new_password": "AnotherP@ss123!"
                }),
            )
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        client
            .delete_json(
                "/api/v1/me",
                &serde_json::json!({"confirmation": "wrong@example.com"}),
            )
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        client
            .delete_json(
                "/api/v1/me",
                &serde_json::json!({"confirmation": session.user.email}),
            )
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    let status: String = sqlx::query_scalar("SELECT status FROM users WHERE id = $1")
        .bind(session.user.id.into_uuid())
        .fetch_one(app.pool())
        .await
        .unwrap();
    assert_eq!(status, "deleted");
}
