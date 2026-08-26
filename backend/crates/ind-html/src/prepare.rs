//! Sanitizes reader HTML and gives headings, footnotes, and citations durable `ind-` ids.
//! Output is byte-for-byte idempotent.

use std::cell::Cell;
use std::collections::HashSet;

use lol_html::html_content::Element;
use lol_html::{RewriteStrSettings, element, rewrite_str};
use scraper::{ElementRef, Html};
use thiserror::Error;

/// Prefix applied to every id and fragment target this module owns or rewrites.
pub const ANCHOR_ID_PREFIX: &str = "ind-";
const HEADING_ID_PREFIX: &str = "ind-toc-";

#[derive(Debug, Error)]
pub enum PrepareError {
    #[error("reader html rewrite failed: {0}")]
    Rewrite(String),
}

/// On error callers fall back to `sanitize_reader_html`; ingest never fails on this.
pub fn prepare_reader_html(html: &str) -> Result<String, PrepareError> {
    let filtered = crate::reader_allowlist::drop_foreign_iframes(html)
        .map_err(|err| PrepareError::Rewrite(err.to_string()))?;
    let sanitized = sanitize_keeping_ids(&filtered);
    let plan = build_rewrite_plan(&sanitized);
    let rewritten = apply(&plan, &sanitized)?;
    // The final pass canonicalizes attribute order so the output is byte-for-byte idempotent.
    Ok(sanitize_keeping_ids(&rewritten))
}

/// Retained ids are safe only because `apply` prefixes every one with [`ANCHOR_ID_PREFIX`].
fn sanitize_keeping_ids(html: &str) -> String {
    crate::reader_allowlist::reader_sanitizer()
        .add_generic_attributes(&["id"])
        .clean(html)
        .to_string()
}

/// Indexed by element ordinal per selector (`h1..h6`, `li`, `a[href]`) in document order.
struct RewritePlan {
    heading_ids: Vec<Option<String>>,
    li_ids: Vec<Option<String>>,
    anchor_ids: Vec<Option<String>>,
    /// `true` strips the href: its local fragment has no target in the final document.
    dead_anchors: Vec<bool>,
}

fn build_rewrite_plan(sanitized: &str) -> RewritePlan {
    let doc = Html::parse_document(sanitized);
    let elements: Vec<ElementRef<'_>> = doc
        .root_element()
        .descendants()
        .filter_map(ElementRef::wrap)
        .collect();

    // Collision namespace is the final id set: prefixed existing ids plus everything assigned below.
    let mut used: HashSet<String> = elements
        .iter()
        .filter_map(|el| el.value().attr("id"))
        .map(prefixed)
        .collect();

    let mut li_ids = Vec::new();
    for li in elements.iter().filter(|el| el.value().name() == "li") {
        li_ids.push(footnote_li_id(li, &mut used));
    }

    let anchors: Vec<&ElementRef<'_>> = elements
        .iter()
        .filter(|el| el.value().name() == "a" && el.value().attr("href").is_some())
        .collect();

    let mut anchor_ids = Vec::new();
    let mut citation_counts: Vec<(String, usize)> = Vec::new();
    for a in &anchors {
        anchor_ids.push(citation_anchor_id(a, &mut citation_counts, &mut used));
    }

    let mut heading_ids = Vec::new();
    for (ordinal, heading) in elements
        .iter()
        .filter(|el| is_heading(el.value().name()))
        .enumerate()
    {
        if heading.value().attr("id").is_some() {
            heading_ids.push(None);
            continue;
        }
        let slug = slugify(&heading.text().collect::<String>());
        let base = if slug.is_empty() {
            format!("{HEADING_ID_PREFIX}{ordinal}")
        } else {
            format!("{HEADING_ID_PREFIX}{slug}")
        };
        heading_ids.push(Some(claim_unique(base, &mut used)));
    }

    let dead_anchors = anchors
        .iter()
        .map(|a| {
            #[expect(clippy::expect_used, reason = "anchors were filtered on href presence")]
            let href = a.value().attr("href").expect("anchor has href");
            match href.strip_prefix('#') {
                Some(fragment) => !used.contains(&prefixed(fragment)),
                None => false,
            }
        })
        .collect();

    RewritePlan {
        heading_ids,
        li_ids,
        anchor_ids,
        dead_anchors,
    }
}

/// Footnote `<li>` N is identified by its `#fnref:N` (or `#fnref:N-k`) backlink.
fn footnote_li_id(li: &ElementRef<'_>, used: &mut HashSet<String>) -> Option<String> {
    if li.value().attr("id").is_some() {
        return None;
    }
    let backlink_name = li
        .descendants()
        .filter_map(ElementRef::wrap)
        .filter(|el| el.value().name() == "a")
        .filter_map(|a| a.value().attr("href"))
        .find_map(|href| fragment_name(href, "fnref:"))?;
    let id = format!("{ANCHOR_ID_PREFIX}fn:{}", occurrence_base(&backlink_name));
    if used.contains(&id) {
        return None;
    }
    used.insert(id.clone());
    Some(id)
}

