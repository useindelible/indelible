use chrono::{DateTime, Utc};
use ind_application::AppError;
use ind_domain::{DomainError, ItemType, TriageState};

use super::types::{ReadwiseCsvRow, ZipEntry};

const READWISE_CSV_HEADERS: &[&str] = &[
    "Title",
    "URL",
    "ID",
    "Document tags",
    "Saved date",
    "Reading progress",
    "Location",
    "Seen",
];

pub(super) fn parse_csv(bytes: &[u8]) -> Result<Vec<ReadwiseCsvRow>, AppError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes);

    let headers = rdr
        .headers()
        .map_err(|e| AppError::Repository(format!("CSV parse: {e}").into()))?
        .clone();

    for (i, expected) in READWISE_CSV_HEADERS.iter().enumerate() {
        let actual = headers.get(i).unwrap_or("").trim();
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(AppError::Domain(DomainError::Validation {
                field: format!("csv[{i}]"),
                message: format!(
                    "not a Readwise export CSV: expected column '{}' at position {}, got '{}'",
                    expected, i, actual
                ),
            }));
        }
    }

    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| AppError::Repository(format!("CSV parse: {e}").into()))?;

        // Columns: Title(0), URL(1), ID(2), Document tags(3), Saved date(4), Reading progress(5), Location(6), Seen(7)
        let title = record.get(0).unwrap_or("").to_string();
        let url_raw = record.get(1).unwrap_or("").to_string();
        let id = record.get(2).unwrap_or("").to_string();
        let document_tags = record.get(3).unwrap_or("").to_string();
        let saved_date = record.get(4).unwrap_or("").to_string();
        let reading_progress: f32 = record.get(5).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let location = record.get(6).unwrap_or("new").to_string();
        let seen_str = record.get(7).unwrap_or("False");
        let seen = seen_str.eq_ignore_ascii_case("true");

        if id.is_empty() {
            continue;
        }

        let url = if url_raw.is_empty() {
            None
        } else {
            Some(url_raw)
        };

        rows.push(ReadwiseCsvRow {
            title,
            url,
            id,
            document_tags,
            saved_date,
            reading_progress,
            location,
            seen,
        });
    }

    Ok(rows)
}

pub(super) fn location_to_triage(location: &str) -> (TriageState, bool) {
    match location {
        "new" | "feed" => (TriageState::Inbox, false),
        "later" => (TriageState::Later, false),
        "archive" => (TriageState::Archive, false),
        "shortlist" => (TriageState::Inbox, true),
        _ => (TriageState::Inbox, false),
    }
}

pub(super) fn detect_item_type(url: Option<&str>, zip_entry: Option<&ZipEntry>) -> ItemType {
    // URL-native content types win over ZIP HTML snapshots. For example,
    // Readwise's YouTube HTML is just a watch page, and Twitter/X status rows
    // should remain tweets even when the export includes HTML.
    if let Some(inferred) = url
        .map(ind_application::dispatch::infer_item_type_for_url)
        .filter(|item_type| *item_type != ItemType::Article)
    {
        return inferred;
    }
    if let Some(entry) = zip_entry {
        return ext_to_item_type(&entry.extension);
    }
    if url.is_some_and(|u| u.ends_with(".pdf")) {
        return ItemType::Pdf;
    }
    ItemType::Article
}

pub(super) fn ext_to_item_type(ext: &str) -> ItemType {
    match ext {
        "pdf" => ItemType::Pdf,
        "epub" => ItemType::Book,
        _ => ItemType::Article,
    }
}

pub(super) fn parse_readwise_date(s: &str) -> Result<DateTime<Utc>, AppError> {
    // Format: "2024-01-15 14:30:00.123456+00:00" or "2024-01-15 14:30:00+00:00"
    let formats = [
        "%Y-%m-%d %H:%M:%S%.f%:z",
        "%Y-%m-%d %H:%M:%S%:z",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d",
    ];
    for fmt in &formats {
        if let Ok(dt) = chrono::DateTime::parse_from_str(s, fmt) {
            return Ok(dt.with_timezone(&Utc));
        }
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
            return Ok(naive.and_utc());
        }
        if let Ok(date) = chrono::NaiveDate::parse_from_str(s, fmt) {
            #[expect(
                clippy::unwrap_used,
                reason = "midnight is always a valid wall-clock time"
            )]
            let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            return Ok(start_of_day);
        }
    }
    Err(AppError::Repository(
        format!("cannot parse date: {s}").into(),
    ))
}

pub(super) fn extract_domain(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    parsed.host_str().map(|h| h.to_string())
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct ParsedTags {
    pub(super) tags: Vec<String>,
    pub(super) errors: Vec<String>,
}

/// Parses a Python-list-style tag string like `['tag1', 'tag, comma', "it's fine"]`.
pub(super) fn parse_python_list_with_errors(input: &str) -> ParsedTags {
    let input = input.trim();
    if input.is_empty() || input == "[]" {
        return ParsedTags::default();
    }

    let mut errors = Vec::new();
    let inner = if let Some(stripped) = input.strip_prefix('[') {
        if let Some(stripped) = stripped.strip_suffix(']') {
            stripped
        } else {
            errors.push("tag literal starts with '[' but is missing closing ']'".to_string());
            stripped
        }
    } else {
        errors.push("tag literal must be a bracketed list".to_string());
        input
    };

    let mut tags = Vec::new();
    let mut chars = inner.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            ',' | ' ' | '\t' | '\r' | '\n' => {}
            '\'' | '"' => {
                let quote = ch;
                let mut current = String::new();
                let mut closed = false;
                while let Some(next) = chars.next() {
                    match next {
                        '\\' => match chars.peek() {
                            Some(&escaped) if escaped == quote || escaped == '\\' => {
                                current.push(escaped);
                                chars.next();
                            }
                            _ => current.push(next),
                        },
                        c if c == quote => {
                            closed = true;
                            break;
                        }
                        _ => current.push(next),
                    }
                }

                if !closed {
                    errors.push(format!("unterminated quoted tag: {current}"));
                    break;
                }

                let tag = current.trim().to_string();
                if !tag.is_empty() {
                    tags.push(tag);
                }

                while chars.peek().is_some_and(|p| p.is_whitespace()) {
                    chars.next();
                }
                if chars.peek().is_some_and(|&p| p == ',') {
                    chars.next();
                } else if let Some(&p) = chars.peek() {
                    errors.push(format!("unexpected character after quoted tag: {p}"));
                    while let Some(&q) = chars.peek() {
                        chars.next();
                        if q == ',' {
                            break;
                        }
                    }
                }
            }
            _ => {
                let mut token = String::from(ch);
                while let Some(&next) = chars.peek() {
                    if next == ',' {
                        break;
                    }
                    token.push(next);
                    chars.next();
                }
                let token = token.trim();
                if !token.is_empty() {
                    errors.push(format!("unquoted tag literal skipped: {token}"));
                }
            }
        }
    }

    ParsedTags { tags, errors }
}
