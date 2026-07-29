use chrono::Utc;
use ind_domain::{TtsChunkRecordId, TtsElementTiming, TtsSpokenElement, TtsTimingSource};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub(super) enum TtsTimingError {
    #[error(
        "provider transcript covered {actual} of {expected} requested elements; heuristic fallback is disabled for transcript providers"
    )]
    IncompleteTranscript { expected: usize, actual: usize },
    #[error("provider transcript contains duplicate timing for element {element_index}")]
    DuplicateElement { element_index: i32 },
    #[error("provider transcript contains timing for unexpected element {element_index}")]
    UnexpectedElement { element_index: i32 },
    #[error("provider transcript contains an invalid range for element {element_index}")]
    InvalidRange { element_index: i32 },
    #[error("provider transcript timings are not ordered at element {element_index}")]
    OutOfOrder { element_index: i32 },
}

pub(super) fn current_month_bounds(
    now: chrono::DateTime<Utc>,
) -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
    use chrono::{Datelike, NaiveDate, TimeZone};
    let year = now.year();
    let month = now.month();
    #[expect(
        clippy::expect_used,
        reason = "year/month come from a real DateTime so day 1 is valid, and midnight is always a valid wall-clock time"
    )]
    let start = NaiveDate::from_ymd_opt(year, month, 1)
        .expect("year/month in range")
        .and_hms_opt(0, 0, 0)
        .expect("start of day in range");
    let start = Utc.from_utc_datetime(&start);
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    #[expect(
        clippy::expect_used,
        reason = "next month's day 1 is always a valid date and midnight is always a valid wall-clock time"
    )]
    let end = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .expect("year/month in range")
        .and_hms_opt(0, 0, 0)
        .expect("start of day in range");
    let end = Utc.from_utc_datetime(&end);
    (start, end)
}

/// Canonical upper-bound estimate of synthesised audio seconds for a chunk,
/// reconciled to provider-reported actuals by `ensure_actual_usage_reserved`.
/// Backend audio is always generated at the provider's default pace; the
/// player-side playback rate is applied by the client and never participates
/// in backend quota.
pub(super) fn estimate_audio_seconds(normalized_text: &str) -> i64 {
    let chars = normalized_text.chars().count() as f64;
    if chars <= 0.0 {
        return 0;
    }
    (chars / 5.0).ceil() as i64
}

pub(super) fn build_element_timings(
    chunk_record_id: TtsChunkRecordId,
    elements: &[TtsSpokenElement],
    duration_seconds: f64,
    provider_timings: &[ind_domain::ProviderElementTiming],
    timing_source: TtsTimingSource,
) -> Result<Vec<TtsElementTiming>, TtsTimingError> {
    if elements.is_empty() {
        return Ok(Vec::new());
    }

    match timing_source {
        TtsTimingSource::ProviderTranscript => {
            build_provider_timings(chunk_record_id, elements, provider_timings)
        }
        TtsTimingSource::Heuristic => Ok(build_heuristic_timings(
            chunk_record_id,
            elements,
            duration_seconds,
        )),
    }
}

fn build_provider_timings(
    chunk_record_id: TtsChunkRecordId,
    elements: &[TtsSpokenElement],
    provider_timings: &[ind_domain::ProviderElementTiming],
) -> Result<Vec<TtsElementTiming>, TtsTimingError> {
    let expected_indices = elements
        .iter()
        .map(|element| element.element_index)
        .collect::<std::collections::HashSet<_>>();
    let mut by_element = std::collections::HashMap::with_capacity(provider_timings.len());
    for timing in provider_timings {
        if !expected_indices.contains(&timing.element_index) {
            return Err(TtsTimingError::UnexpectedElement {
                element_index: timing.element_index,
            });
        }
        if by_element.insert(timing.element_index, timing).is_some() {
            return Err(TtsTimingError::DuplicateElement {
                element_index: timing.element_index,
            });
        }
    }
    if by_element.len() != elements.len() {
        return Err(TtsTimingError::IncompleteTranscript {
            expected: elements.len(),
            actual: by_element.len(),
        });
    }

    let mut previous_start = 0.0;
    elements
        .iter()
        .map(|element| {
            let timing = by_element.get(&element.element_index).ok_or(
                TtsTimingError::IncompleteTranscript {
                    expected: elements.len(),
                    actual: by_element.len(),
                },
            )?;
            let Some(end_timestamp) = timing.end_timestamp else {
                return Err(TtsTimingError::InvalidRange {
                    element_index: element.element_index,
                });
            };
            if !timing.start_timestamp.is_finite()
                || !end_timestamp.is_finite()
                || timing.start_timestamp < 0.0
                || end_timestamp <= timing.start_timestamp
            {
                return Err(TtsTimingError::InvalidRange {
                    element_index: element.element_index,
                });
            }
            if timing.start_timestamp < previous_start {
                return Err(TtsTimingError::OutOfOrder {
                    element_index: element.element_index,
                });
            }
            previous_start = timing.start_timestamp;
            Ok(TtsElementTiming {
                chunk_record_id,
                element_index: element.element_index,
                start_timestamp: timing.start_timestamp,
                end_timestamp: Some(end_timestamp),
            })
        })
        .collect()
}

