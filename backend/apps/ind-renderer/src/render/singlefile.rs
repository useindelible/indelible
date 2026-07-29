use super::{CaptureError, CaptureStage};

const SINGLEFILE_JS: &str = include_str!("../single-file.js");

pub(super) async fn run_singlefile(page: &chromiumoxide::Page) -> Result<String, CaptureError> {
    page.evaluate(SINGLEFILE_JS)
        .await
        .map_err(|e| CaptureError::cdp(CaptureStage::SingleFile, e))?;

    let capture_js = r#"
        (async () => {
            try {
                const sf = window.singlefile;
                if (!sf) throw new Error('SingleFile not loaded');
                const result = await sf.getPageData({
                    removeHiddenElements: true,
                    removeUnusedStyles: true,
                    removeUnusedFonts: true,
                    removeImports: true,
                    blockScripts: true,
                    blockAudios: true,
                    blockVideos: true,
                    compressHTML: false,
                    removeAlternativeFonts: true,
                    removeAlternativeMedias: true,
                    removeAlternativeImages: false,
                    groupDuplicateImages: true,
                });
                return result.content;
            } catch(e) { return { error: String(e) }; }
        })()
    "#;

    let eval = page
        .evaluate(capture_js)
        .await
        .map_err(|e| CaptureError::cdp(CaptureStage::SingleFile, e))?;

    let val: serde_json::Value = eval.into_value().map_err(|e| {
        CaptureError::other(
            CaptureStage::SingleFile,
            anyhow::anyhow!("singlefile result deserialize failed: {e}"),
        )
    })?;

    if let Some(obj) = val.as_object()
        && let Some(err) = obj.get("error")
    {
        return Err(CaptureError::other(
            CaptureStage::SingleFile,
            anyhow::anyhow!("singlefile error: {}", err.as_str().unwrap_or("unknown")),
        ));
    }

    val.as_str().map(|s| s.to_string()).ok_or_else(|| {
        CaptureError::other(
            CaptureStage::SingleFile,
            anyhow::anyhow!("singlefile returned non-string result"),
        )
    })
}
