use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use minijinja::value::Value;
use minijinja::{Environment, Error, ErrorKind};

use super::category::category_for_item_type;
use super::types::{ObsidianRenderDocument, ObsidianRenderError, ObsidianRenderHighlight};

pub(super) fn render_template(
    name: &'static str,
    template: &str,
    doc: &ObsidianRenderDocument,
    highlights: &[ObsidianRenderHighlight],
    is_new_page: bool,
    now: DateTime<Utc>,
    highlight: Option<&ObsidianRenderHighlight>,
) -> Result<String, ObsidianRenderError> {
    let env = obsidian_template_environment();
    let tpl = env
        .template_from_str(template)
        .map_err(|source| ObsidianRenderError::Template { name, source })?;
    let mut ctx = document_template_context(doc, highlights, is_new_page, now);
    if let Some(h) = highlight {
        // highlight_text / highlight_note are body-only and legitimately
        // multi-line; keep newlines but still neutralize embed/wikilink markup.
        ctx.insert(
            "highlight_text",
            Value::from(sanitize_obsidian_value(&h.text, true)),
        );
        insert_sanitized_optional(&mut ctx, "highlight_location", h.location.as_deref());
        insert_sanitized_optional(
            &mut ctx,
            "highlight_location_url",
            h.location_url.as_deref(),
        );
        ctx.insert("highlight_tags", Value::from(sanitize_tags(&h.tags)));
        insert_optional_block(&mut ctx, "highlight_note", h.note.as_deref());
        ctx.insert(
            "color",
            Value::from(sanitize_obsidian_value(&h.color, false)),
        );
        ctx.insert("created_at", Value::from(h.created_at.to_rfc3339()));
    }
    let rendered = tpl.render(Value::from_object(ctx));
    rendered.map_err(|source| ObsidianRenderError::Template { name, source })
}

fn document_template_context(
    doc: &ObsidianRenderDocument,
    highlights: &[ObsidianRenderHighlight],
    is_new_page: bool,
    now: DateTime<Utc>,
) -> BTreeMap<&'static str, Value> {
    let mut ctx = BTreeMap::new();
    ctx.insert(
        "title",
        Value::from(sanitize_obsidian_value(&doc.title, false)),
    );
    ctx.insert(
        "full_title",
        Value::from(sanitize_obsidian_value(&doc.full_title, false)),
    );
    insert_sanitized_optional(&mut ctx, "author", doc.author.as_deref());
    insert_sanitized_optional(&mut ctx, "url", doc.url.as_deref());
    // category comes from a trusted enum, no sanitization needed.
    ctx.insert(
        "category",
        Value::from(category_for_item_type(doc.item_type)),
    );
    insert_sanitized_optional(&mut ctx, "image_url", doc.image_url.as_deref());
    ctx.insert(
        "document_tags",
        Value::from(sanitize_tags(&doc.document_tags)),
    );
    insert_sanitized_optional(&mut ctx, "summary", doc.summary.as_deref());
    ctx.insert("date", Value::from(format_date(now, "F j, Y")));
    ctx.insert("time", Value::from(now.format("%H:%M").to_string()));
    ctx.insert("is_new_page", Value::from(is_new_page));
    ctx.insert("has_new_highlights", Value::from(!highlights.is_empty()));
    ctx
}

/// Neutralize Obsidian active markup in untrusted exported values so a saved
/// title / author / tag / highlight can't transclude vault files (`![[file]]`),
/// inject wikilinks (`[[file]]`), hide content with comments (`%%...%%`), or
/// break out of YAML frontmatter via control characters. Doubled brackets are
/// split so they no longer form links/embeds. When `allow_newlines` is false
/// (inline fields like title/author/tags/url) newlines collapse to spaces,
/// closing the frontmatter-breakout vector; body fields (highlight text/note)
/// keep their line structure.
fn sanitize_obsidian_value(value: &str, allow_newlines: bool) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\n' | '\t' if allow_newlines => out.push(ch),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out.replace("[[", "[ [")
        .replace("]]", "] ]")
        .replace("%%", "% %")
}

fn sanitize_tags(tags: &[String]) -> Vec<String> {
    tags.iter()
        .map(|t| sanitize_obsidian_value(t, false))
        .collect()
}

fn insert_sanitized_optional(
    ctx: &mut BTreeMap<&'static str, Value>,
    key: &'static str,
    value: Option<&str>,
) {
    if let Some(value) = value.filter(|s| !s.is_empty()) {
        ctx.insert(key, Value::from(sanitize_obsidian_value(value, false)));
    }
}

fn insert_optional_block(
    ctx: &mut BTreeMap<&'static str, Value>,
    key: &'static str,
    value: Option<&str>,
) {
    if let Some(value) = value.filter(|s| !s.is_empty()) {
        ctx.insert(key, Value::from(sanitize_obsidian_value(value, true)));
    }
}

pub(super) fn obsidian_template_environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_filter("date", date_filter);
    env.set_unknown_method_callback(|state, value, method, args| {
        if method == "split" && value.as_str().is_some() {
            let mut filter_args = Vec::with_capacity(args.len() + 1);
            filter_args.push(value.clone());
            filter_args.extend_from_slice(args);
            state.apply_filter("split", &filter_args)
        } else {
            Err(Error::from(ErrorKind::UnknownMethod))
        }
    });
    env
}

fn date_filter(value: String, format: String) -> String {
    parse_template_date(&value)
        .map(|dt| format_date(dt, &format))
        .unwrap_or(value)
}

fn parse_template_date(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }

    ["%B %-d, %Y", "%B %e, %Y", "%B %d, %Y", "%Y-%m-%d"]
        .into_iter()
        .find_map(|format| NaiveDate::parse_from_str(value, format).ok())
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc())
}

pub(super) fn format_date(dt: DateTime<Utc>, format: &str) -> String {
    match format {
        "F j, Y" => dt.format("%B %-d, %Y").to_string(),
        "Y-m-d" => dt.format("%Y-%m-%d").to_string(),
        other => other
            .replace('Y', &dt.format("%Y").to_string())
            .replace('m', &dt.format("%m").to_string())
            .replace('d', &dt.format("%d").to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_obsidian_value;

    #[test]
    fn sanitization_table_preserves_text_without_exposing_obsidian_markup() {
        for (input, allow_newlines, expected) in [
            (
                "x]] ![[secret_note]] rest",
                true,
                "x] ] ![ [secret_note] ] rest",
            ),
            ("%%hidden%%", true, "% %hidden% %"),
            ("Title\nmalicious: value", false, "Title malicious: value"),
            ("Line one\nLine two", true, "Line one\nLine two"),
            ("A normal title", false, "A normal title"),
        ] {
            assert_eq!(
                sanitize_obsidian_value(input, allow_newlines),
                expected,
                "{input}"
            );
        }
    }
}
