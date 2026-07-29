pub fn format_full_document_text(title: &str, body: &str) -> Option<String> {
    let title = title.trim();
    let body = body.trim();
    if title.is_empty() || body.is_empty() {
        return None;
    }

    let body = strip_leading_title_boilerplate(title, body);
    if body.is_empty() {
        None
    } else {
        Some(body.to_string())
    }
}

fn strip_leading_title_boilerplate<'a>(title: &str, body: &'a str) -> &'a str {
    let mut remaining = body.trim_start();
    let mut stripped_count = 0;

    while let Some(next) = strip_leading_title_once(title, remaining) {
        stripped_count += 1;
        remaining = next.trim_start();
    }

    if stripped_count == 0 {
        return body;
    }
    if stripped_count > 1 || should_strip_single_leading_title(title, remaining) {
        return remaining;
    }
    body
}

fn should_strip_single_leading_title(title: &str, remaining_after_title: &str) -> bool {
    if title.split_whitespace().count() >= 3 || title.chars().count() >= 20 {
        return true;
    }
    if title.chars().filter(|ch| ch.is_alphanumeric()).count() > 5 {
        return true;
    }

    let remaining = remaining_after_title.trim_start();
    remaining.is_empty()
        || remaining
            .chars()
            .next()
            .is_some_and(|ch| ch.is_uppercase() || ch.is_numeric())
}

fn strip_leading_title_once<'a>(title: &str, body: &'a str) -> Option<&'a str> {
    let body = body.trim_start();
    let title_tokens = normalized_title_tokens(title);
    if title_tokens.is_empty() {
        return None;
    }

    let mut cursor = 0;
    let mut last_token_end = 0;
    for expected in &title_tokens {
        let (actual, _, token_end) = next_normalized_token(body, cursor)?;
        if &actual != expected {
            return None;
        }
        cursor = token_end;
        last_token_end = token_end;
    }

    if let Some((_, next_token_start, _)) = next_normalized_token(body, cursor)
        && next_token_start == cursor
    {
        return None;
    }

    Some(
        body[last_token_end..]
            .trim_start_matches(is_title_boundary)
            .trim_start(),
    )
}

fn is_title_boundary(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(
            ch,
            '.' | ','
                | ':'
                | ';'
                | '!'
                | '?'
                | '-'
                | '—'
                | '–'
                | '/'
                | '\\'
                | '|'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '"'
                | '\''
                | '“'
                | '”'
                | '‘'
                | '’'
                | '<'
                | '>'
        )
}

fn normalized_title_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while let Some((token, _, token_end)) = next_normalized_token(text, cursor) {
        tokens.push(token);
        cursor = token_end;
    }
    tokens
}

fn next_normalized_token(text: &str, cursor: usize) -> Option<(String, usize, usize)> {
    let mut token = String::new();
    let mut token_start = None;
    let mut token_end = cursor.min(text.len());

    for (offset, ch) in text[cursor.min(text.len())..].char_indices() {
        let idx = cursor + offset;
        if ch.is_alphanumeric() {
            token_start.get_or_insert(idx);
            token.extend(ch.to_lowercase());
            token_end = idx + ch.len_utf8();
            continue;
        }

        if token_start.is_some() {
            break;
        }
    }

    token_start.map(|start| (token, start, token_end))
}
