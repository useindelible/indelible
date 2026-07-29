#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use http::HeaderMap;
use ind_application::repos::email_ingest::ClaimAndEnqueueInput;
use ind_domain::ops::EmailIngestJob;
use ind_domain::{GenericJobEnvelope, JobOutboxId};
use ind_integrations::email::{
    EmailIngestError, EmailIngestProvider, InboundEmail, InboundEmailProvider, WebhookMetadata,
};
use ind_test_support::{TestDb, UserFactory};
use ind_worker::context::EmailJobDeps;
use ind_worker::jobs::email_ingest::dispatch_generic_job;

use common::build_worker_ctx_with_email_services;

#[derive(Clone)]
struct FixedEmailProvider(InboundEmail);

type PersistedDocumentRow = (
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
);

#[async_trait::async_trait]
impl InboundEmailProvider for FixedEmailProvider {
    fn provider(&self) -> EmailIngestProvider {
        EmailIngestProvider::Resend
    }

    fn verify_signature(&self, _: &[u8], _: &HeaderMap) -> Result<(), EmailIngestError> {
        Ok(())
    }

    fn parse_webhook_metadata(&self, _: &[u8]) -> Result<WebhookMetadata, EmailIngestError> {
        Ok(WebhookMetadata {
            provider_email_id: self.0.provider_email_id.clone(),
            to_addresses: self.0.to_addresses.clone(),
            from_address: self.0.from_address.clone(),
            list_id: self.0.list_id.clone(),
        })
    }

    async fn resolve_full_email(
        &self,
        _: &WebhookMetadata,
        _: &[u8],
    ) -> Result<InboundEmail, EmailIngestError> {
        Ok(self.0.clone())
    }
}

fn email(id: &str, subject: &str, html: &str, text: &str) -> InboundEmail {
    InboundEmail {
        provider_email_id: id.into(),
        from_address: "Indelible Dispatch <dispatch@example.com>".into(),
        from_display_name: Some("Indelible Dispatch".into()),
        to_addresses: Vec::new(),
        subject: subject.into(),
        html_body: Some(html.into()),
        text_body: Some(text.into()),
        message_id: Some(format!("<{id}@example.com>")),
        list_id: None,
        list_unsubscribe: None,
        list_unsubscribe_post: None,
        headers: HashMap::new(),
        received_at: Utc::now(),
    }
}

