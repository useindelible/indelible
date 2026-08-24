#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkingConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChunk {
    pub index: i32,
    pub content: String,
    pub token_count: i32,
}

pub fn approximate_token_count(text: &str) -> usize {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        0
    } else {
        trimmed.chars().count().div_ceil(4)
    }
}

pub fn chunk_text(text: &str, config: ChunkingConfig) -> Vec<TextChunk> {
    let normalized = normalize_text(text);
    if normalized.is_empty() {
        return Vec::new();
    }

    let sentences = split_sentences(&normalized)
        .into_iter()
        .flat_map(|sentence| split_oversized_sentence(&sentence, config))
        .collect::<Vec<_>>();
    if sentences.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut start = 0_usize;

    while start < sentences.len() {
        let mut end = start;
        let mut selected = Vec::new();
        let mut total_tokens = 0_usize;

        while end < sentences.len() {
            let sentence = &sentences[end];
            let sentence_tokens = approximate_token_count(sentence);
            if !selected.is_empty() && total_tokens + sentence_tokens > config.chunk_size {
                break;
            }

            total_tokens += sentence_tokens;
            selected.push(sentence.clone());
            end += 1;

            if sentence_tokens > config.chunk_size {
                break;
            }
        }

        let content = selected.join(" ");
        let token_count = approximate_token_count(&content).max(1);
        chunks.push(TextChunk {
            index: chunks.len() as i32,
            content,
            token_count: token_count as i32,
        });

        if end >= sentences.len() {
            break;
        }

        let mut next_start = end;
        let mut overlap_tokens = 0_usize;

        while next_start > start + 1 {
            let candidate_tokens = approximate_token_count(&sentences[next_start - 1]);
            if overlap_tokens > 0 && overlap_tokens + candidate_tokens > config.chunk_overlap {
                break;
            }

            overlap_tokens += candidate_tokens;
            next_start -= 1;

            if overlap_tokens >= config.chunk_overlap {
                break;
            }
        }

        if next_start <= start {
            next_start = end;
        }

        start = next_start;
    }

    chunks
}

fn split_oversized_sentence(sentence: &str, config: ChunkingConfig) -> Vec<String> {
    if approximate_token_count(sentence) <= config.chunk_size.max(1) {
        return vec![sentence.to_string()];
    }

    let max_chars = config.chunk_size.max(1).saturating_mul(4).max(1);
    let overlap_chars = config
        .chunk_overlap
        .saturating_mul(4)
        .min(max_chars.saturating_sub(1));
    let chars = sentence.chars().collect::<Vec<_>>();
    let mut chunks = Vec::new();
    let mut start = 0_usize;

    while start < chars.len() {
        let end = start.saturating_add(max_chars).min(chars.len());
        let chunk = chars[start..end].iter().copied().collect::<String>();
        if !chunk.is_empty() {
            chunks.push(chunk);
        }
        if end >= chars.len() {
            break;
        }
        start = end.saturating_sub(overlap_chars);
    }

    chunks
}

pub fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Postgres `text` rejects NUL (`0x00`); everything else it stores as-is.
pub fn strip_nul(value: &str) -> String {
    value.replace('\0', "")
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut sentence_start = 0_usize;

    for (idx, ch) in text.char_indices() {
        if !matches!(ch, '.' | '!' | '?') {
            continue;
        }

        let next_index = idx + ch.len_utf8();
        let next_char = text[next_index..].chars().next();
        if next_char.is_some_and(|next| !next.is_whitespace()) {
            continue;
        }

        let sentence = text[sentence_start..next_index].trim();
        if !sentence.is_empty() {
            sentences.push(sentence.to_string());
        }
        sentence_start = next_index;
    }

    let tail = text[sentence_start..].trim();
    if !tail.is_empty() {
        sentences.push(tail.to_string());
    }

    if sentences.is_empty() {
        vec![text.trim().to_string()]
    } else {
        sentences
    }
}

#[cfg(test)]
mod strip_nul_tests {
    use super::strip_nul;

    #[test]
    fn removes_nul_and_nothing_else() {
        assert_eq!(strip_nul("Insider\u{0}s Guide"), "Insiders Guide");
        assert_eq!(
            strip_nul("a\tb\nc \u{2019} \u{e9}"),
            "a\tb\nc \u{2019} \u{e9}"
        );
    }
}