fn build_heuristic_timings(
    chunk_record_id: TtsChunkRecordId,
    elements: &[TtsSpokenElement],
    duration_seconds: f64,
) -> Vec<TtsElementTiming> {
    let duration_seconds = duration_seconds.max(0.0);
    let total_weight = elements
        .iter()
        .map(element_timing_weight)
        .sum::<f64>()
        .max(1.0);
    let mut prefix_weight = 0.0;

    elements
        .iter()
        .enumerate()
        .map(|(idx, element)| {
            let weight = element_timing_weight(element);
            let derived_start = duration_seconds * (prefix_weight / total_weight);
            prefix_weight += weight;
            let derived_end = if idx + 1 == elements.len() {
                duration_seconds
            } else {
                duration_seconds * (prefix_weight / total_weight)
            };

            TtsElementTiming {
                chunk_record_id,
                element_index: element.element_index,
                start_timestamp: derived_start,
                end_timestamp: Some(derived_end),
            }
        })
        .collect()
}

fn element_timing_weight(element: &TtsSpokenElement) -> f64 {
    let chars = element.text.chars().collect::<Vec<_>>();
    let spoken_chars = chars
        .iter()
        .map(|ch| speech_char_weight(*ch, element.kind))
        .sum::<f64>()
        .max(1.0);
    let sentence_pauses = chars
        .iter()
        .enumerate()
        .filter(|(idx, ch)| is_sentence_pause(&chars, *idx, **ch))
        .count() as f64
        * 4.0;
    let clause_pauses = chars
        .iter()
        .filter(|ch| matches!(ch, ',' | ';' | ':'))
        .count() as f64
        * 1.5;
    let code_symbols = if element.kind == ind_domain::TtsElementKind::Code {
        chars
            .iter()
            .filter(|ch| !ch.is_alphanumeric() && !ch.is_whitespace())
            .count() as f64
            * 1.25
    } else {
        0.0
    };
    let kind_pause = match element.kind {
        ind_domain::TtsElementKind::Title => 8.0,
        ind_domain::TtsElementKind::Heading => 8.0,
        ind_domain::TtsElementKind::Blockquote => 4.0,
        ind_domain::TtsElementKind::ListItem | ind_domain::TtsElementKind::Caption => 2.0,
        ind_domain::TtsElementKind::Paragraph | ind_domain::TtsElementKind::Code => 0.0,
    };
    spoken_chars + sentence_pauses + clause_pauses + code_symbols + kind_pause
}

fn speech_char_weight(ch: char, kind: ind_domain::TtsElementKind) -> f64 {
    if ch.is_whitespace() {
        return 0.0;
    }
    let base = if ch.is_ascii_digit() {
        1.7
    } else if is_cjk(ch) {
        1.5
    } else if !ch.is_alphanumeric() {
        0.65
    } else {
        1.0
    };
    if kind == ind_domain::TtsElementKind::Code {
        base * 1.35
    } else {
        base
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x3040..=0x309F
            | 0x30A0..=0x30FF
            | 0xAC00..=0xD7AF
    )
}

fn is_sentence_pause(chars: &[char], idx: usize, ch: char) -> bool {
    match ch {
        '!' | '?' => true,
        '.' => !is_abbreviation_period(chars, idx),
        _ => false,
    }
}

fn is_abbreviation_period(chars: &[char], idx: usize) -> bool {
    let previous = idx.checked_sub(1).and_then(|prev| chars.get(prev)).copied();
    if previous.is_none_or(|ch| !ch.is_ascii_alphabetic()) {
        return false;
    }
    let next = chars.get(idx + 1).copied();
    if next.is_some_and(|ch| ch.is_ascii_alphabetic()) {
        return true;
    }
    if idx >= 2 && chars[idx - 2] == '.' {
        return true;
    }

    let token_start = chars[..idx]
        .iter()
        .rposition(|ch| ch.is_whitespace())
        .map(|pos| pos + 1)
        .unwrap_or(0);
    let token = chars[token_start..idx]
        .iter()
        .collect::<String>()
        .to_ascii_lowercase();
    matches!(
        token.as_str(),
        "dr" | "mr" | "mrs" | "ms" | "prof" | "sr" | "jr" | "st" | "vs" | "etc"
    )
}

#[cfg(test)]
mod tests;
