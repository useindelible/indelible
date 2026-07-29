pub mod account;
pub mod archive;
pub mod asset_proxy;
pub mod auth;
pub mod collections;
pub mod documents;
pub mod email_aliases;
pub mod email_senders;
pub mod entities;
pub mod epub;
pub mod events;
pub mod export;
pub mod extension;
pub mod feed_deliveries;
pub mod feeds;
pub mod highlights;
pub mod home;
pub mod imports;
pub mod integrations;
pub mod library;
pub mod mila;
pub mod onboarding;
pub mod search;
pub mod settings;
pub mod smart_lists;
pub(crate) mod sse;
pub mod tags;
pub mod tokens;
pub mod tts;
pub mod webhooks;

pub use account::account_routes;
pub use archive::archive_routes;
pub use asset_proxy::asset_proxy_routes;
pub use collections::collection_routes;
pub use documents::document_routes;
pub use email_aliases::email_alias_routes;
pub use email_senders::email_sender_routes;
pub use entities::entity_routes;
pub use epub::epub_routes;
pub use events::event_routes;
pub use export::export_routes;
pub use extension::extension_routes;
pub use feed_deliveries::feed_delivery_routes;
pub use feeds::feed_routes;
pub use highlights::highlight_routes;
pub use home::home_routes;
pub use imports::import_routes;
pub use integrations::integration_routes;
pub use library::library_routes;
pub use mila::mila_routes;
pub use onboarding::onboarding_routes;
pub use search::search_routes;
pub use settings::settings_routes;
pub use smart_lists::smart_list_routes;
pub use tags::tag_routes;
pub use tokens::token_routes;
pub use tts::tts_routes;
pub use webhooks::webhook_routes;

use axum::Router;

use crate::middleware::rate_limit::{
    RateLimiters, login_account_rate_limit, login_rate_limit, password_reset_account_rate_limit,
    password_reset_rate_limit, registration_rate_limit,
};
use crate::state::AppState;

pub fn auth_routes() -> Router<AppState> {
    use axum::routing::{get, post};

    Router::new()
        .route("/api/v1/auth/logout", post(auth::logout))
        .route("/api/v1/auth/refresh", post(auth::refresh))
        .route(
            "/api/v1/auth/refresh-tokens",
            get(auth::list_refresh_tokens).delete(auth::revoke_all_refresh_tokens),
        )
        .route("/api/v1/auth/password/reset", post(auth::reset_password))
        .route("/api/v1/auth/email/resend", post(auth::resend_verification))
        .route("/api/v1/auth/email/verify", post(auth::verify_email))
        .route("/api/v1/auth/providers", get(auth::list_providers))
        .route(
            "/api/v1/auth/oauth/native/token",
            post(auth::native_oauth_token),
        )
        .route(
            "/api/v1/auth/oauth/{provider}/native/start",
            get(auth::native_oauth_start),
        )
        .route(
            "/api/v1/auth/oauth/{provider}/start",
            get(auth::oauth_start),
        )
        .route(
            "/api/v1/auth/oauth/{provider}/callback",
            get(auth::oauth_callback).post(auth::oauth_callback_post),
        )
}

/// Routes mounted with auth rate limiting in `ind-api` bootstrap.
pub fn rate_limited_auth_routes(limiters: RateLimiters) -> Router<AppState> {
    use axum::routing::post;

    Router::new()
        .route(
            "/api/v1/auth/register",
            post(auth::register).route_layer(axum::middleware::from_fn_with_state(
                limiters.clone(),
                registration_rate_limit,
            )),
        )
        .route(
            "/api/v1/auth/login",
            post(auth::login)
                // Inner: per-account (per-email) throttle. Outer: per-IP. The IP
                // layer (added last) runs first and fails fast on IP floods.
                .route_layer(axum::middleware::from_fn_with_state(
                    limiters.clone(),
                    login_account_rate_limit,
                ))
                .route_layer(axum::middleware::from_fn_with_state(
                    limiters.clone(),
                    login_rate_limit,
                )),
        )
        .route(
            "/api/v1/auth/password/forgot",
            post(auth::forgot_password)
                .route_layer(axum::middleware::from_fn_with_state(
                    limiters.clone(),
                    password_reset_account_rate_limit,
                ))
                .route_layer(axum::middleware::from_fn_with_state(
                    limiters,
                    password_reset_rate_limit,
                )),
        )
}
