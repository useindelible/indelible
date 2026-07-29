use std::time::Instant;

use crate::browser::PageGuard;
use crate::browser_identity::apply_live_browser_identity;
use crate::config::CaptureSettings;
use crate::storage::S3Storage;
use crate::types::{ArtifactResponse, AssetError, RenderResponse};

mod capture;
mod defuddle;
mod dom_cleanup;
mod error;
mod lead_image;
mod network;
mod response;
mod singlefile;

use capture::{
    DerivedCaptureResults, capture_and_upload_pdf, capture_and_upload_screenshot,
    capture_derived_from_monolith, set_viewport,
};
use defuddle::extract_defuddle;
use dom_cleanup::clean_live_document;
pub(crate) use error::{CaptureError, CaptureStage};
use network::block_network;
use response::finish_render_response;
use singlefile::run_singlefile;

pub async fn render_url(
    page_guard: &PageGuard,
    storage: &S3Storage,
    url: &str,
    user_id: &str,
    item_id: &str,
    outputs: &[String],
    capture_settings: &CaptureSettings,
) -> Result<RenderResponse, CaptureError> {
    let start = Instant::now();
    let page = page_guard.page();
    apply_live_browser_identity(page, capture_settings).await?;

    // O.3: log host only at INFO; full URLs (incl. query-string tokens) stay at DEBUG.
    let log_host = ::url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_string))
        .unwrap_or_else(|| "<unparseable>".to_string());

    tracing::info!(host = %log_host, "render_url: setting viewport");
    tracing::debug!(url = %url, "render_url: target url");
    set_viewport(page).await;

    tracing::info!(host = %log_host, "render_url: starting page.goto");
    if let Err(e) = page.goto(url).await {
        // Heavy ad/consent stacks (e.g. theverge.com) hold the load event past the CDP
        // request timeout while the document itself committed long ago. Extraction only
        // needs the DOM, so continue when navigation actually happened and fail only
        // when the page never left about:blank.
        let committed_url = page
            .url()
            .await
            .ok()
            .flatten()
            .filter(|u| u != "about:blank" && !u.is_empty());
        match committed_url {
            Some(u) => {
                tracing::warn!(error = %e, committed_url = %u, "render_url: page.goto did not settle; continuing with committed DOM");
            }
            None => return Err(CaptureError::cdp(CaptureStage::Navigation, e)),
        }
    }
    tracing::info!(elapsed_ms = %start.elapsed().as_millis(), "render_url: page.goto complete");

    let removed_overlays = clean_live_document(page).await?;
    tracing::info!(removed_overlays, "render_url: capture DOM cleanup complete");

    let final_url = page
        .url()
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| url.to_string());

    let mut artifacts = Vec::new();
    let mut errors = Vec::new();

    tracing::info!(elapsed_ms = %start.elapsed().as_millis(), "render_url: starting defuddle extraction");
    if outputs.contains(&"readable_html".to_string()) {
        let defuddle_result = match extract_defuddle(page, Some(&final_url)).await {
            Ok(result) => Some(result),
            Err(e) => {
                if e.requires_browser_recovery() {
                    return Err(e);
                }
                tracing::error!(error = %e, "defuddle extraction failed");
                errors.push(format!("readable_html extraction: {e}"));
                None
            }
        };

        if let Some((html, metadata)) = defuddle_result {
            let key = format!("{user_id}/{item_id}/readable_html.html");
            match storage.upload(&key, "text/html", html.into_bytes()).await {
                Ok(size) => artifacts.push(ArtifactResponse {
                    kind: "readable_html".into(),
                    s3_key: key,
                    content_type: "text/html".into(),
                    size_bytes: size,
                    metadata: Some(metadata),
                }),
                Err(e) => {
                    tracing::error!(error = %e, "failed to upload readable_html");
                    errors.push(format!("readable_html upload: {e}"));
                }
            }
        }
    }

    tracing::info!(elapsed_ms = %start.elapsed().as_millis(), "render_url: starting singlefile");
    let needs_monolith = outputs.contains(&"monolith".to_string());
    let needs_derived =
        outputs.contains(&"pdf".to_string()) || outputs.contains(&"screenshot".to_string());

    let monolith_html = if needs_monolith || needs_derived {
        match run_singlefile(page).await {
            Ok(html) => {
                tracing::info!(elapsed_ms = %start.elapsed().as_millis(), size_bytes = %html.len(), "render_url: singlefile complete");
                Some(html)
            }
            Err(e) => {
                if e.requires_browser_recovery() {
                    return Err(e);
                }
                tracing::error!(error = %e, "singlefile failed");
                if needs_monolith {
                    errors.push(format!("monolith extraction: {e}"));
                }
                if needs_derived {
                    errors.push(format!(
                        "pdf/screenshot skipped: monolith unavailable ({e})"
                    ));
                }
                None
            }
        }
    } else {
        None
    };

    tracing::info!(elapsed_ms = %start.elapsed().as_millis(), "render_url: starting derived capture (pdf/screenshot)");
    if needs_derived && let Some(ref html) = monolith_html {
        let derived =
            capture_derived_from_monolith(page, storage, html, user_id, item_id, outputs).await;
        merge_derived_capture(derived, outputs, &mut artifacts, &mut errors)?;
    }

    if needs_monolith && let Some(html) = monolith_html {
        let key = format!("{user_id}/{item_id}/monolith.html");
        match storage.upload(&key, "text/html", html.into_bytes()).await {
            Ok(size) => artifacts.push(ArtifactResponse {
                kind: "monolith".into(),
                s3_key: key,
                content_type: "text/html".into(),
                size_bytes: size,
                metadata: None,
            }),
            Err(e) => {
                tracing::error!(error = %e, "failed to upload monolith");
                errors.push(format!("monolith upload: {e}"));
            }
        }
    }

    finish_render_response(
        artifacts,
        asset_errors(errors),
        start.elapsed().as_millis() as u64,
        Some(final_url),
    )
    .map_err(|error| CaptureError::other(CaptureStage::Response, anyhow::anyhow!(error)))
}

