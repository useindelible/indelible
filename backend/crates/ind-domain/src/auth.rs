use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::string_enum::impl_string_enum;
use crate::{ApiTokenId, AuthorizationCodeId, OAuthIdentityId, RefreshTokenId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuthProvider {
    Google,
    Apple,
    Oidc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthIdentity {
    pub id: OAuthIdentityId,
    pub user_id: UserId,
    pub provider: OAuthProvider,
    pub provider_user_id: String,
    pub provider_email: Option<String>,
    pub access_token_enc: Option<Vec<u8>>,
    pub refresh_token_enc: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
    Web,
    Ios,
    Android,
    Desktop,
    Extension,
    Cli,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToken {
    pub id: ApiTokenId,
    pub user_id: UserId,
    pub name: String,
    pub token_hash: String,
    pub prefix: String,
    pub permissions: Vec<ApiPermission>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApiPermission {
    #[serde(rename = "library:read")]
    LibraryRead,
    #[serde(rename = "library:write")]
    LibraryWrite,
    #[serde(rename = "feeds:read")]
    FeedsRead,
    #[serde(rename = "feeds:write")]
    FeedsWrite,
    #[serde(rename = "integrations:read")]
    IntegrationsRead,
    #[serde(rename = "integrations:write")]
    IntegrationsWrite,
    #[serde(rename = "webhooks:read")]
    WebhooksRead,
    #[serde(rename = "webhooks:write")]
    WebhooksWrite,
    #[serde(rename = "ai:read")]
    AiRead,
    #[serde(rename = "ai:write")]
    AiWrite,
    #[serde(rename = "ai:use")]
    AiUse,
    #[serde(rename = "obsidian:sync")]
    ObsidianSync,
}

impl_string_enum!(ApiPermission, "API permission", {
    LibraryRead => "library:read",
    LibraryWrite => "library:write",
    FeedsRead => "feeds:read",
    FeedsWrite => "feeds:write",
    IntegrationsRead => "integrations:read",
    IntegrationsWrite => "integrations:write",
    WebhooksRead => "webhooks:read",
    WebhooksWrite => "webhooks:write",
    AiRead => "ai:read",
    AiWrite => "ai:write",
    AiUse => "ai:use",
    ObsidianSync => "obsidian:sync",
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: RefreshTokenId,
    pub family_id: uuid::Uuid,
    pub user_id: UserId,
    pub token_hash: String,
    pub client_type: ClientType,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub replaced_by: Option<RefreshTokenId>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub absolute_expires_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCode {
    pub id: AuthorizationCodeId,
    pub user_id: UserId,
    pub code_hash: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub client_type: ClientType,
    pub redirect_uri: String,
    pub used_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationToken {
    pub id: uuid::Uuid,
    pub user_id: UserId,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub id: uuid::Uuid,
    pub user_id: UserId,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
