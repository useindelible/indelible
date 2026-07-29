use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use anyhow::Result;
use chromiumoxide::cdp::browser_protocol::browser::BrowserContextId;
use chromiumoxide::cdp::browser_protocol::target::{
    CreateBrowserContextParams, CreateTargetParams,
};
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

mod cleanup;

use cleanup::DetachedPageCleanup;

const BROWSER_CLOSE_TIMEOUT: Duration = Duration::from_secs(2);
const BROWSER_EXIT_TIMEOUT: Duration = Duration::from_secs(5);
const CAPTURE_CLEANUP_TIMEOUT: Duration = Duration::from_secs(2);

struct BrowserHandle {
    browser: Browser,
    handler_task: tokio::task::JoinHandle<()>,
    created_at: Instant,
    profile_dir: PathBuf,
}

pub struct BrowserManager {
    inner: Arc<Mutex<Option<BrowserHandle>>>,
    capture_permits: Arc<Semaphore>,
    active_renders: Arc<AtomicUsize>,
    last_used_epoch_ms: Arc<AtomicI64>,
    chromium_path: PathBuf,
    single_process: bool,
    virtual_time_budget: Option<u32>,
    idle_timeout_secs: u64,
}

impl BrowserManager {
    pub fn new(
        chromium_path: PathBuf,
        single_process: bool,
        virtual_time_budget: Option<u32>,
        idle_timeout_secs: u64,
        max_concurrent_pages: usize,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
            capture_permits: Arc::new(Semaphore::new(max_concurrent_pages)),
            active_renders: Arc::new(AtomicUsize::new(0)),
            last_used_epoch_ms: Arc::new(AtomicI64::new(0)),
            chromium_path,
            single_process,
            virtual_time_budget,
            idle_timeout_secs,
        }
    }

    pub fn is_browser_running(&self) -> bool {
        self.active_renders.load(Ordering::Relaxed) > 0
            || self.last_used_epoch_ms.load(Ordering::Relaxed) > 0
    }

    pub async fn acquire_page(self: &Arc<Self>) -> Result<PageGuard> {
        let permit = self
            .capture_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| anyhow::anyhow!("capture concurrency limiter closed"))?;
        let mut guard = self.inner.lock().await;

        if guard.is_none() {
            let handle = launch_browser(
                &self.chromium_path,
                self.single_process,
                self.virtual_time_budget,
            )
            .await?;
            *guard = Some(handle);
            tracing::info!("browser launched");
        }

        #[expect(
            clippy::unwrap_used,
            reason = "the block above initializes the browser when the slot is empty"
        )]
        let handle = guard.as_mut().unwrap();
        let context_id = handle
            .browser
            .create_browser_context(
                CreateBrowserContextParams::builder()
                    .dispose_on_detach(true)
                    .build(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("create browser context failed: {e}"))?;
        let target = CreateTargetParams::builder()
            .url("about:blank")
            .browser_context_id(context_id.clone())
            .build()
            .map_err(|e| anyhow::anyhow!("create target params failed: {e}"))?;
        let page = match handle.browser.new_page(target).await {
            Ok(page) => page,
            Err(error) => {
                let _ = handle
                    .browser
                    .dispose_browser_context(context_id.clone())
                    .await;
                return Err(anyhow::anyhow!("new_page failed: {error}"));
            }
        };

        self.active_renders.fetch_add(1, Ordering::Relaxed);
        self.update_last_used();

        Ok(PageGuard {
            page: Some(page),
            context_id: Some(context_id),
            manager: Arc::clone(self),
            permit: Some(permit),
            active: true,
        })
    }

    fn update_last_used(&self) {
        self.last_used_epoch_ms
            .store(chrono::Utc::now().timestamp_millis(), Ordering::Relaxed);
    }

    fn finish_capture(&self) {
        self.active_renders.fetch_sub(1, Ordering::Relaxed);
        self.update_last_used();
    }

    pub fn spawn_idle_watchdog(self: &Arc<Self>) {
        let this = Arc::clone(self);
        tokio::spawn(async move {
            let watchdog_interval = Duration::from_secs(this.idle_timeout_secs.clamp(1, 10));
            loop {
                tokio::time::sleep(watchdog_interval).await;
                if this.active_renders.load(Ordering::Relaxed) > 0 {
                    continue;
                }

                let last_ms = this.last_used_epoch_ms.load(Ordering::Relaxed);
                if last_ms == 0 {
                    continue;
                }

                let idle_ms = (chrono::Utc::now().timestamp_millis() - last_ms).max(0) as u64;
                if idle_ms < this.idle_timeout_secs * 1000 {
                    continue;
                }

                let mut guard = this.inner.lock().await;
                if this.active_renders.load(Ordering::Relaxed) > 0 {
                    continue;
                }

                if let Some(handle) = guard.take() {
                    let uptime_secs = handle.created_at.elapsed().as_secs();
                    shutdown_browser(handle).await;
                    this.last_used_epoch_ms.store(0, Ordering::Relaxed);
                    tracing::info!(
                        idle_secs = idle_ms / 1000,
                        uptime_secs,
                        "browser idle shutdown"
                    );
                }
            }
        });
    }

    async fn dispose_context(&self, context_id: BrowserContextId) -> Result<()> {
        let guard = self.inner.lock().await;
        let handle = guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("browser stopped before context disposal"))?;
        handle
            .browser
            .dispose_browser_context(context_id)
            .await
            .map_err(|e| anyhow::anyhow!("dispose browser context failed: {e}"))
    }

    pub async fn recycle_unhealthy(&self, reason: &'static str) {
        let mut guard = self.inner.lock().await;
        if let Some(handle) = guard.take() {
            tracing::warn!(reason, "recycling unhealthy browser");
            shutdown_browser(handle).await;
        }
        self.last_used_epoch_ms.store(0, Ordering::Relaxed);
    }

    pub async fn shutdown(&self) {
        let mut guard = self.inner.lock().await;
        if let Some(handle) = guard.take() {
            shutdown_browser(handle).await;
        }
        self.last_used_epoch_ms.store(0, Ordering::Relaxed);
    }
}

