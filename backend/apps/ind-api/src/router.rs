use std::time::Duration;

use axum::Router;
use axum::routing::get;
use http::Method;
use http::header::{self, HeaderName, HeaderValue};
use ind_http_api::middleware::ip_extract::TrustedProxies;
use ind_http_api::middleware::rate_limit::{RateLimitConfig, RateLimiters};
use ind_http_api::{
    AppState, account_routes, archive_routes, asset_proxy_routes, auth_routes, collection_routes,
    document_routes, email_alias_routes, email_sender_routes, entity_routes, epub_routes,
    event_routes, export_routes, extension_routes, feed_delivery_routes, feed_routes,
    highlight_routes, home_routes, import_routes, integration_routes, library_routes, mila_routes,
    onboarding_routes, rate_limited_auth_routes, scalar_ui, search_routes, settings_routes,
    smart_list_routes, swagger_ui, tag_routes, token_routes, tts_routes, webhook_routes,
};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

use crate::config::ServerConfig;
use crate::health;

pub fn build(
    mut state: AppState,
    config: &ServerConfig,
    rate_limit_config: RateLimitConfig,
) -> Router {
    let cors = build_cors_layer(&config.cors.origins);
    let trusted_proxies = TrustedProxies::from_entries(&config.network.trusted_proxies);
    state.trusted_proxies = trusted_proxies.clone();
    let rate_limiters = RateLimiters::new(rate_limit_config, trusted_proxies);
    let frontend_url = config.cors.frontend_url.clone();
    let max_upload_bytes = state.config.max_upload_bytes;
    let max_import_upload_bytes = state.config.max_import_upload_bytes;

    let mut app = Router::new()
        .route("/health", get(health::health_check))
        .route("/api/health", get(health::health_check))
        .merge(rate_limited_auth_routes(rate_limiters.clone()))
        .merge(auth_routes())
        .merge(account_routes())
        .merge(onboarding_routes())
        .merge(token_routes())
        .merge(settings_routes())
        .merge(library_routes(max_upload_bytes))
        .merge(event_routes())
        .merge(extension_routes())
        .merge(archive_routes())
        .merge(asset_proxy_routes())
        .merge(epub_routes())
        .merge(feed_routes())
        .merge(feed_delivery_routes())
        .merge(highlight_routes())
        .merge(document_routes())
        .merge(home_routes())
        .merge(search_routes())
        .merge(mila_routes())
        .merge(entity_routes())
        .merge(collection_routes())
        .merge(email_sender_routes())
        .merge(email_alias_routes())
        .merge(tag_routes())
        .merge(smart_list_routes())
        .merge(tts_routes())
        .merge(integration_routes(rate_limiters))
        .merge(import_routes(max_import_upload_bytes))
        .merge(export_routes())
        .merge(webhook_routes());

    {
        let fe = frontend_url.clone();
        app = app
            .route(
                "/extension/auth",
                get(move |req: axum::extract::Request| async move {
                    let query = req.uri().query().unwrap_or("");
                    let redirect = if query.is_empty() {
                        format!("{}/extension/auth", fe)
                    } else {
                        format!("{}/extension/auth?{}", fe, query)
                    };
                    axum::response::Redirect::temporary(&redirect)
                }),
            )
            .route(
                "/extension/auth/callback",
                get(|| async {
                    axum::response::Html(
                        "<html><body><p>Authorization complete. This tab will close automatically.</p></body></html>",
                    )
                }),
            );
    }

    let app = app.layer(axum::middleware::from_fn_with_state(
        state.clone(),
        ind_http_api::csrf_middleware,
    ));
    // Security headers are applied here, before the dev-only API-docs merge, so
    // the restrictive `default-src 'none'` CSP does not break Swagger/Scalar in
    // development (Axum `layer` only wraps routes added before the call).
    let mut app = apply_security_headers(app).with_state(state);

    if !config.is_production() {
        app = app.merge(swagger_ui());
        app = app.merge(scalar_ui());
    }

    // Mounted last so every API route wins the match, and after the security
    // headers for the same reason the API docs are: `default-src 'none'` would
    // stop the SPA loading its own bundle.
    if let Some(web_ui) = crate::web_ui::spa_service(std::path::Path::new(&config.server.web_root))
    {
        tracing::info!(web_root = %config.server.web_root, "serving web app");
        app = app.fallback_service(web_ui);
    }

    app.layer(cors)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &http::Request<_>| {
                let request_id = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .map(String::from)
                    .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());

                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    request_id = %request_id,
                )
            }),
        )
}

fn build_cors_layer(origins: &[String]) -> CorsLayer {
    let allowed_origins: Vec<http::HeaderValue> =
        origins.iter().filter_map(|o| o.parse().ok()).collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(move |origin, _| {
            allowed_origins.contains(origin) || is_browser_extension_origin(origin)
        }))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::COOKIE,
            HeaderName::from_static("x-csrf-token"),
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-client-type"),
            HeaderName::from_static("x-suppress-auth-redirect"),
            HeaderName::from_static("last-event-id"),
            HeaderName::from_static("idempotency-key"),
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}

fn is_browser_extension_origin(origin: &HeaderValue) -> bool {
    origin.to_str().is_ok_and(|origin| {
        origin.starts_with("moz-extension://") || origin.starts_with("chrome-extension://")
    })
}

/// Baseline security response headers for every API route.
///
/// CSP is API-appropriate (`default-src 'none'`): responses are JSON or
/// access-controlled asset streams, so the document context never needs to load
/// its own scripts/styles. `frame-ancestors 'none'` (plus `X-Frame-Options`)
/// closes clickjacking; `Cache-Control: private, no-store` keeps authenticated
/// bodies out of shared caches. HSTS omits `preload` deliberately so self-hosters
/// on shared parent domains are not forced into the preload list — the hosted
/// edge can add it.
fn security_header_pairs() -> [(HeaderName, HeaderValue); 7] {
    [
        (
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; includeSubDomains"),
        ),
        (
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'",
            ),
        ),
        (header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY")),
        (
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ),
        (
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ),
        (
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static(
                "accelerometer=(), autoplay=(), camera=(), display-capture=(), encrypted-media=(), fullscreen=(), geolocation=(), gyroscope=(), microphone=(), payment=(), usb=()",
            ),
        ),
        (
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-store"),
        ),
    ]
}

/// Attach the baseline security headers with `if_not_present` semantics so routes
/// that set a stricter value themselves (the asset proxy's `Cache-Control` and
/// `Content-Disposition`) are never clobbered.
fn apply_security_headers<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let mut router = router;
    for (name, value) in security_header_pairs() {
        router = router.layer(SetResponseHeaderLayer::if_not_present(name, value));
    }
    router
}
