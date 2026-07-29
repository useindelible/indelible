use std::time::{Duration, Instant};

use chromiumoxide::cdp::browser_protocol::{
    emulation::SetDeviceMetricsOverrideParams,
    page::{CaptureScreenshotFormat, CaptureScreenshotParams, PrintToPdfParams},
};

use crate::storage::S3Storage;
use crate::types::ArtifactResponse;

use super::dom_cleanup::clean_live_document;
use super::network::block_network;
use super::response::unsupported_asset_error;
use super::{CaptureError, CaptureStage};

pub(super) const CHROMIUM_MAX_FULL_PAGE_SCREENSHOT_DIMENSION_PX: u64 = 128 * 1024;
pub(super) type DerivedCaptureResults = Vec<(&'static str, Result<ArtifactResponse, CaptureError>)>;

pub(super) async fn capture_derived_from_monolith(
    page: &chromiumoxide::Page,
    storage: &S3Storage,
    monolith_html: &str,
    user_id: &str,
    item_id: &str,
    outputs: &[String],
) -> Result<DerivedCaptureResults, CaptureError> {
    let mut results = Vec::new();

    let temp_dir = std::env::temp_dir().join("ind-renderer");
    tokio::fs::create_dir_all(&temp_dir).await.ok();
    let temp_file = temp_dir.join(format!("{}.html", uuid::Uuid::now_v7()));
    if let Err(e) = tokio::fs::write(&temp_file, monolith_html.as_bytes()).await {
        return Err(CaptureError::other(
            CaptureStage::Response,
            anyhow::anyhow!("failed to write monolith temp file: {e}"),
        ));
    }

    let file_url = format!("file://{}", temp_file.display());

    let derived_start = Instant::now();
    tracing::info!(file_url = %file_url, "capture_derived: starting page.goto");
    if let Err(e) = page.goto(&file_url).await {
        tokio::fs::remove_file(&temp_file).await.ok();
        return Err(CaptureError::cdp(CaptureStage::Navigation, e));
    }
    tracing::info!(elapsed_ms = %derived_start.elapsed().as_millis(), "capture_derived: page.goto complete, blocking network");

    if let Err(error) = block_network(page).await {
        tokio::fs::remove_file(&temp_file).await.ok();
        return Err(error);
    }
    tracing::info!(elapsed_ms = %derived_start.elapsed().as_millis(), "capture_derived: network blocked");

    if let Err(error) = clean_live_document(page).await {
        tokio::fs::remove_file(&temp_file).await.ok();
        return Err(error);
    }

    if outputs.contains(&"screenshot".to_string()) {
        tracing::info!(elapsed_ms = %derived_start.elapsed().as_millis(), "capture_derived: starting screenshot");
        let result = capture_and_upload_screenshot(page, storage, user_id, item_id).await;
        tracing::info!(elapsed_ms = %derived_start.elapsed().as_millis(), ok = %result.is_ok(), "capture_derived: screenshot done");
        results.push(("screenshot", result));
    }

    if outputs.contains(&"pdf".to_string()) {
        tracing::info!(elapsed_ms = %derived_start.elapsed().as_millis(), "capture_derived: starting pdf");
        let result = capture_and_upload_pdf(page, storage, user_id, item_id).await;
        tracing::info!(elapsed_ms = %derived_start.elapsed().as_millis(), ok = %result.is_ok(), "capture_derived: pdf done");
        results.push(("pdf", result));
    }

    tokio::fs::remove_file(&temp_file).await.ok();
    Ok(results)
}

pub(super) async fn capture_and_upload_screenshot(
    page: &chromiumoxide::Page,
    storage: &S3Storage,
    user_id: &str,
    item_id: &str,
) -> Result<ArtifactResponse, CaptureError> {
    let bytes = capture_screenshot(page).await?;
    let key = format!("{user_id}/{item_id}/screenshot.jpg");
    let size = storage
        .upload(&key, "image/jpeg", bytes)
        .await
        .map_err(|error| {
            CaptureError::other(
                CaptureStage::Screenshot,
                anyhow::anyhow!("failed to upload screenshot: {error}"),
            )
        })?;
    Ok(ArtifactResponse {
        kind: "screenshot".into(),
        s3_key: key,
        content_type: "image/jpeg".into(),
        size_bytes: size,
        metadata: None,
    })
}

pub(super) async fn capture_and_upload_pdf(
    page: &chromiumoxide::Page,
    storage: &S3Storage,
    user_id: &str,
    item_id: &str,
) -> Result<ArtifactResponse, CaptureError> {
    let bytes = capture_pdf(page).await?;
    let key = format!("{user_id}/{item_id}/pdf.pdf");
    let size = storage
        .upload(&key, "application/pdf", bytes)
        .await
        .map_err(|error| {
            CaptureError::other(
                CaptureStage::Pdf,
                anyhow::anyhow!("failed to upload pdf: {error}"),
            )
        })?;
    Ok(ArtifactResponse {
        kind: "pdf".into(),
        s3_key: key,
        content_type: "application/pdf".into(),
        size_bytes: size,
        metadata: None,
    })
}

pub(super) async fn set_viewport(page: &chromiumoxide::Page) {
    if let Ok(params) = SetDeviceMetricsOverrideParams::builder()
        .width(1280u32)
        .height(900u32)
        .device_scale_factor(1.0f64)
        .mobile(false)
        .build()
    {
        page.execute(params).await.ok();
    }
}

async fn capture_screenshot(page: &chromiumoxide::Page) -> Result<Vec<u8>, CaptureError> {
    let capture_beyond_viewport = cfg!(target_os = "linux");

    if capture_beyond_viewport {
        ensure_full_page_screenshot_supported(page).await?;
    }

    let fut = page.screenshot(
        CaptureScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Jpeg)
            .quality(80i64)
            .capture_beyond_viewport(capture_beyond_viewport)
            .from_surface(true)
            .build(),
    );
    let bytes = tokio::time::timeout(Duration::from_secs(30), fut)
        .await
        .map_err(|_| CaptureError::Timeout {
            stage: CaptureStage::Screenshot.label(),
        })?
        .map_err(|error| CaptureError::cdp(CaptureStage::Screenshot, error))?;
    Ok(bytes)
}