pub struct PageGuard {
    page: Option<Page>,
    context_id: Option<BrowserContextId>,
    manager: Arc<BrowserManager>,
    permit: Option<OwnedSemaphorePermit>,
    active: bool,
}

impl PageGuard {
    #[expect(
        clippy::expect_used,
        reason = "render code accesses the page only before close takes it"
    )]
    pub fn page(&self) -> &Page {
        self.page.as_ref().expect("page already closed")
    }

    pub async fn close(&mut self) -> Result<()> {
        let result = self.close_resources().await;
        if result.is_err() && self.context_id.is_some() {
            self.manager.recycle_unhealthy("capture_cleanup").await;
        }
        self.finish();
        result
    }

    pub async fn close_after_capture_failure(
        &mut self,
        stage: &'static str,
        browser_is_unhealthy: bool,
    ) {
        let cleanup = tokio::time::timeout(CAPTURE_CLEANUP_TIMEOUT, self.close_resources()).await;
        let should_recycle = browser_is_unhealthy || !matches!(cleanup, Ok(Ok(())));
        if should_recycle {
            self.manager.recycle_unhealthy(stage).await;
        }
        self.finish();
    }

    async fn close_resources(&mut self) -> Result<()> {
        cleanup::close_resources(&self.manager, &mut self.page, &mut self.context_id).await
    }

    fn finish(&mut self) {
        if self.active {
            self.manager.finish_capture();
            self.active = false;
        }
        self.permit.take();
    }
}

impl Drop for PageGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            tracing::warn!(
                "capture guard dropped outside a Tokio runtime; browser cleanup deferred to shutdown"
            );
            self.finish();
            return;
        };

        self.active = false;
        let cleanup = DetachedPageCleanup::new(
            Arc::clone(&self.manager),
            self.page.take(),
            self.context_id.take(),
            self.permit.take(),
        );
        runtime.spawn(cleanup.run());
    }
}

