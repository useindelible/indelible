use chromiumoxide::cdp::browser_protocol::network::{
    BlockPattern, EnableParams as NetworkEnableParams, SetBlockedUrLsParams,
};

use super::{CaptureError, CaptureStage};

pub(super) async fn block_network(page: &chromiumoxide::Page) -> Result<(), CaptureError> {
    page.execute(NetworkEnableParams::default())
        .await
        .map_err(|error| CaptureError::cdp(CaptureStage::NetworkBlock, error))?;

    page.execute(
        SetBlockedUrLsParams::builder()
            .url_pattern(BlockPattern::new("http://*", true))
            .url_pattern(BlockPattern::new("https://*", true))
            .build(),
    )
    .await
    .map_err(|error| CaptureError::cdp(CaptureStage::NetworkBlock, error))?;

    Ok(())
}
