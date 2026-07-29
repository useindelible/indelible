#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::unimplemented,
        clippy::todo
    )
)]

pub mod error;
pub mod extract;
pub mod middleware;
pub mod openapi;
pub mod realtime;
pub mod response;
pub mod routes;
pub mod state;
pub mod validation;

pub use error::{ApiError, FieldError};
pub use extract::{Json, Validate, ValidatedJson};
pub use ind_application::ports::{
    AccountOperations, ApiTokenOperations, AuthOperations, ExtensionAuthOperations,
    ExtensionTokenResult, IntegrationAuthorizeStart, IntegrationOperations,
    IntegrationSyncEnqueued, OAuthOperations, OAuthTokenResult, OnboardingOperations,
};
pub use middleware::{
    AccessPolicy, ApiCredential, AvatarAssetAccess, CsrfToken, DocumentAssetAccess,
    PermissionAccess, Principal, RequireAccountSession, RequireAiRead, RequireAiReadAndLibraryRead,
    RequireAiUse, RequireAiUseAndLibraryRead, RequireAiWrite, RequireAiWriteAndAiUse,
    RequireAiWriteAndAiUseAndLibraryRead, RequireDocumentAssetRead, RequireExtensionAccess,
    RequireExtensionAccessJwt, RequireFeedsRead, RequireFeedsWrite, RequireIntegrationsRead,
    RequireIntegrationsWrite, RequireLibraryRead, RequireLibraryWrite, RequireMobileAccess,
    RequireMobileAccessJwt, RequireObsidianSync, RequireUserAccessJwt,
    RequireVerifiedUserAccessJwt, RequireVerifiedWebAccess, RequireVerifiedWebAccessJwt,
    RequireWebAccess, RequireWebhooksRead, RequireWebhooksWrite, TtsAssetAccess,
    clear_asset_cookie, clear_refresh_cookie, csrf_middleware, extract_refresh_cookie,
    set_asset_cookie, set_refresh_cookie,
};
pub use openapi::{ApiDoc, scalar_ui, swagger_ui};
pub use response::{ApiResponse, EmptyResponse, PaginatedResponse};
pub use routes::{
    account_routes, archive_routes, asset_proxy_routes, auth_routes, collection_routes,
    document_routes, email_alias_routes, email_sender_routes, entity_routes, epub_routes,
    event_routes, export_routes, extension_routes, feed_delivery_routes, feed_routes,
    highlight_routes, home_routes, import_routes, integration_routes, library_routes, mila_routes,
    onboarding_routes, rate_limited_auth_routes, search_routes, settings_routes, smart_list_routes,
    tag_routes, token_routes, tts_routes, webhook_routes,
};
pub use state::{AppConfig, AppState, Environment, HighlightWithNote};
