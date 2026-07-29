use axum::response::{IntoResponse, Response};
use http::StatusCode;
use ind_application::repos::{Cursor, Page};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    #[serde(flatten)]
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}

pub struct EmptyResponse;

impl IntoResponse for EmptyResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(bound = "T: utoipa::ToSchema")]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub page: PageInfo,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PageInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

impl<T: Serialize> IntoResponse for PaginatedResponse<T> {
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}

impl<T: Serialize> From<Page<T>> for PaginatedResponse<T> {
    fn from(page: Page<T>) -> Self {
        let has_more = page.next_cursor.is_some();
        Self {
            data: page.items,
            page: PageInfo {
                next_cursor: page.next_cursor.map(|Cursor(c)| c),
                has_more,
            },
        }
    }
}