async fn capture_pdf(page: &chromiumoxide::Page) -> Result<Vec<u8>, CaptureError> {
    let bytes = page
        .pdf(
            PrintToPdfParams::builder()
                .print_background(true)
                .prefer_css_page_size(true)
                .build(),
        )
        .await
        .map_err(|error| CaptureError::cdp(CaptureStage::Pdf, error))?;
    Ok(bytes)
}

async fn ensure_full_page_screenshot_supported(
    page: &chromiumoxide::Page,
) -> Result<(), CaptureError> {
    let metrics = page
        .layout_metrics()
        .await
        .map_err(|error| CaptureError::cdp(CaptureStage::Screenshot, error))?;

    if let Some(message) = full_page_screenshot_limit_error(
        metrics.css_content_size.width,
        metrics.css_content_size.height,
    ) {
        return Err(CaptureError::other(
            CaptureStage::Screenshot,
            unsupported_asset_error("page_too_large", message),
        ));
    }

    Ok(())
}

pub(super) fn full_page_screenshot_limit_error(width_px: f64, height_px: f64) -> Option<String> {
    let width_px = width_px.ceil().max(0.0) as u64;
    let height_px = height_px.ceil().max(0.0) as u64;

    if width_px >= CHROMIUM_MAX_FULL_PAGE_SCREENSHOT_DIMENSION_PX
        || height_px >= CHROMIUM_MAX_FULL_PAGE_SCREENSHOT_DIMENSION_PX
    {
        Some(format!(
            "full-page screenshot dimensions {}x{}px exceed Chromium's {}px limit",
            width_px, height_px, CHROMIUM_MAX_FULL_PAGE_SCREENSHOT_DIMENSION_PX
        ))
    } else {
        None
    }
}
