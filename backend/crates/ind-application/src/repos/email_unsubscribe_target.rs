use chrono::{DateTime, Utc};
use ind_domain::EmailSenderId;

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailUnsubscribeTarget {
    pub sender_id: EmailSenderId,
    pub one_click_post_url: Option<String>,
    pub mailto_addr: Option<String>,
    pub web_url: Option<String>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnsubscribeTargetUpsert {
    pub one_click_post_url: Option<String>,
    pub mailto_addr: Option<String>,
    pub web_url: Option<String>,
}

#[async_trait::async_trait]
pub trait EmailUnsubscribeTargetRepository: Send + Sync {
    async fn upsert(
        &self,
        sender_id: EmailSenderId,
        targets: UnsubscribeTargetUpsert,
    ) -> Result<EmailUnsubscribeTarget, AppError>;

    async fn find_by_sender(
        &self,
        sender_id: EmailSenderId,
    ) -> Result<Option<EmailUnsubscribeTarget>, AppError>;
}