fn merge_derived_capture(
    derived: Result<DerivedCaptureResults, CaptureError>,
    outputs: &[String],
    artifacts: &mut Vec<ArtifactResponse>,
    errors: &mut Vec<String>,
) -> Result<(), CaptureError> {
    match derived {
        Ok(results) => {
            for (kind, artifact) in results {
                match artifact {
                    Ok(artifact) => artifacts.push(artifact),
                    Err(error) if error.requires_browser_recovery() => return Err(error),
                    Err(error) => errors.push(format!("{kind}: {error}")),
                }
            }
        }
        Err(error) if error.requires_browser_recovery() => return Err(error),
        Err(error) => {
            let message = error.to_string();
            for kind in ["screenshot", "pdf"] {
                if outputs.iter().any(|output| output == kind) {
                    errors.push(format!("{kind}: {message}"));
                }
            }
        }
    }
    Ok(())
}

pub async fn render_monolith(
    page_guard: &PageGuard,
    storage: &S3Storage,
    monolith_s3_key: &str,
    user_id: &str,
    item_id: &str,
    outputs: &[String],
) -> Result<RenderResponse, CaptureError> {
    let start = Instant::now();
    let page = page_guard.page();

    tracing::info!(s3_key = %monolith_s3_key, "render_monolith: downloading monolith");
    let monolith_bytes = match storage.download(monolith_s3_key).await {
        Ok(b) => b,
        Err(e) => {
            return Err(CaptureError::other(
                CaptureStage::Response,
                anyhow::anyhow!("failed to download monolith: {e}"),
            ));
        }
    };
    tracing::info!(elapsed_ms = %start.elapsed().as_millis(), size_bytes = %monolith_bytes.len(), "render_monolith: monolith downloaded, writing temp file");

    let temp_dir = std::env::temp_dir().join("ind-renderer");
    tokio::fs::create_dir_all(&temp_dir).await.ok();
    let temp_file = temp_dir.join(format!("{}.html", uuid::Uuid::now_v7()));
    if let Err(e) = tokio::fs::write(&temp_file, &monolith_bytes).await {
        return Err(CaptureError::other(
            CaptureStage::Response,
            anyhow::anyhow!("failed to write temp file: {e}"),
        ));
    }
    drop(monolith_bytes);

    let file_url = format!("file://{}", temp_file.display());

    set_viewport(page).await;

    tracing::info!(elapsed_ms = %start.elapsed().as_millis(), file_url = %file_url, "render_monolith: starting page.goto");
    if let Err(e) = page.goto(&file_url).await {
        tokio::fs::remove_file(&temp_file).await.ok();
        return Err(CaptureError::cdp(CaptureStage::Navigation, e));
    }
    tracing::info!(elapsed_ms = %start.elapsed().as_millis(), "render_monolith: page.goto complete, blocking network");

    if let Err(error) = block_network(page).await {
        tokio::fs::remove_file(&temp_file).await.ok();
        return Err(error);
    }
    tracing::info!(elapsed_ms = %start.elapsed().as_millis(), "render_monolith: network blocked");

    let removed_overlays = match clean_live_document(page).await {
        Ok(removed) => removed,
        Err(error) => {
            tokio::fs::remove_file(&temp_file).await.ok();
            return Err(error);
        }
    };
    tracing::info!(
        removed_overlays,
        "render_monolith: capture DOM cleanup complete"
    );

    let mut artifacts = Vec::new();
    let mut errors = Vec::new();

    if outputs.contains(&"readable_html".to_string()) {
        match extract_defuddle(page, None).await {
            Ok((html, metadata)) => {
                let key = format!("{user_id}/{item_id}/readable_html.html");
                match storage.upload(&key, "text/html", html.into_bytes()).await {
                    Ok(size) => artifacts.push(ArtifactResponse {
                        kind: "readable_html".into(),
                        s3_key: key,
                        content_type: "text/html".into(),
                        size_bytes: size,
                        metadata: Some(metadata),
                    }),
                    Err(e) => {
                        tracing::error!(error = %e, "failed to upload readable_html from monolith");
                        errors.push(format!("readable_html upload: {e}"));
                    }
                }
            }
            Err(e) => {
                if e.requires_browser_recovery() {
                    tokio::fs::remove_file(&temp_file).await.ok();
                    return Err(e);
                }
                tracing::error!(error = %e, "defuddle on monolith failed");
                errors.push(format!("readable_html extraction: {e}"));
            }
        }
    }

    if outputs.contains(&"screenshot".to_string()) {
        tracing::info!(elapsed_ms = %start.elapsed().as_millis(), "render_monolith: starting screenshot");
        match capture_and_upload_screenshot(page, storage, user_id, item_id).await {
            Ok(a) => {
                tracing::info!(elapsed_ms = %start.elapsed().as_millis(), size_bytes = %a.size_bytes, "render_monolith: screenshot complete");
                artifacts.push(a);
            }
            Err(error) => {
                if error.requires_browser_recovery() {
                    tokio::fs::remove_file(&temp_file).await.ok();
                    return Err(error);
                }
                tracing::error!(error = %error, elapsed_ms = %start.elapsed().as_millis(), "screenshot failed");
                errors.push(format!("screenshot: {error}"));
            }
        }
    }

    if outputs.contains(&"pdf".to_string()) {
        tracing::info!(elapsed_ms = %start.elapsed().as_millis(), "render_monolith: starting pdf");
        match capture_and_upload_pdf(page, storage, user_id, item_id).await {
            Ok(a) => {
                tracing::info!(elapsed_ms = %start.elapsed().as_millis(), size_bytes = %a.size_bytes, "render_monolith: pdf complete");
                artifacts.push(a);
            }
            Err(error) => {
                if error.requires_browser_recovery() {
                    tokio::fs::remove_file(&temp_file).await.ok();
                    return Err(error);
                }
                tracing::error!(error = %error, elapsed_ms = %start.elapsed().as_millis(), "pdf failed");
                errors.push(format!("pdf: {error}"));
            }
        }
    }

    tokio::fs::remove_file(&temp_file).await.ok();

    finish_render_response(
        artifacts,
        asset_errors(errors),
        start.elapsed().as_millis() as u64,
        None,
    )
    .map_err(|error| CaptureError::other(CaptureStage::Response, anyhow::anyhow!(error)))
}

fn asset_errors(errors: Vec<String>) -> Vec<AssetError> {
    errors
        .into_iter()
        .map(|msg| {
            let kind = msg
                .split(':')
                .next()
                .unwrap_or("unknown")
                .trim()
                .to_string();
            AssetError {
                kind,
                error: msg.clone(),
            }
        })
        .collect()
}
