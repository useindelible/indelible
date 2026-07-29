use http::HeaderMap;

use super::types::{EmailIngestError, EmailIngestProvider, InboundEmail, WebhookMetadata};

#[async_trait::async_trait]
pub trait InboundEmailProvider: Send + Sync {
    fn provider(&self) -> EmailIngestProvider;

    fn verify_signature(&self, body: &[u8], headers: &HeaderMap) -> Result<(), EmailIngestError>;

    fn parse_webhook_metadata(&self, body: &[u8]) -> Result<WebhookMetadata, EmailIngestError>;

    async fn resolve_full_email(
        &self,
        metadata: &WebhookMetadata,
        raw_payload: &[u8],
    ) -> Result<InboundEmail, EmailIngestError>;
}
