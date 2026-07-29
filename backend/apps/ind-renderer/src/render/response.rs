use crate::types::{ArtifactResponse, AssetError, RenderResponse};

const UNSUPPORTED_ASSET_ERROR_PREFIX: &str = "unsupported(";

pub(super) fn finish_render_response(
    artifacts: Vec<ArtifactResponse>,
    asset_errors: Vec<AssetError>,
    wall_time_ms: u64,
    final_url: Option<String>,
) -> Result<RenderResponse, String> {
    let only_unsupported_asset_errors = !asset_errors.is_empty()
        && asset_errors
            .iter()
            .all(|e| is_unsupported_asset_error(&e.error));

    if artifacts.is_empty() && !asset_errors.is_empty() && !only_unsupported_asset_errors {
        Err(asset_errors
            .iter()
            .map(|e| e.error.as_str())
            .collect::<Vec<_>>()
            .join("; "))
    } else {
        Ok(RenderResponse {
            artifacts,
            asset_errors,
            wall_time_ms,
            final_url,
        })
    }
}

pub(super) fn unsupported_asset_error(reason_code: &str, message: String) -> anyhow::Error {
    anyhow::anyhow!("{UNSUPPORTED_ASSET_ERROR_PREFIX}{reason_code}): {message}")
}

fn is_unsupported_asset_error(error: &str) -> bool {
    error.starts_with(UNSUPPORTED_ASSET_ERROR_PREFIX)
}
