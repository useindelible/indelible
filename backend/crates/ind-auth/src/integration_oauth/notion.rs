use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use crate::integration_oauth::{
    IntegrationOAuthError, IntegrationOAuthProviderAdapter, ProviderTokens,
};
use ind_domain::IntegrationOAuthProvider;

const ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

pub struct NotionOAuthAdapter {
    client_id: String,
    client_secret: String,
    api_base: String,
    redirect_uri: String,
    http: reqwest::Client,
}

impl NotionOAuthAdapter {
    pub fn new(
        client_id: String,
        client_secret: String,
        api_base: String,
        redirect_uri: String,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            api_base,
            redirect_uri,
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl IntegrationOAuthProviderAdapter for NotionOAuthAdapter {
    fn provider(&self) -> IntegrationOAuthProvider {
        IntegrationOAuthProvider::Notion
    }

    fn authorize_url(&self, state: &str, _redirect_uri: &str) -> String {
        let encoded_redirect = utf8_percent_encode(&self.redirect_uri, ENCODE_SET).to_string();
        let encoded_state = utf8_percent_encode(state, ENCODE_SET).to_string();
        let encoded_client_id = utf8_percent_encode(&self.client_id, ENCODE_SET).to_string();
        format!(
            "{base}/v1/oauth/authorize?client_id={cid}&response_type=code&owner=user&redirect_uri={redir}&state={state}",
            base = self.api_base.trim_end_matches('/'),
            cid = encoded_client_id,
            redir = encoded_redirect,
            state = encoded_state,
        )
    }

    async fn exchange_code(
        &self,
        code: &str,
        _state: &str,
    ) -> Result<ProviderTokens, IntegrationOAuthError> {
        use base64::Engine;
        let credentials = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", self.client_id, self.client_secret));

        let body = serde_json::json!({
            "grant_type": "authorization_code",
            "code": code,
            "redirect_uri": self.redirect_uri,
        });

        let resp = self
            .http
            .post(format!(
                "{}/v1/oauth/token",
                self.api_base.trim_end_matches('/')
            ))
            .header("Authorization", format!("Basic {credentials}"))
            .header("Notion-Version", ind_domain::NOTION_API_VERSION)
            .json(&body)
            .send()
            .await
            .map_err(|e| IntegrationOAuthError::Exchange(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(IntegrationOAuthError::Exchange(format!(
                "HTTP {status}: {text}"
            )));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| IntegrationOAuthError::Exchange(e.to_string()))?;

        let access_token = data["access_token"]
            .as_str()
            .ok_or_else(|| IntegrationOAuthError::Exchange("missing access_token".into()))?
            .to_string();
        let refresh_token = data["refresh_token"].as_str().map(str::to_string);

        let extra = serde_json::json!({
            "workspace_id": data["workspace_id"],
            "workspace_name": data["workspace_name"],
            "workspace_icon": data["workspace_icon"],
            "bot_id": data["bot_id"],
            "owner_type": data["owner"]["type"],
            "duplicated_template_id": data["duplicated_template_id"],
        });

        Ok(ProviderTokens {
            access_token,
            refresh_token,
            expires_at: None,
            extra,
        })
    }

    async fn revoke_token(&self, access_token: &str) -> Result<(), IntegrationOAuthError> {
        use base64::Engine;
        let credentials = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", self.client_id, self.client_secret));

        let resp = self
            .http
            .post(format!(
                "{}/v1/oauth/revoke",
                self.api_base.trim_end_matches('/')
            ))
            .header("Authorization", format!("Basic {credentials}"))
            .header("Notion-Version", ind_domain::NOTION_API_VERSION)
            .json(&serde_json::json!({ "token": access_token }))
            .send()
            .await
            .map_err(|e| IntegrationOAuthError::Exchange(e.to_string()))?;

        let status = resp.status();
        if status.is_success() {
            return Ok(());
        }

        let text = resp.text().await.unwrap_or_default();
        // Notion reports an already-dead token as 400 invalid_grant; the goal
        // state is reached, so retries stay idempotent. Every other failure —
        // including 401 invalid_client, which means OUR credentials are wrong —
        // must surface so the disconnect can be retried.
        if status.as_u16() == 400 {
            // Notion error objects carry `code`; the RFC 6749 OAuth shape uses
            // `error`. Accept either spelling of the same condition.
            let code = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|body| {
                    body.get("code")
                        .or_else(|| body.get("error"))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                });
            if code.as_deref() == Some("invalid_grant") {
                return Ok(());
            }
        }
        Err(IntegrationOAuthError::Exchange(format!(
            "token revocation failed: HTTP {}: {text}",
            status.as_u16()
        )))
    }
}
