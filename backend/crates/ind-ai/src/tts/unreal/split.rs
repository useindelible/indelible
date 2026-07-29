use crate::tts::adapter::TtsAdapterError;

/// Strip a leading ID3v2 tag from an MP3 byte slice if one is present.
///
/// MP3 frames without an ID3v2 header can be concatenated safely; a mid-stream
/// ID3v2 header, however, is not a valid MP3 frame and confuses decoders into
/// silently skipping audio or bailing out with malformed-stream errors. This
/// helper only trims the *outer* ID3v2 tag at offset 0 — embedded ID3v1
/// trailers (last 128 bytes, starting with `"TAG"`) are harmless for playback
/// and removing them would require buffering the entire window, so the adapter
/// leaves them in place.
///
/// ID3v2 layout: 3-byte magic `"ID3"`, 2-byte version, 1-byte flags,
/// 4-byte synchsafe size (each byte's high bit cleared; payload-size only,
/// excluding the 10-byte header itself, or 20 if the footer flag is set).
pub(super) fn strip_id3v2_prefix(bytes: &[u8]) -> &[u8] {
    if bytes.len() < 10 || &bytes[0..3] != b"ID3" {
        return bytes;
    }
    let flags = bytes[5];
    let has_footer = (flags & 0b0001_0000) != 0;
    let size = ((bytes[6] & 0x7F) as usize) << 21
        | ((bytes[7] & 0x7F) as usize) << 14
        | ((bytes[8] & 0x7F) as usize) << 7
        | (bytes[9] & 0x7F) as usize;
    let header_total = 10 + size + if has_footer { 10 } else { 0 };
    if header_total >= bytes.len() {
        // A malformed tag that claims to span the entire buffer is treated as
        // unstrippable; returning the original slice is the safer fallback
        // because we would otherwise emit an empty window.
        return bytes;
    }
    &bytes[header_total..]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TextWindow {
    pub(super) text: String,
    pub(super) start_char: usize,
}

/// Split `text` into source-preserving request windows, preferring paragraph
/// breaks, then sentence boundaries, then whitespace.
///
/// Retaining each window's original character offset is required because
/// Unreal's `TextOffset` values are relative to the submitted window. Rewriting
/// whitespace while splitting would make those offsets impossible to project
/// back onto the document elements reliably.
pub(super) fn split_for_request(
    text: &str,
    limit: usize,
) -> Result<Vec<TextWindow>, TtsAdapterError> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    if limit == 0 {
        return Err(TtsAdapterError::InvalidRequest(
            "unreal: request character limit must be positive".into(),
        ));
    }
    if text.chars().count() <= limit {
        return Ok(vec![TextWindow {
            text: text.to_string(),
            start_char: 0,
        }]);
    }

    let chars = text.chars().collect::<Vec<_>>();
    let mut windows = Vec::new();
    let mut cursor = 0usize;

    while cursor < chars.len() {
        while cursor < chars.len() && chars[cursor].is_whitespace() {
            cursor += 1;
        }
        if cursor == chars.len() {
            break;
        }

        let hard_end = (cursor + limit).min(chars.len());
        let end = if hard_end == chars.len() {
            hard_end
        } else {
            preferred_window_end(&chars, cursor, hard_end).ok_or_else(|| {
                TtsAdapterError::InvalidRequest(format!(
                    "unreal: text contains a {limit}+ char token with no breakable boundary"
                ))
            })?
        };
        if end <= cursor {
            return Err(TtsAdapterError::InvalidRequest(
                "unreal: text contains no usable request boundary".into(),
            ));
        }

        windows.push(TextWindow {
            text: chars[cursor..end].iter().collect(),
            start_char: cursor,
        });
        cursor = end;
    }

    Ok(windows)
}

fn preferred_window_end(chars: &[char], start: usize, hard_end: usize) -> Option<usize> {
    if let Some(second_newline) = (start + 1..hard_end)
        .rev()
        .find(|index| chars[index - 1] == '\n' && chars[*index] == '\n')
    {
        return Some(second_newline - 1);
    }

    if let Some(sentence_end) = (start..hard_end).rev().find(|index| {
        matches!(chars[*index], '.' | '!' | '?')
            && chars
                .get(index + 1)
                .is_some_and(|next| next.is_whitespace())
    }) {
        return Some(sentence_end + 1);
    }

    if chars
        .get(hard_end)
        .is_some_and(|boundary| boundary.is_whitespace())
    {
        return Some(hard_end);
    }

    (start + 1..hard_end)
        .rev()
        .find(|index| chars[*index].is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitter_preserves_content_within_character_limits() {
        assert!(split_for_request("", 5).unwrap().is_empty());
        assert_eq!(
            split_for_request("héllo", 5).unwrap(),
            [TextWindow {
                text: "héllo".into(),
                start_char: 0
            }]
        );

        let text = "First sentence. Second sentence.\n\nThird paragraph.";
        let chunks = split_for_request(text, 20).unwrap();
        assert!(chunks.iter().all(|chunk| chunk.text.chars().count() <= 20));
        for fragment in ["First sentence.", "Second sentence.", "Third paragraph."] {
            assert!(chunks.iter().any(|chunk| chunk.text.contains(fragment)));
        }
        for chunk in chunks {
            let source = text
                .chars()
                .skip(chunk.start_char)
                .take(chunk.text.chars().count())
                .collect::<String>();
            assert_eq!(chunk.text, source);
        }
        assert!(matches!(
            split_for_request(&"x".repeat(21), 20),
            Err(TtsAdapterError::InvalidRequest(_))
        ));
    }

    #[test]
    fn splitter_retains_original_offsets_and_whitespace() {
        let text = "Alpha.\n\nBeta   gamma delta.";
        let chunks = split_for_request(text, 12).unwrap();
        assert_eq!(
            chunks,
            [
                TextWindow {
                    text: "Alpha.".into(),
                    start_char: 0,
                },
                TextWindow {
                    text: "Beta   gamma".into(),
                    start_char: 8,
                },
                TextWindow {
                    text: "delta.".into(),
                    start_char: 21,
                },
            ]
        );
    }

    #[test]
    fn id3_prefix_removal_is_bounded_and_footer_aware() {
        let raw = [0xff, 0xfb, 0x00, 0x55];
        assert_eq!(strip_id3v2_prefix(&raw), raw);

        let mut tagged = b"ID3\x04\x00\x00\x00\x00\x00\x02".to_vec();
        tagged.extend_from_slice(&[0xaa, 0xbb]);
        tagged.extend_from_slice(&raw);
        assert_eq!(strip_id3v2_prefix(&tagged), raw);

        let malformed = b"ID3\x04\x00\x00\x7f\x7f\x7f\x7f\xaa";
        assert_eq!(strip_id3v2_prefix(malformed), malformed);

        let mut footer = b"ID3\x04\x00\x10\x00\x00\x00\x00".to_vec();
        footer.extend_from_slice(b"3DI\x04\x00\x10\x00\x00\x00\x00");
        footer.extend_from_slice(&raw);
        assert_eq!(strip_id3v2_prefix(&footer), raw);
    }
}
