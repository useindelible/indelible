use chrono::Utc;

use super::*;

#[test]
fn search_cursor_decodes_legacy_cursor_without_score_reference_time() {
    let saved_at = Utc::now();
    let result_id = uuid::Uuid::now_v7();
    let parsed: SearchCursor = serde_json::from_value(serde_json::json!({
        "score": 0.9,
        "saved_at": saved_at,
        "result_id": result_id,
        "section_key": "chapter-02"
    }))
    .unwrap();
    assert_eq!(
        (parsed.score, parsed.saved_at, parsed.result_id),
        (0.9, saved_at, result_id)
    );
    assert_eq!(parsed.section_key, "chapter-02");
}
