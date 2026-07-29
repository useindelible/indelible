use std::collections::HashMap;

use scraper::{Html, Selector};

use super::cleaner::clean_email_html;
use super::types::EmailDestination;

const MODE_A_TEXT_THRESHOLD: usize = 500;

pub fn canonicalize_address(address: &str) -> String {
    let trimmed = address.trim().to_lowercase();
    match trimmed.rsplit_once('@') {
        Some((local, domain)) => {
            let base_local = local.split('+').next().unwrap_or(local);
            format!("{base_local}@{domain}")
        }
        None => trimmed,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForwardedSender {
    pub original_address: String,
    pub display_name: Option<String>,
    pub is_forwarded: bool,
}

pub fn parse_forwarded_headers(
    headers: &HashMap<String, String>,
    envelope_from: &str,
) -> ForwardedSender {
    if let Some(value) = header_lookup(headers, "X-Original-From") {
        let (display_name, addr) = split_addr_spec(value);
        return ForwardedSender {
            original_address: canonicalize_address(&addr),
            display_name,
            is_forwarded: true,
        };
    }

    if header_lookup(headers, "X-Forwarded-For").is_some() {
        if let Some(value) = header_lookup(headers, "Reply-To") {
            let (display_name, addr) = split_addr_spec(value);
            return ForwardedSender {
                original_address: canonicalize_address(&addr),
                display_name,
                is_forwarded: true,
            };
        }
        return ForwardedSender {
            original_address: canonicalize_address(envelope_from),
            display_name: None,
            is_forwarded: true,
        };
    }

    ForwardedSender {
        original_address: canonicalize_address(envelope_from),
        display_name: None,
        is_forwarded: false,
    }
}

fn header_lookup<'a>(headers: &'a HashMap<String, String>, name: &str) -> Option<&'a String> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v)
}

fn split_addr_spec(raw: &str) -> (Option<String>, String) {
    let raw = raw.trim();
    if let Some(open) = raw.find('<')
        && let Some(close) = raw.rfind('>')
        && close > open
    {
        let name = raw[..open].trim().trim_matches('"').trim();
        let addr = raw[open + 1..close].trim().to_string();
        let display_name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        return (display_name, addr);
    }
    (None, raw.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIngestAddress {
    pub token: String,
    pub destination: EmailDestination,
}

pub fn format_ingest_address(
    local_part: &str,
    destination: EmailDestination,
    feed_domain: Option<&str>,
    library_domain: Option<&str>,
) -> Option<String> {
    let domain = match destination {
        EmailDestination::Feed => feed_domain?,
        EmailDestination::Library => library_domain?,
    };
    let suffix = match (feed_domain, library_domain) {
        (Some(feed), Some(library)) if feed.eq_ignore_ascii_case(library) => match destination {
            EmailDestination::Feed => "-feed",
            EmailDestination::Library => "-lib",
        },
        _ => "",
    };
    Some(format!("{local_part}{suffix}@{domain}"))
}

pub fn parse_ingest_address(
    address: &str,
    feed_domain: &str,
    library_domain: &str,
) -> Option<ParsedIngestAddress> {
    let (local, domain) = address.rsplit_once('@')?;

    let (raw_local, destination) = if feed_domain.eq_ignore_ascii_case(library_domain)
        && domain.eq_ignore_ascii_case(feed_domain)
    {
        if let Some(token) = local.strip_suffix("-feed") {
            (token, EmailDestination::Feed)
        } else if let Some(token) = local.strip_suffix("-lib") {
            (token, EmailDestination::Library)
        } else {
            (local, EmailDestination::Feed)
        }
    } else if domain.eq_ignore_ascii_case(feed_domain) {
        (local, EmailDestination::Feed)
    } else if domain.eq_ignore_ascii_case(library_domain) {
        (local, EmailDestination::Library)
    } else if let Some(token) = local.strip_suffix("-feed") {
        // Shared-domain fallback: token-feed@shared
        (token, EmailDestination::Feed)
    } else if let Some(token) = local.strip_suffix("-lib") {
        // Shared-domain fallback: token-lib@shared
        (token, EmailDestination::Library)
    } else {
        return None;
    };

    // Accept any syntactically valid ingest local part: a user-customised alias or a
    // generated seed token (whose 8-char lowercase-alphanumeric shape is a strict subset
    // of the alias rules). validate_local_part enforces the alias charset/length limits,
    // normalises case, and rejects malformed / enumeration probes before any DB hit; the
    // recipient resolver then decides whether it maps to a real alias or token.
    let token = ind_domain::validate_local_part(raw_local).ok()?;

    Some(ParsedIngestAddress { token, destination })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentMode {
    ModeA,
    ModeB,
}

pub fn detect_content_mode(text_body: Option<&str>, html_body: Option<&str>) -> ContentMode {
    if let Some(text) = text_body {
        let stripped = strip_boilerplate(text);
        if stripped.len() > MODE_A_TEXT_THRESHOLD {
            return ContentMode::ModeA;
        }
    }

    if let Some(html) = html_body {
        let text_from_html = ind_html::html_to_text(&clean_email_html(html));
        let stripped = strip_boilerplate(&text_from_html);
        if stripped.len() > MODE_A_TEXT_THRESHOLD {
            return ContentMode::ModeA;
        }
    }

    ContentMode::ModeB
}

pub fn extract_primary_url(html: Option<&str>, text: Option<&str>) -> Option<String> {
    if let Some(html) = html
        && let Some(url) = extract_url_from_html(html)
    {
        return Some(url);
    }

    if let Some(text) = text
        && let Some(url) = extract_url_from_text(text)
    {
        return Some(url);
    }

    None
}

fn extract_url_from_html(html: &str) -> Option<String> {
    let selector = Selector::parse("a[href]").ok()?;
    let cleaned = clean_email_html(html);
    let fragment = Html::parse_fragment(&cleaned);

    for anchor in fragment.select(&selector) {
        if let Some(url) = anchor.value().attr("href")
            && is_article_candidate(url)
        {
            return Some(url.to_string());
        }
    }

    None
}

fn extract_url_from_text(text: &str) -> Option<String> {
    for word in text.split_whitespace() {
        let trimmed = word.trim_matches(|c: char| {
            matches!(
                c,
                '<' | '>' | '(' | ')' | '"' | '\'' | ',' | '.' | ';' | ':'
            )
        });
        if is_article_candidate(trimmed) {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn is_article_candidate(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };

    if !matches!(parsed.scheme(), "http" | "https") {
        return false;
    }

    let dominated = [
        "unsubscribe",
        "mailto:",
        "manage-preferences",
        "email-preferences",
        "list-unsubscribe",
        "tracking",
        "click.convertkit",
        "trk.klclick",
        "email.mg.",
    ];

    let lower = url.to_ascii_lowercase();
    for skip in &dominated {
        if lower.contains(skip) {
            return false;
        }
    }

    true
}

fn strip_boilerplate(text: &str) -> String {
    let skip_lines = [
        "unsubscribe",
        "view in browser",
        "view this email",
        "email preferences",
        "manage your subscription",
        "update your preferences",
        "sent to you because",
        "©",
        "all rights reserved",
    ];

    text.lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            let trimmed = lower.trim();
            if trimmed.is_empty() {
                return false;
            }
            !skip_lines.iter().any(|s| trimmed.contains(s))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests;
