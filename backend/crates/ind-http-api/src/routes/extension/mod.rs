use axum::Router;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::routing::{get, post};
use http::HeaderMap;
use ind_application::handlers::extension_save::{
    FullArchiveInput, QuickSaveInput, ReaderSaveInput,
};
use ind_application::ports::PatchExtensionEntryRequest;
use ind_domain::{ClientType, ItemType};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::{ClientIp, RequireExtensionAccess, RequireWebAccess};
use crate::response::ApiResponse;
use crate::routes::highlights::{
    HighlightListResponse, HighlightResponse, HighlightWithNoteResponse, LocatorSchema,
    SourceLocatorSchema,
};
use crate::state::AppState;

// -- Response DTOs --

pub(crate) mod auth;
mod dto;
pub(crate) mod entries;
mod library_alias;
pub(crate) mod save;

pub use auth::*;
pub use dto::*;
pub use entries::*;
pub use save::*;

pub fn extension_routes() -> Router<AppState> {
    let save_routes = Router::new()
        .route("/api/v1/extension/reader-save", post(extension_reader_save))
        .route(
            "/api/v1/extension/full-archive",
            post(extension_full_archive),
        )
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024));

    Router::new()
        .route("/api/v1/extension/status", get(extension_status))
        .route("/api/v1/extension/check-url", get(extension_check_url))
        .route("/api/v1/extension/quick-save", post(extension_quick_save))
        .route(
            "/api/v1/extension/entries/{library_entry_id}",
            get(extension_get_entry).patch(extension_patch_entry),
        )
        .route(
            "/api/v1/extension/entries/{library_entry_id}/highlights",
            get(extension_list_highlights).post(extension_create_highlight),
        )
        .route(
            "/api/v1/extension/entries/{library_entry_id}/assets/{asset_kind}",
            get(extension_get_entry_asset),
        )
        .route(
            "/api/v1/extension/entries/{library_entry_id}/note",
            axum::routing::put(extension_upsert_note),
        )
        .route(
            "/api/v1/extension/entries/{library_entry_id}/tags",
            axum::routing::put(extension_replace_tags),
        )
        .merge(save_routes)
        .route("/api/v1/auth/extension/start", get(extension_auth_start))
        .route(
            "/api/v1/auth/extension/authorize",
            post(extension_authorize),
        )
        .route("/api/v1/auth/extension/token", post(extension_token))
        .route("/api/v1/auth/extension/refresh", post(extension_refresh))
        .route("/api/v1/auth/extension/revoke", post(extension_revoke))
}
