use ind_test_support::{
    TestApiCredential, TestApp, TestAuthSession, TestPersonalAccessToken, spawn_app,
};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::assert_json_response;

#[derive(Clone, Copy, Debug)]
pub struct RouteCase {
    pub method: &'static str,
    pub path: &'static str,
}

impl RouteCase {
    pub const fn get(path: &'static str) -> Self {
        Self {
            method: "GET",
            path,
        }
    }

    pub const fn post(path: &'static str) -> Self {
        Self {
            method: "POST",
            path,
        }
    }

    pub const fn patch(path: &'static str) -> Self {
        Self {
            method: "PATCH",
            path,
        }
    }

    pub const fn put(path: &'static str) -> Self {
        Self {
            method: "PUT",
            path,
        }
    }

    pub const fn delete(path: &'static str) -> Self {
        Self {
            method: "DELETE",
            path,
        }
    }
}

pub struct RoutePermissionFixture {
    pub app: TestApp,
    web: TestAuthSession,
}

impl RoutePermissionFixture {
    pub async fn new() -> Self {
        let app = spawn_app().await;
        let web = app.create_web_session().await;
        Self { app, web }
    }

    pub async fn mint_token(&self, name: &str, permission: &str) -> TestPersonalAccessToken {
        self.mint_token_with_permissions(name, &[permission]).await
    }

    pub async fn mint_token_with_permissions(
        &self,
        name: &str,
        permissions: &[&str],
    ) -> TestPersonalAccessToken {
        let created = assert_json_response(
            self.app
                .authed_client(&self.web)
                .post_json(
                    "/api/v1/tokens",
                    &json!({"name": name, "permissions": permissions}),
                )
                .await,
            StatusCode::CREATED,
        )
        .await;
        TestPersonalAccessToken::new(
            created["raw_token"]
                .as_str()
                .expect("created token exposes raw_token"),
        )
    }

    pub async fn assert_pat_matrix(
        &self,
        required_permission: &str,
        denied_permission: &str,
        cases: &[RouteCase],
    ) {
        let allowed = self
            .mint_token(
                &format!("{required_permission} route matrix"),
                required_permission,
            )
            .await;
        let denied = self
            .mint_token(
                &format!("{denied_permission} denied route matrix"),
                denied_permission,
            )
            .await;

        for case in cases {
            let allowed_response = self.request(&allowed, *case).await;
            let allowed_status = allowed_response.status();
            assert!(
                !matches!(
                    allowed_status,
                    StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
                ),
                "{required_permission} PAT must authenticate and reach {} {}, got {allowed_status}",
                case.method,
                case.path,
            );

            let denied_response = self.request(&denied, *case).await;
            let denied_status = denied_response.status();
            let denied_body = denied_response
                .json::<Value>()
                .await
                .unwrap_or_else(|error| {
                    panic!(
                        "{} {} denial did not return JSON: {error}",
                        case.method, case.path
                    )
                });
            assert_eq!(
                denied_status,
                StatusCode::FORBIDDEN,
                "{denied_permission} PAT must be denied by {} {}: {denied_body}",
                case.method,
                case.path,
            );
            assert_eq!(
                denied_body["code"], "insufficient_permissions",
                "{} {} must fail at its named permission policy",
                case.method, case.path,
            );
        }
    }

    pub async fn assert_pat_composite_matrix(
        &self,
        required_permissions: &[&str],
        incomplete_permission_sets: &[&[&str]],
        cases: &[RouteCase],
    ) {
        let allowed = self
            .mint_token_with_permissions("composite route matrix", required_permissions)
            .await;
        let mut denied = Vec::with_capacity(incomplete_permission_sets.len());
        for (index, permissions) in incomplete_permission_sets.iter().enumerate() {
            denied.push(
                self.mint_token_with_permissions(
                    &format!("incomplete composite route matrix {index}"),
                    permissions,
                )
                .await,
            );
        }

        for case in cases {
            let allowed_response = self.request(&allowed, *case).await;
            let allowed_status = allowed_response.status();
            assert!(
                !matches!(
                    allowed_status,
                    StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
                ),
                "{required_permissions:?} PAT must authenticate and reach {} {}, got {allowed_status}",
                case.method,
                case.path,
            );

            for (permissions, token) in incomplete_permission_sets.iter().zip(&denied) {
                let denied_response = self.request(token, *case).await;
                let denied_status = denied_response.status();
                let denied_body = denied_response
                    .json::<Value>()
                    .await
                    .unwrap_or_else(|error| {
                        panic!(
                            "{} {} denial did not return JSON: {error}",
                            case.method, case.path
                        )
                    });
                assert_eq!(
                    denied_status,
                    StatusCode::FORBIDDEN,
                    "{permissions:?} PAT must be denied by {} {}: {denied_body}",
                    case.method,
                    case.path,
                );
                assert_eq!(
                    denied_body["code"], "insufficient_permissions",
                    "{} {} must fail at its composite permission policy",
                    case.method, case.path,
                );
            }
        }
    }

    pub async fn assert_jwt_only_matrix(&self, cases: &[RouteCase]) {
        let pat = self
            .mint_token("JWT-only route matrix", "library:read")
            .await;

        for case in cases {
            let jwt_response = self.request(&self.web, *case).await;
            let jwt_status = jwt_response.status();
            assert!(
                !matches!(jwt_status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN),
                "verified user JWT must authenticate and reach {} {}, got {jwt_status}",
                case.method,
                case.path,
            );

            let pat_response = self.request(&pat, *case).await;
            assert_eq!(
                pat_response.status(),
                StatusCode::FORBIDDEN,
                "PAT must not reach aggregate route {} {}",
                case.method,
                case.path,
            );
        }
    }

    pub async fn request<C: TestApiCredential>(
        &self,
        credential: &C,
        case: RouteCase,
    ) -> reqwest::Response {
        let client = self.app.authed_client(credential);
        match case.method {
            "GET" => client.get(case.path).await,
            "POST" => client.post_json(case.path, &json!({})).await,
            "PATCH" => client.patch_json(case.path, &json!({})).await,
            "PUT" => client.put_json(case.path, &json!({})).await,
            "DELETE" => client.delete(case.path).await,
            method => panic!("unsupported route case method {method}"),
        }
    }
}
