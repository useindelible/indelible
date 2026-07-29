mod dto;

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use chrono::Utc;
use http::{HeaderMap, StatusCode};
use ind_application::ports::{OAuthCallbackContext, OAuthCallbackResult};
use ind_auth::oauth_flow::{
    NativeOAuthFlow, OAuthFlowStorageError, StoredOAuthFlow, StoredOAuthFlowKind,
    consume_oauth_flow as consume_stored_oauth_flow, store_oauth_flow as store_stored_oauth_flow,
};
use ind_domain::{ClientType, OAuthProvider, UserId};
use url::form_urlencoded;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::{AuthUser, ClientIp};
use crate::middleware::{
    clear_asset_cookie, clear_refresh_cookie, extract_refresh_cookie, set_asset_cookie,
    set_refresh_cookie,
};
use crate::state::{AppConfig, AppState};
pub(crate) use dto::{
    AuthResponse, ForgotPasswordRequest, LoginRequest, MessageResponse, NativeOAuthStartQuery,
    NativeOAuthTokenForm, NativeOAuthTokenResponse, OAuthCallbackForm, OAuthCallbackQuery,
    OAuthProviderInfo, OAuthProvidersResponse, RefreshResponse, RefreshTokenDetail,
    RefreshTokenListResponse, RefreshTokenRequest, RegisterRequest, ResetPasswordRequest,
    VerifyEmailRequest,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub(crate) const NATIVE_OAUTH_REDIRECT_URI: &str = "com.useindelible.app:/oauth/callback";
const OAUTH_FLOW_MAX_AGE_SECS: i64 = 600;
const NATIVE_APP_STATE_MIN_LEN: usize = 22;
const NATIVE_PKCE_CHALLENGE_MIN_LEN: usize = 43;
const NATIVE_PKCE_CHALLENGE_MAX_LEN: usize = 128;

mod helpers;
pub(crate) mod oauth;
pub(crate) mod sessions;

pub use oauth::*;
pub use sessions::*;
