use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use http::{HeaderMap, StatusCode};
use ind_application::storage::ByteRange;
use ind_domain::{DocumentId, TtsSessionId};

use crate::error::ApiError;
use crate::middleware::TtsAssetAccess;
use crate::state::AppState;

/// Stream a TTS audio chunk with optional HTTP Range support.
///
/// The URL is scoped by document, session, and stable chunk id so the same
/// `chunk_id` can safely exist for multiple voices, formats, and speeds.
#[utoipa::path(
    get,
    path = "/api/v1/assets/documents/{document_id}/tts/{session_id}/{chunk_file}",
    params(
        ("document_id" = String, Path, description = "Document id with doc_ prefix"),
        ("session_id" = String, Path, description = "Session id with tss_ prefix"),
        ("chunk_file" = String, Path, description = "Chunk id plus audio extension"),
    ),
    responses(
        (status = 200, description = "Full audio body", content_type = "audio/mpeg"),
        (status = 206, description = "Partial content", content_type = "audio/mpeg"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Chunk not found"),
        (status = 416, description = "Range not satisfiable"),
    ),
    security(("bearer" = []), ("api_token" = []), ("asset_cookie" = [])),
    extensions(("x-indelible-permissions" = json!(["ai:read", "library:read"]))),
    tag = "TTS",
)]
pub async fn stream_session_chunk_audio(
    asset_access: TtsAssetAccess,
    State(state): State<AppState>,
    Path((document_id, session_id, chunk_file)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let ops = state
        .tts_ops
        .as_deref()
        .ok_or_else(|| ApiError::ServiceUnavailable {
            message: "tts service not configured".into(),
        })?;

    let parsed_document_id: DocumentId = document_id.parse().map_err(|_| ApiError::NotFound {
        entity: "Document",
        id: document_id.clone(),
    })?;
    let parsed_session_id: TtsSessionId = session_id.parse().map_err(|_| ApiError::NotFound {
        entity: "TtsSession",
        id: session_id.clone(),
    })?;
    let chunk_id = chunk_file
        .rsplit_once('.')
        .map(|(chunk_id, _)| chunk_id)
        .unwrap_or(chunk_file.as_str())
        .to_string();

    let range_header = headers
        .get(http::header::RANGE)
        .and_then(|h| h.to_str().ok())
        .map(str::to_owned);

    // Parse the range header up-front so total_length is known before the body
    // is consumed. A malformed byte-range spec is a 416 regardless of whether
    // the bytes would exist, matching RFC 9110.
    let parsed_range = match range_header.as_deref() {
        None => None,
        Some(raw) => match parse_range_header(raw) {
            Ok(r) => Some(r),
            Err(RangeError::Malformed) => {
                return Err(ApiError::BadRequest {
                    message: format!("malformed Range header: {raw}"),
                });
            }
            Err(RangeError::Unsupported) => {
                // Multi-range or suffix-with-prefix: we serve the full body in
                // that case, matching conservative server behaviour.
                None
            }
        },
    };

    let data = ops
        .get_session_chunk_audio(
            asset_access.user_id,
            parsed_document_id,
            parsed_session_id,
            chunk_id.clone(),
            parsed_range,
        )
        .await
        .map_err(ApiError::from)?;

    let total = data.total_length;
    if let Some(mut br) = data.range {
        let total_u = total as u64;
        if total_u > 0 && br.end_inclusive >= total_u {
            br.end_inclusive = total_u - 1;
        }
        if br.start > br.end_inclusive || br.start >= total_u {
            return Err(ApiError::RangeNotSatisfiable { total });
        }
        let content_range = format!("bytes {}-{}/{}", br.start, br.end_inclusive, total);
        let response = Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
            .header(http::header::CONTENT_TYPE, &data.content_type)
            .header(http::header::CONTENT_LENGTH, data.content_length)
            .header(http::header::CONTENT_RANGE, content_range)
            .header(http::header::ACCEPT_RANGES, "bytes")
            .header(http::header::CACHE_CONTROL, "private, max-age=3600")
            .body(Body::from_stream(data.body))
            .map_err(|error| ApiError::Internal {
                message: format!("failed to build audio response: {error}"),
            })?
            .into_response();
        return Ok(response);
    }

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, &data.content_type)
        .header(http::header::CONTENT_LENGTH, data.content_length)
        .header(http::header::ACCEPT_RANGES, "bytes")
        .header(http::header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from_stream(data.body))
        .map_err(|error| ApiError::Internal {
            message: format!("failed to build audio response: {error}"),
        })?
        .into_response();
    Ok(response)
}

#[derive(Debug)]
enum RangeError {
    Malformed,
    Unsupported,
}

/// Parse a single-range HTTP Range header of the form `bytes=a-b` or `bytes=a-`.
/// Suffix ranges (`bytes=-N`) and multi-range specs are treated as unsupported
/// so the handler falls back to a full 200 response rather than misrepresenting
/// the payload.
fn parse_range_header(raw: &str) -> Result<ByteRange, RangeError> {
    let value = raw.trim();
    let spec = value.strip_prefix("bytes=").ok_or(RangeError::Malformed)?;
    if spec.contains(',') {
        return Err(RangeError::Unsupported);
    }
    let (start, end) = spec.split_once('-').ok_or(RangeError::Malformed)?;
    if start.is_empty() {
        return Err(RangeError::Unsupported);
    }
    let start_val: u64 = start.parse().map_err(|_| RangeError::Malformed)?;
    if end.is_empty() {
        return Ok(ByteRange {
            start: start_val,
            end_inclusive: u64::MAX,
        });
    }
    let end_val: u64 = end.parse().map_err(|_| RangeError::Malformed)?;
    if end_val < start_val {
        return Err(RangeError::Malformed);
    }
    Ok(ByteRange {
        start: start_val,
        end_inclusive: end_val,
    })
}