async fn dispatch(
    deps: &mut EmailJobDeps,
    user_id: ind_domain::UserId,
    destination: &str,
    inbound: InboundEmail,
) {
    let provider_email_id = inbound.provider_email_id.clone();
    let from_address = inbound.from_address.clone();
    let logs = deps.email_ingest_log_repo.clone().unwrap();
    let log = logs
        .claim_and_enqueue(ClaimAndEnqueueInput {
            provider: "resend",
            provider_email_id: &provider_email_id,
            user_id,
            destination,
            job_type: "email.ingest",
            job_payload: serde_json::json!({}),
            raw_payload: Some(b"raw-email"),
            from_address: &from_address,
            list_id: None,
        })
        .await
        .unwrap()
        .unwrap();
    deps.email_ingest_provider = Some(Arc::new(FixedEmailProvider(inbound)));
    let handled = dispatch_generic_job(
        deps,
        GenericJobEnvelope {
            outbox_id: JobOutboxId::new(),
            job_type: "email.ingest".into(),
            payload: serde_json::to_value(EmailIngestJob {
                provider: "resend".into(),
                provider_email_id,
                raw_payload: b"raw-email".to_vec(),
                user_id,
                destination: destination.into(),
                ingest_log_id: Some(log.id),
            })
            .unwrap(),
            dedupe_key: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(handled, Some(()));
}

#[tokio::test]
async fn email_ingest_routes_feed_body_link_and_fallback_through_durable_boundaries() {
    let db = TestDb::new().await;
    let user = UserFactory::new().insert(db.pool()).await;
    let ctx = build_worker_ctx_with_email_services(&db).await;
    let mut deps = ctx.email_jobs();
    let long_body = "The researchers are running careful experiments across several systems. \
        Their observations explain how the components interact and why the results remain stable. \
        Readers can compare the evidence, understand the conclusions, and revisit the archived \
        material whenever they need the original context. "
        .repeat(8);

    dispatch(
        &mut deps,
        user.id,
        "feed",
        email(
            "feed-message",
            "Newsletter delivery",
            &format!("<article><h1>Newsletter</h1><p>{long_body}</p></article>"),
            &long_body,
        ),
    )
    .await;
    dispatch(
        &mut deps,
        user.id,
        "library",
        email(
            "body-message",
            "Archived email body",
            &format!("<article><h1>Archive</h1><p>{long_body}</p><script>bad()</script></article>"),
            &long_body,
        ),
    )
    .await;
    dispatch(
        &mut deps,
        user.id,
        "library",
        email(
            "link-message",
            "Pointer to an article",
            "<p><a href='https://example.com/deep-story?utm_source=email'>Read it</a></p>",
            "Read it",
        ),
    )
    .await;
    dispatch(
        &mut deps,
        user.id,
        "library",
        email(
            "fallback-message",
            "Pointer without a link",
            "<p>There is deliberately no link here.</p>",
            "There is deliberately no link here.",
        ),
    )
    .await;

    let feed_delivery: (String, String, Option<String>, bool) = sqlx::query_as(
        "SELECT fs.feed_type, fse.title, fse.language, fd.document_id IS NULL \
         FROM feed_deliveries fd \
         JOIN feed_subscriptions sub ON sub.id = fd.subscription_id \
         JOIN feed_sources fs ON fs.id = sub.source_id \
         JOIN feed_source_entries fse ON fse.id = fd.source_entry_id \
         WHERE sub.user_id = $1",
    )
    .bind(user.id.into_uuid())
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(
        feed_delivery,
        (
            "newsletter".into(),
            "Newsletter delivery".into(),
            Some("en".into()),
            true
        )
    );

    let documents: Vec<PersistedDocumentRow> = sqlx::query_as(
        "SELECT d.title, d.document_type, d.canonical_url, d.domain, d.language \
         FROM documents d JOIN library_entries le ON le.document_id = d.id \
         WHERE le.user_id = $1 AND le.deleted_at IS NULL ORDER BY d.title",
    )
    .bind(user.id.into_uuid())
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert_eq!(documents.len(), 3);
    assert!(documents.iter().any(|row| {
        row.0 == "Archived email body"
            && row.1 == "email"
            && row.2.is_none()
            && row.3.as_deref() == Some("example.com")
            && row.4.as_deref() == Some("en")
    }));
    assert!(documents.iter().any(|row| {
        row.0 == "Pointer to an article"
            && row.1 == "article"
            && row.2.as_deref() == Some("https://example.com/deep-story")
    }));

    let fallback_tag: Option<String> = sqlx::query_scalar(
        "SELECT t.name FROM tags t \
         JOIN library_entry_tags letag ON letag.tag_id = t.id \
         JOIN library_entries le ON le.id = letag.library_entry_id \
         JOIN documents d ON d.id = le.document_id \
         WHERE le.user_id = $1 AND d.title = 'Pointer without a link'",
    )
    .bind(user.id.into_uuid())
    .fetch_optional(db.pool())
    .await
    .unwrap();
    assert_eq!(fallback_tag.as_deref(), Some("no-link-found"));

    let outbox: Vec<(String, i64)> = sqlx::query_as(
        "SELECT job_type, count(*) FROM job_outbox \
         WHERE job_type IN ('document.attach_provided_content', 'search.reindex_document') \
         GROUP BY job_type ORDER BY job_type",
    )
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert_eq!(
        outbox,
        vec![
            ("document.attach_provided_content".into(), 4),
            ("search.reindex_document".into(), 1),
        ]
    );
    let processed_logs: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM email_ingest_log WHERE user_id = $1 AND status = 'processed'",
    )
    .bind(user.id.into_uuid())
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(processed_logs, 4);
    let sender_deliveries: i32 = sqlx::query_scalar(
        "SELECT delivery_count FROM email_senders WHERE user_id = $1 AND canonical_addr = 'dispatch@example.com'",
    )
    .bind(user.id.into_uuid())
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(sender_deliveries, 4);
}
