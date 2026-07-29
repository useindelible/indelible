use ind_domain::{
    ProviderElementTiming, TtsChunkRecordId, TtsElementKind, TtsSpokenElement, TtsTimingSource,
};
use uuid::Uuid;

use super::{TtsTimingError, build_element_timings};

fn element(element_index: i32, text: &str) -> TtsSpokenElement {
    TtsSpokenElement {
        element_index,
        kind: TtsElementKind::Paragraph,
        text: text.to_string(),
        char_start: 0,
        char_end: text.chars().count() as i32,
        chunk_id: "section_001".into(),
    }
}

fn chunk_id() -> TtsChunkRecordId {
    TtsChunkRecordId::from_uuid(Uuid::now_v7())
}

#[test]
fn provider_transcript_ranges_are_authoritative() {
    let elements = [element(4, "Short."), element(5, "Much longer paragraph.")];
    let provider = [
        ProviderElementTiming {
            element_index: 4,
            start_timestamp: 0.5,
            end_timestamp: Some(1.25),
        },
        ProviderElementTiming {
            element_index: 5,
            start_timestamp: 1.75,
            end_timestamp: Some(9.5),
        },
    ];

    let timings = build_element_timings(
        chunk_id(),
        &elements,
        1000.0,
        &provider,
        TtsTimingSource::ProviderTranscript,
    )
    .unwrap();

    assert_eq!(timings[0].start_timestamp, 0.5);
    assert_eq!(timings[0].end_timestamp, Some(1.25));
    assert_eq!(timings[1].start_timestamp, 1.75);
    assert_eq!(timings[1].end_timestamp, Some(9.5));
}

#[test]
fn transcript_providers_never_mix_in_heuristic_ranges() {
    let elements = [element(4, "First."), element(5, "Second.")];
    let partial = [ProviderElementTiming {
        element_index: 4,
        start_timestamp: 0.5,
        end_timestamp: Some(1.25),
    }];

    let error = build_element_timings(
        chunk_id(),
        &elements,
        10.0,
        &partial,
        TtsTimingSource::ProviderTranscript,
    )
    .unwrap_err();

    assert_eq!(
        error,
        TtsTimingError::IncompleteTranscript {
            expected: 2,
            actual: 1,
        }
    );
}

#[test]
fn providers_without_transcripts_use_only_heuristic_ranges() {
    let elements = [element(4, "A."), element(5, "A much longer paragraph.")];
    let ignored_provider_row = [ProviderElementTiming {
        element_index: 4,
        start_timestamp: 99.0,
        end_timestamp: Some(100.0),
    }];

    let timings = build_element_timings(
        chunk_id(),
        &elements,
        12.0,
        &ignored_provider_row,
        TtsTimingSource::Heuristic,
    )
    .unwrap();

    assert_eq!(timings.len(), 2);
    assert_eq!(timings[0].start_timestamp, 0.0);
    assert_eq!(timings[1].end_timestamp, Some(12.0));
    assert_ne!(timings[0].start_timestamp, 99.0);
}
