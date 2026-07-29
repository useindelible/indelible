use super::*;
use crate::JobOutboxId;

#[test]
fn generic_job_envelope_deserializes_pre_task_193_payloads_without_dedupe_key() {
    let envelope: GenericJobEnvelope = serde_json::from_value(serde_json::json!({
        "outbox_id": JobOutboxId::new(),
        "job_type": "document.ai.embed",
        "payload": {"document_id": "doc_abc"}
    }))
    .unwrap();
    assert_eq!(envelope.job_type, "document.ai.embed");
    assert!(envelope.dedupe_key.is_none());
}
