use chromiumoxide::error::CdpError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CdpErrorCategory {
    Timeout,
    NoResponse,
    Channel,
    Other,
}

impl CdpErrorCategory {
    fn from_error(error: &CdpError) -> Self {
        match error {
            CdpError::Timeout => Self::Timeout,
            CdpError::NoResponse => Self::NoResponse,
            CdpError::ChannelSendError(_) => Self::Channel,
            _ => Self::Other,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CaptureStage {
    Navigation,
    Identity,
    DomCleanup,
    NetworkBlock,
    Defuddle,
    SingleFile,
    Screenshot,
    Pdf,
    Response,
}

impl CaptureStage {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Navigation => "navigation",
            Self::Identity => "browser_identity",
            Self::DomCleanup => "dom_cleanup",
            Self::NetworkBlock => "network_block",
            Self::Defuddle => "defuddle",
            Self::SingleFile => "singlefile",
            Self::Screenshot => "screenshot",
            Self::Pdf => "pdf",
            Self::Response => "response",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum CaptureError {
    #[error("{stage} timed out")]
    Timeout { stage: &'static str },
    #[error("capture exceeded the {budget_secs}s deadline")]
    DeadlineExceeded { budget_secs: u64 },
    #[error("{}: {source}", stage.label())]
    Cdp {
        stage: CaptureStage,
        category: CdpErrorCategory,
        #[source]
        source: CdpError,
    },
    #[error("{}: {source}", stage.label())]
    Other {
        stage: CaptureStage,
        #[source]
        source: anyhow::Error,
    },
}

impl CaptureError {
    pub(crate) fn cdp(stage: CaptureStage, source: CdpError) -> Self {
        Self::Cdp {
            stage,
            category: CdpErrorCategory::from_error(&source),
            source,
        }
    }

    pub(crate) fn other(stage: CaptureStage, source: impl Into<anyhow::Error>) -> Self {
        Self::Other {
            stage,
            source: source.into(),
        }
    }

    pub(crate) fn is_timeout(&self) -> bool {
        matches!(
            self,
            Self::Timeout { .. }
                | Self::DeadlineExceeded { .. }
                | Self::Cdp {
                    category: CdpErrorCategory::Timeout,
                    ..
                }
        )
    }

    /// A capture that outlived its whole-request budget may be waiting on a command Chromium will
    /// never answer (a tab whose renderer process died, for instance); only a relaunch guarantees
    /// the next capture a working browser.
    pub(crate) fn browser_is_unhealthy(&self) -> bool {
        matches!(
            self,
            Self::DeadlineExceeded { .. }
                | Self::Cdp {
                    category: CdpErrorCategory::NoResponse | CdpErrorCategory::Channel,
                    ..
                }
        )
    }

    pub(crate) fn requires_browser_recovery(&self) -> bool {
        self.is_timeout() || self.browser_is_unhealthy()
    }

    pub(crate) fn stage_label(&self) -> &'static str {
        match self {
            Self::Timeout { stage } => stage,
            Self::DeadlineExceeded { .. } => "capture_deadline",
            Self::Cdp { stage, .. } | Self::Other { stage, .. } => stage.label(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdp_failures_keep_stage_and_recovery_category() {
        let timeout = CaptureError::cdp(CaptureStage::SingleFile, CdpError::Timeout);
        assert!(timeout.is_timeout());
        assert_eq!(timeout.stage_label(), "singlefile");

        let no_response = CaptureError::cdp(CaptureStage::Pdf, CdpError::NoResponse);
        assert!(no_response.browser_is_unhealthy());
        assert_eq!(no_response.stage_label(), "pdf");

        let deadline = CaptureError::DeadlineExceeded { budget_secs: 120 };
        assert!(deadline.is_timeout());
        assert!(deadline.browser_is_unhealthy());
        assert_eq!(deadline.stage_label(), "capture_deadline");
        assert_eq!(deadline.to_string(), "capture exceeded the 120s deadline");
    }
}
