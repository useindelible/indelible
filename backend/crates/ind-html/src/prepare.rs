//! Reader-HTML preparation: sanitize untrusted article HTML and make its anchors
//! durable, so a stored document supports table-of-contents navigation and working
//! in-document links on every client.
//!
//! Every retained `id` gets the `ind-` prefix and every local fragment href is
//! rewritten to match, because a fragment link is only as good as its target.
//! Headings without ids get slugified `ind-toc-*` ids; footnote list items and
//! citation anchors get ids inferred from the `#fn:`/`#fnref:` link graph that
//! readability extractors emit (legacy stored content lost the original targets
//! to sanitization, leaving every footnote link dead).
//!
//! The output is byte-for-byte idempotent: anything already prefixed is never
//! touched again, which is what lets re-preparation double as a cheap
//! "is this content already prepared" check.

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

/// Sanitize untrusted article HTML and inject durable anchors. Replaces
/// `sanitize_reader_html` at content write paths; callers fall back to plain
/// sanitization on error rather than failing ingest.
pub fn prepare_reader_html(html: &str) -> Result<String, PrepareError> {
    let sanitized = sanitize_keeping_ids(html);
    let plan = build_rewrite_plan(&sanitized);
    let rewritten = apply(&plan, &sanitized)?;
    // Ammonia serializes attributes in canonical order while lol_html appends
    // injected ones at the tag end; a final sanitize pass canonicalizes the
    // output so preparation is byte-for-byte idempotent.
    Ok(sanitize_keeping_ids(&rewritten))
}

/// Ammonia's vetted default allowlist, plus `id` — the defaults strip it, which is
/// exactly what orphaned every fragment link in previously stored content. Ids are
/// safe to retain because the apply pass namespaces every one of them with
/// [`ANCHOR_ID_PREFIX`], which also prevents DOM clobbering of host-page globals.
fn sanitize_keeping_ids(html: &str) -> String {
    ammonia::Builder::default()
        .add_generic_attributes(&["id"])
        .clean(html)
        .to_string()
}

/// Ids to inject, addressed by element ordinal so the streaming apply pass can
/// assign them without re-parsing. Ordinals count, per element type, exactly the
/// elements the apply pass's selectors visit.
struct RewritePlan {
    /// By `h1..h6` ordinal; `Some` only for headings that need an injected id.
    heading_ids: Vec<Option<String>>,
    /// By `li` ordinal; `Some` for footnote items identified via their backlinks.
    li_ids: Vec<Option<String>>,
    /// By `a[href]` ordinal; `Some` for citation anchors that footnote backlinks
    /// point back at.
    anchor_ids: Vec<Option<String>>,
    /// By `a[href]` ordinal; `true` when the link's local fragment has no target
    /// anywhere in the final document (extraction removed it). The href is
    /// stripped so the link degrades to inert text instead of a broken jump.
    dead_anchors: Vec<bool>,
}

fn build_rewrite_plan(sanitized: &str) -> RewritePlan {
    let doc = Html::parse_document(sanitized);
    let elements: Vec<ElementRef<'_>> = doc
        .root_element()
        .descendants()
        .filter_map(ElementRef::wrap)
        .collect();

    // The collision namespace is the FINAL id set: existing ids as they will look
    // after prefixing, plus everything assigned below.
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

    // `used` now holds the complete final id namespace; any local fragment
    // without a member there can never resolve.
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

/// A footnote item is recognized by the backlink(s) it contains: readability
/// output puts `<a href="#fnref:N">` (or `#fnref:N-k` for repeat citations)
/// inside `<li>` N of the footnote list, so the base name identifies the item.
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

/// Citation anchors (`<a href="#fn:N">`) receive the ids the footnote backlinks
/// already reference: the first citation of footnote N is `fnref:N`, the k-th is
/// `fnref:N-k` — the convention readability extractors use when generating the
/// backlinks, so the two sides meet.
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
        // Prefix every retained id, whatever the element: fragment hrefs are
        // rewritten globally below, so their targets must move with them.
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

/// `#fnref:1-2` names occurrence 2 of footnote 1; the base identifies the footnote.
/// The suffix is stripped only when purely numeric so footnotes whose own names
/// contain dashes are left intact.
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

/// The name of a `#`-fragment href after `marker`, accepting both raw and
/// already-prefixed forms so re-preparation sees the same graph.
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
