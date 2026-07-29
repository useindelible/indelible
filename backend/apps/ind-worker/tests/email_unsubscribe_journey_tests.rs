#![allow(clippy::unwrap_used)]

use ind_application::repos::email_sender::EmailSenderRepository;
use ind_application::repos::email_unsubscribe_target::{
    EmailUnsubscribeTargetRepository, UnsubscribeTargetUpsert,
};
use ind_domain::ops::EmailUnsubscribeJob;
use ind_domain::{CanonicalAddress, GenericJobEnvelope, JobOutboxId};
use ind_persistence::repos::{PgEmailSenderRepository, PgEmailUnsubscribeTargetRepository};
use ind_test_support::{TestDb, UserFactory};
use ind_worker::jobs::email_unsubscribe::{
    OneClickPolicy, dispatch_generic_job, handle_email_unsubscribe, validate_one_click_url,
};
use wiremock::matchers::{body_string, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;

#[tokio::test]
async fn unsubscribe_handoff_posts_rfc_body_persists_fallbacks_and_enforces_url_policy() {
    for invalid in [
        "http://example.com/unsubscribe",
        "https://127.0.0.1/unsubscribe",
        "https://169.254.169.254/latest/meta-data",
        "not a url",
    ] {
        assert!(
            validate_one_click_url(invalid, OneClickPolicy::strict()).is_err(),
            "{invalid}"
        );
    }
    assert!(
        validate_one_click_url(
            "https://newsletter.example/unsubscribe",
            OneClickPolicy::strict()
        )
        .is_ok()
    );

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/unsubscribe"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .and(body_string("List-Unsubscribe=One-Click"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    let db = TestDb::new().await;
    let user = UserFactory::new().insert(db.pool()).await;
    let sender = PgEmailSenderRepository::new(db.pool().clone())
        .upsert_for_user(
            user.id,
            &CanonicalAddress::new("digest@example.com"),
            Some("<digest.example>"),
            Some("Digest"),
        )
        .await
        .unwrap();
    let targets = PgEmailUnsubscribeTargetRepository::new(db.pool().clone());
    targets
        .upsert(
            sender.id,
            UnsubscribeTargetUpsert {
                one_click_post_url: Some(format!("{}/unsubscribe", server.uri())),
                mailto_addr: Some("leave@example.com".into()),
                web_url: Some("https://example.com/preferences".into()),
            },
        )
        .await
        .unwrap();
    let mut worker = common::build_worker_ctx_with_email_services(&db).await;
    worker.email_unsubscribe_url_policy = OneClickPolicy {
        allow_http: true,
        allow_private_ips: true,
    };
    let deps = worker.email_jobs();
    let job = EmailUnsubscribeJob {
        user_id: user.id,
        sender_id: sender.id,
    };
    let claimed = dispatch_generic_job(
        &deps,
        GenericJobEnvelope {
            outbox_id: JobOutboxId::new(),
            job_type: "email.unsubscribe".into(),
            payload: serde_json::to_value(&job).unwrap(),
            dedupe_key: Some(format!("unsubscribe:{}", sender.id)),
        },
    )
    .await
    .unwrap();
    assert_eq!(claimed, Some(()));
    server.verify().await;

    targets
        .upsert(
            sender.id,
            UnsubscribeTargetUpsert {
                one_click_post_url: None,
                mailto_addr: Some("leave@example.com".into()),
                web_url: None,
            },
        )
        .await
        .unwrap();
    handle_email_unsubscribe(&deps, job).await.unwrap();
    let ignored = dispatch_generic_job(
        &deps,
        GenericJobEnvelope {
            outbox_id: JobOutboxId::new(),
            job_type: "unrelated".into(),
            payload: serde_json::json!({}),
            dedupe_key: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(ignored, None);
}
