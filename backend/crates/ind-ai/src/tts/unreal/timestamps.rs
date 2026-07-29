use ind_domain::{ProviderElementTiming, TtsSpokenElement};
use serde::Deserialize;

use super::UnrealSpeechAdapter;
use crate::tts::adapter::TtsAdapterError;

impl UnrealSpeechAdapter {
    pub(super) async fn fetch_timestamp_rows(
        &self,
        timestamps_uri: &str,
    ) -> Result<Vec<UnrealTimestampRow>, TtsAdapterError> {
        let response = self.get_download(timestamps_uri).await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TtsAdapterError::ProviderError {
                status_code: status.as_u16(),
                message: format!("unreal TimestampsUri download failed: {body}"),
            });
        }
        let payload: UnrealTimestampPayload = response.json().await.map_err(|e| {
            TtsAdapterError::MalformedResponse(format!(
                "failed to decode unreal timestamp rows: {e}"
            ))
        })?;
        Ok(payload.into_rows())
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum UnrealTimestampPayload {
    Rows(Vec<UnrealTimestampRow>),
    Envelope {
        #[serde(default, alias = "Timestamps")]
        timestamps: Vec<UnrealTimestampRow>,
    },
}

impl UnrealTimestampPayload {
    fn into_rows(self) -> Vec<UnrealTimestampRow> {
        match self {
            Self::Rows(rows) => rows,
            Self::Envelope { timestamps } => timestamps,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct UnrealTimestampRow {
    #[serde(default, alias = "Start", alias = "start_time", alias = "StartTime")]
    start: Option<f64>,
    #[serde(default, alias = "End", alias = "end_time", alias = "EndTime")]
    end: Option<f64>,
    #[serde(default, alias = "TextOffset", alias = "textOffset")]
    text_offset: Option<usize>,
}

#[derive(Debug, Clone)]
pub(super) struct ElementTextRange {
    element_index: i32,
    start_char: usize,
    end_char: usize,
}

pub(super) fn element_ranges_for_text(
    text: &str,
    elements: &[TtsSpokenElement],
) -> Vec<ElementTextRange> {
    let mut ranges = Vec::with_capacity(elements.len());
    let mut search_byte = 0usize;
    let mut fallback_char = 0usize;

    for element in elements {
        let found = text
            .get(search_byte..)
            .and_then(|remaining| remaining.find(&element.text))
            .map(|relative_byte| search_byte + relative_byte);
        let start_char = found
            .map(|byte_index| text[..byte_index].chars().count())
            .unwrap_or(fallback_char);
        let end_char = start_char + element.text.chars().count();
        if let Some(byte_index) = found {
            search_byte = byte_index + element.text.len();
        }
        fallback_char = end_char + 1;
        ranges.push(ElementTextRange {
            element_index: element.element_index,
            start_char,
            end_char,
        });
    }

    ranges
}

pub(super) fn map_timestamp_rows_to_elements(
    element_ranges: &[ElementTextRange],
    rows: &[UnrealTimestampRow],
    window_char_offset: usize,
    seconds_offset: f64,
) -> Vec<ProviderElementTiming> {
    element_ranges
        .iter()
        .filter_map(|range| {
            let mut start: Option<f64> = None;
            let mut end: Option<f64> = None;

            for row in rows {
                let Some(text_offset) = row.text_offset else {
                    continue;
                };
                let Some(row_start) = row.start else {
                    continue;
                };
                let Some(row_end) = row.end else {
                    continue;
                };
                let absolute_char = window_char_offset + text_offset;
                if absolute_char < range.start_char || absolute_char >= range.end_char {
                    continue;
                }
                start = Some(start.map_or(row_start, |current| current.min(row_start)));
                end = Some(end.map_or(row_end, |current| current.max(row_end)));
            }

            start.map(|start_timestamp| ProviderElementTiming {
                element_index: range.element_index,
                start_timestamp: start_timestamp + seconds_offset,
                end_timestamp: end.map(|end_timestamp| end_timestamp + seconds_offset),
            })
        })
        .collect()
}

pub(super) fn timestamp_window_duration(rows: &[UnrealTimestampRow]) -> Option<f64> {
    rows.iter()
        .filter_map(|row| row.end)
        .filter(|end| end.is_finite() && *end >= 0.0)
        .max_by(|a, b| a.total_cmp(b))
}

pub(super) fn merge_element_timings(
    mut timings: Vec<ProviderElementTiming>,
) -> Vec<ProviderElementTiming> {
    timings.sort_by_key(|timing| timing.element_index);
    let mut merged = Vec::<ProviderElementTiming>::with_capacity(timings.len());

    for timing in timings {
        if let Some(previous) = merged.last_mut()
            && previous.element_index == timing.element_index
        {
            previous.start_timestamp = previous.start_timestamp.min(timing.start_timestamp);
            previous.end_timestamp = match (previous.end_timestamp, timing.end_timestamp) {
                (Some(previous), Some(current)) => Some(previous.max(current)),
                (previous, current) => previous.or(current),
            };
        } else {
            merged.push(timing);
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use ind_domain::TtsElementKind;

    fn element(index: i32, text: &str) -> TtsSpokenElement {
        TtsSpokenElement {
            element_index: index,
            kind: TtsElementKind::Paragraph,
            text: text.into(),
            char_start: 0,
            char_end: text.chars().count() as i32,
            chunk_id: "chunk".into(),
        }
    }

    #[test]
    fn timestamp_aliases_map_unicode_elements_and_offsets() {
        let rows: Vec<UnrealTimestampRow> = serde_json::from_value(serde_json::json!([
            {"Start": 0.1, "EndTime": 0.4, "TextOffset": 0},
            {"start_time": 0.5, "end_time": 0.9, "textOffset": 6},
            {"Start": 1.0, "End": 1.4, "TextOffset": 12}
        ]))
        .unwrap();
        let ranges = element_ranges_for_text(
            "héllo world again",
            &[element(3, "héllo"), element(4, "world")],
        );
        let timings = map_timestamp_rows_to_elements(&ranges, &rows, 0, 2.0);
        assert_eq!(timings.len(), 2);
        assert_eq!(timings[0].element_index, 3);
        assert_eq!(timings[0].start_timestamp, 2.1);
        assert_eq!(timings[1].end_timestamp, Some(2.9));
        assert_eq!(timestamp_window_duration(&rows), Some(1.4));
    }

    #[test]
    fn timings_for_an_element_spanning_windows_are_merged() {
        let timings = merge_element_timings(vec![
            ProviderElementTiming {
                element_index: 4,
                start_timestamp: 2.5,
                end_timestamp: Some(4.0),
            },
            ProviderElementTiming {
                element_index: 4,
                start_timestamp: 0.5,
                end_timestamp: Some(2.0),
            },
            ProviderElementTiming {
                element_index: 5,
                start_timestamp: 4.2,
                end_timestamp: Some(5.0),
            },
        ]);

        assert_eq!(timings.len(), 2);
        assert_eq!(timings[0].element_index, 4);
        assert_eq!(timings[0].start_timestamp, 0.5);
        assert_eq!(timings[0].end_timestamp, Some(4.0));
        assert_eq!(timings[1].element_index, 5);
    }

    #[test]
    fn timestamp_payload_accepts_rows_and_envelopes() {
        for value in [
            serde_json::json!([{"Start": 0.0, "End": 1.0}]),
            serde_json::json!({"Timestamps": [{"Start": 0.0, "End": 1.0}]}),
        ] {
            let payload: UnrealTimestampPayload = serde_json::from_value(value).unwrap();
            assert_eq!(payload.into_rows().len(), 1);
        }
    }
}