/// The k-th citation of footnote N gets `fnref:N-k` (`fnref:N` for k = 1), matching its backlink.
fn citation_anchor_id(
    a: &ElementRef<'_>,
    citation_counts: &mut Vec<(String, usize)>,
    used: &mut HashSet<String>,
) -> Option<String> {
    if a.value().attr("id").is_some() {
        return None;
    }
    let name = fragment_name(a.value().attr("href")?, "fn:")?;
    let count = match citation_counts.iter_mut().find(|(n, _)| *n == name) {
        Some((_, count)) => {
            *count += 1;
            *count
        }
        None => {
            citation_counts.push((name.clone(), 1));
            1
        }
    };
    let id = if count == 1 {
        format!("{ANCHOR_ID_PREFIX}fnref:{name}")
    } else {
        format!("{ANCHOR_ID_PREFIX}fnref:{name}-{count}")
    };
    if used.contains(&id) {
        return None;
    }
    used.insert(id.clone());
    Some(id)
}

fn apply(plan: &RewritePlan, html: &str) -> Result<String, PrepareError> {
    let heading_i = Cell::new(0usize);
    let li_i = Cell::new(0usize);
    let anchor_i = Cell::new(0usize);

    let settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("[id]", |el: &mut Element| {
            if let Some(id) = el.get_attribute("id")
                && !id.starts_with(ANCHOR_ID_PREFIX)
            {
                el.set_attribute("id", &prefixed(&id))?;
            }
            Ok(())
        }))
        .append_element_content_handler(element!("h1,h2,h3,h4,h5,h6", |el: &mut Element| {
            let i = heading_i.replace(heading_i.get() + 1);
            if el.get_attribute("id").is_none()
                && let Some(Some(id)) = plan.heading_ids.get(i)
            {
                el.set_attribute("id", id)?;
            }
            Ok(())
        }))
        .append_element_content_handler(element!("li", |el: &mut Element| {
            let i = li_i.replace(li_i.get() + 1);
            if el.get_attribute("id").is_none()
                && let Some(Some(id)) = plan.li_ids.get(i)
            {
                el.set_attribute("id", id)?;
            }
            Ok(())
        }))
        .append_element_content_handler(element!("a[href]", |el: &mut Element| {
            let i = anchor_i.replace(anchor_i.get() + 1);
            if plan.dead_anchors.get(i).copied().unwrap_or(false) {
                el.remove_attribute("href");
                return Ok(());
            }
            if let Some(href) = el.get_attribute("href")
                && let Some(fragment) = href.strip_prefix('#')
                && !fragment.starts_with(ANCHOR_ID_PREFIX)
            {
                el.set_attribute("href", &format!("#{}", prefixed(fragment)))?;
            }
            if el.get_attribute("id").is_none()
                && let Some(Some(id)) = plan.anchor_ids.get(i)
            {
                el.set_attribute("id", id)?;
            }
            Ok(())
        }));

    rewrite_str(html, settings).map_err(|err| PrepareError::Rewrite(err.to_string()))
}

fn prefixed(id: &str) -> String {
    if id.starts_with(ANCHOR_ID_PREFIX) {
        id.to_string()
    } else {
        format!("{ANCHOR_ID_PREFIX}{id}")
    }
}

fn is_heading(name: &str) -> bool {
    matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
}

/// `fnref:1-2` → `fnref:1`; a non-numeric suffix is part of the footnote name and kept.
fn occurrence_base(name: &str) -> &str {
    match name.rfind('-') {
        Some(pos)
            if pos > 0
                && !name[pos + 1..].is_empty()
                && name[pos + 1..].chars().all(|c| c.is_ascii_digit()) =>
        {
            &name[..pos]
        }
        _ => name,
    }
}

/// Accepts both raw and `ind-`-prefixed fragments so re-preparation sees the same graph.
fn fragment_name(href: &str, marker: &str) -> Option<String> {
    let fragment = href.strip_prefix('#')?;
    let fragment = fragment.strip_prefix(ANCHOR_ID_PREFIX).unwrap_or(fragment);
    fragment.strip_prefix(marker).map(str::to_string)
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;
    for c in text.chars() {
        if c.is_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            for lower in c.to_lowercase() {
                slug.push(lower);
            }
        } else {
            pending_dash = true;
        }
    }
    slug
}

fn claim_unique(base: String, used: &mut HashSet<String>) -> String {
    if !used.contains(&base) {
        used.insert(base.clone());
        return base;
    }
    let mut n = 2usize;
    loop {
        let candidate = format!("{base}-{n}");
        if !used.contains(&candidate) {
            used.insert(candidate.clone());
            return candidate;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn occurrence_base_strips_only_numeric_suffixes() {
        assert_eq!(occurrence_base("1-2"), "1");
        assert_eq!(occurrence_base("1"), "1");
        assert_eq!(occurrence_base("note-two"), "note-two");
        assert_eq!(occurrence_base("note-2"), "note");
    }

    #[test]
    fn slugify_joins_alphanumeric_runs() {
        assert_eq!(slugify("The Ilari"), "the-ilari");
        assert_eq!(
            slugify("  Imperial period (1608–1800) "),
            "imperial-period-1608-1800"
        );
        assert_eq!(slugify("!!!"), "");
    }

    #[test]
    fn claim_unique_suffixes_from_two() {
        let mut used = HashSet::from(["a".to_string(), "a-2".to_string()]);
        assert_eq!(claim_unique("a".into(), &mut used), "a-3");
        assert_eq!(claim_unique("b".into(), &mut used), "b");
    }
}