fn build_flags(single_process: bool, virtual_time_budget: Option<u32>) -> Vec<String> {
    let mut flags: Vec<String> = vec![
        "disable-dev-shm-usage",
        "disable-gpu",
        "disable-gpu-compositing",
        "disable-software-rasterizer",
        "disable-extensions",
        "disable-plugins",
        "disable-background-networking",
        "disable-default-apps",
        "disable-sync",
        "disable-translate",
        "disable-client-side-phishing-detection",
        "enable-features=NetworkService,NetworkServiceInProcess",
        "disable-breakpad",
        "disable-component-extensions-with-background-pages",
        "disable-features=TranslateUI",
        "disable-hang-monitor",
        "disable-ipc-flooding-protection",
        "disable-popup-blocking",
        "disable-prompt-on-repost",
        "force-color-profile=srgb",
        "password-store=basic",
        "use-mock-keychain",
        "enable-blink-features=IdleDetection",
        "lang=en_US",
        "disable-blink-features=AutomationControlled",
        "disable-background-timer-throttling",
        "disable-renderer-backgrounding",
        "disable-backgrounding-occluded-windows",
        "no-first-run",
        "mute-audio",
        "metrics-recording-only",
        "disk-cache-size=1",
        "media-cache-size=1",
        "disable-application-cache",
        "renderer-process-limit=2",
        "run-all-compositor-stages-before-draw",
        "aggressive-cache-discard",
        "process-per-site",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    flags.push("js-flags=--initial-old-space-size=64 --max-old-space-size=512".into());

    #[cfg(target_os = "linux")]
    flags.push("no-zygote".into());

    if single_process {
        flags.push("single-process".into());
    }

    if let Some(budget) = virtual_time_budget {
        flags.push(format!("virtual-time-budget={budget}"));
    }

    flags
}

async fn launch_browser(
    bin: &Path,
    single_process: bool,
    virtual_time_budget: Option<u32>,
) -> Result<BrowserHandle> {
    let flags = build_flags(single_process, virtual_time_budget);
    let profile_dir =
        std::env::temp_dir().join(format!("ind-renderer-profile-{}", uuid::Uuid::now_v7()));
    let config = BrowserConfig::builder()
        .chrome_executable(bin)
        .user_data_dir(profile_dir.clone())
        .disable_default_args()
        .window_size(1280, 900)
        .no_sandbox()
        .new_headless_mode()
        .args(flags)
        .build()
        .map_err(|e| anyhow::anyhow!("BrowserConfig: {e}"))?;

    let (browser, mut handler) = Browser::launch(config).await?;
    let handler_task = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if let Err(ref e) = h {
                let msg = e.to_string();
                if msg.contains("did not match any variant") || msg.contains("unknown variant") {
                    continue;
                }
                tracing::error!(error = %e, "cdp handler error");
                break;
            }
        }
        tracing::warn!("cdp handler exited");
    });

    Ok(BrowserHandle {
        browser,
        handler_task,
        created_at: Instant::now(),
        profile_dir,
    })
}

async fn shutdown_browser(mut handle: BrowserHandle) {
    let _ = tokio::time::timeout(BROWSER_CLOSE_TIMEOUT, handle.browser.close()).await;
    let exited = matches!(
        tokio::time::timeout(BROWSER_EXIT_TIMEOUT, handle.browser.wait()).await,
        Ok(Ok(_))
    );
    if !exited && let Some(result) = handle.browser.kill().await {
        result.unwrap_or_else(|error| tracing::warn!(%error, "failed to kill browser process"));
    }

    handle.handler_task.abort();
    let _ = handle.handler_task.await;
    if let Err(error) = tokio::fs::remove_dir_all(&handle.profile_dir).await
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(%error, profile = %handle.profile_dir.display(), "failed to remove browser profile");
    }

    // The process uses mimalloc globally; forcing collection after all capture buffers and
    // Chromium handles are gone releases otherwise-idle segments back to the OS.
    force_mimalloc_collection();
}

#[allow(
    unsafe_code,
    reason = "mimalloc exposes forced collection only through its official C FFI; the call takes no pointers and is valid while mimalloc is the process global allocator"
)]
fn force_mimalloc_collection() {
    unsafe { libmimalloc_sys::mi_collect(true) };
}
