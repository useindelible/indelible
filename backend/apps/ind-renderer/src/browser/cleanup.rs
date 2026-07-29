use std::sync::Arc;

use anyhow::Result;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::browser::BrowserContextId;
use tokio::sync::OwnedSemaphorePermit;

use super::{BrowserManager, CAPTURE_CLEANUP_TIMEOUT};

pub(super) async fn close_resources(
    manager: &BrowserManager,
    page: &mut Option<Page>,
    context_id: &mut Option<BrowserContextId>,
) -> Result<()> {
    let page_result = match page.take() {
        Some(page) => page
            .close()
            .await
            .map_err(|error| anyhow::anyhow!("page close failed: {error}")),
        None => Ok(()),
    };
    let context_result = match context_id.clone() {
        Some(id) => {
            let result = manager.dispose_context(id).await;
            if result.is_ok() {
                context_id.take();
            }
            result
        }
        None => Ok(()),
    };
    page_result.and(context_result)
}

pub(super) struct DetachedPageCleanup {
    manager: Arc<BrowserManager>,
    page: Option<Page>,
    context_id: Option<BrowserContextId>,
    permit: Option<OwnedSemaphorePermit>,
}

impl DetachedPageCleanup {
    pub(super) fn new(
        manager: Arc<BrowserManager>,
        page: Option<Page>,
        context_id: Option<BrowserContextId>,
        permit: Option<OwnedSemaphorePermit>,
    ) -> Self {
        Self {
            manager,
            page,
            context_id,
            permit,
        }
    }

    pub(super) async fn run(mut self) {
        let cleanup = tokio::time::timeout(
            CAPTURE_CLEANUP_TIMEOUT,
            close_resources(&self.manager, &mut self.page, &mut self.context_id),
        )
        .await;
        if !matches!(cleanup, Ok(Ok(()))) {
            self.manager.recycle_unhealthy("cancelled_capture").await;
        }
    }
}

impl Drop for DetachedPageCleanup {
    fn drop(&mut self) {
        self.manager.finish_capture();
        self.permit.take();
    }
}
