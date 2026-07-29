use chromiumoxide::Page;

use super::{CaptureError, CaptureStage};

const DOM_PREPROCESSOR_JS: &str = include_str!("../dom-preprocessor.js");

pub(super) async fn inject_dom_preprocessor(page: &Page) -> Result<(), CaptureError> {
    page.evaluate(DOM_PREPROCESSOR_JS)
        .await
        .map_err(|error| CaptureError::cdp(CaptureStage::DomCleanup, error))?;
    Ok(())
}

pub(super) async fn clean_live_document(page: &Page) -> Result<usize, CaptureError> {
    inject_dom_preprocessor(page).await?;
    let evaluation = page
        .evaluate(
            "globalThis.IndelibleDomPreprocessor.beginCaptureDomCleanup(document, 'permanent').removedElements",
        )
        .await
        .map_err(|error| CaptureError::cdp(CaptureStage::DomCleanup, error))?;
    evaluation.into_value::<usize>().map_err(|error| {
        CaptureError::other(
            CaptureStage::DomCleanup,
            anyhow::anyhow!("DOM cleanup result deserialize failed: {error}"),
        )
    })
}
