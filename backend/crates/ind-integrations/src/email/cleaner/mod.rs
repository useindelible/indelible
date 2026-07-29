// Pre-Readability email cleaner. Strips noise that confuses Readability's
// scoring heuristics: tracking pixels, footer boilerplate (unsubscribe,
// view-in-browser, manage preferences), CSS-only layout tables, <style>/<meta>
// blocks, and mailto:/tracker links.
//
// Output is not sanitized — it still contains user HTML. Run ammonia or
// Readability after this for final safety.

pub fn clean_email_html(html: &str) -> String {
    let stripped_script = strip_raw_tag(html, "script");
    let stripped_noscript = strip_raw_tag(&stripped_script, "noscript");
    let stripped_style = strip_raw_tag(&stripped_noscript, "style");
    let stripped_meta = strip_void_tag(&stripped_style, "meta");
    let stripped_pixels = strip_imgs_where(&stripped_meta, is_tracking_pixel);
    let unwrapped = unwrap_anchors_where(&stripped_pixels, is_noise_link);
    strip_noise_blocks(&unwrapped)
}

/// Full reader-mode pipeline: cleaner removes email scaffolding, then ammonia
/// strips scripts, event handlers, `javascript:` URLs, and other XSS-prone
/// markup. Output is safe to render inside the app.
pub fn prepare_email_for_reader(html: &str) -> String {
    let cleaned = clean_email_html(html);
    ammonia::clean(&cleaned)
}

#[cfg(test)]
mod tests;

const FOOTER_TEXT_MARKERS: &[&str] = &[
    "unsubscribe",
    "view in browser",
    "view this email in your browser",
    "manage preferences",
    "update your email preferences",
    "update your preferences",
    "email preferences",
];

const FOOTER_BLOCK_TAGS: &[&str] = &["div", "table", "section", "footer", "aside"];

fn strip_noise_blocks(html: &str) -> String {
    let mut current = html.to_string();
    for tag in FOOTER_BLOCK_TAGS {
        current = strip_noise_blocks_for_tag(&current, tag);
    }
    current
}

fn strip_noise_blocks_for_tag(html: &str, tag: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let open_prefix = format!("<{tag}");

    let mut out = String::with_capacity(html.len());
    let mut cursor = 0;
    while cursor < html.len() {
        let Some(rel) = lower[cursor..].find(&open_prefix) else {
            out.push_str(&html[cursor..]);
            break;
        };
        let open_at = cursor + rel;
        let after_idx = open_at + open_prefix.len();
        let after = lower.as_bytes().get(after_idx).copied();
        let is_tag_boundary = matches!(
            after,
            Some(b' ' | b'>' | b'/' | b'\t' | b'\n' | b'\r') | None
        );
        if !is_tag_boundary {
            out.push_str(&html[cursor..=open_at]);
            cursor = open_at + 1;
            continue;
        }
        let Some(open_end_rel) = html[open_at..].find('>') else {
            out.push_str(&html[cursor..]);
            break;
        };
        let open_tag_end = open_at + open_end_rel + 1;
        // self-closing variant (`<div ... />`) has no inner content to inspect.
        if html[open_at..open_tag_end]
            .trim_end_matches('>')
            .ends_with('/')
        {
            out.push_str(&html[cursor..open_tag_end]);
            cursor = open_tag_end;
            continue;
        }
        match find_matching_close(&lower, tag, open_tag_end) {
            Some(close_at) => {
                let close_end = close_at + format!("</{tag}>").len();
                let inner = &html[open_tag_end..close_at];
                if block_text_contains_noise(inner) && !block_contains_article_content(inner) {
                    out.push_str(&html[cursor..open_at]);
                    cursor = close_end;
                } else {
                    out.push_str(&html[cursor..open_tag_end]);
                    cursor = open_tag_end;
                }
            }
            None => {
                out.push_str(&html[cursor..open_tag_end]);
                cursor = open_tag_end;
            }
        }
    }
    out
}

fn find_matching_close(lower: &str, tag: &str, start: usize) -> Option<usize> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut depth = 1usize;
    let mut cursor = start;
    while cursor < lower.len() {
        let open_rel = lower[cursor..].find(&open);
        let close_rel = lower[cursor..].find(&close);
        match (open_rel, close_rel) {
            (_, None) => return None,
            (None, Some(c_rel)) => {
                let c_abs = cursor + c_rel;
                depth -= 1;
                if depth == 0 {
                    return Some(c_abs);
                }
                cursor = c_abs + close.len();
            }
            (Some(o_rel), Some(c_rel)) => {
                if o_rel < c_rel {
                    let o_abs = cursor + o_rel;
                    let after = lower.as_bytes().get(o_abs + open.len()).copied();
                    let is_boundary = matches!(
                        after,
                        Some(b' ' | b'>' | b'/' | b'\t' | b'\n' | b'\r') | None
                    );
                    cursor = o_abs + open.len();
                    if is_boundary {
                        depth += 1;
                    }
                } else {
                    let c_abs = cursor + c_rel;
                    depth -= 1;
                    if depth == 0 {
                        return Some(c_abs);
                    }
                    cursor = c_abs + close.len();
                }
            }
        }
    }
    None
}

fn block_text_contains_noise(inner_html: &str) -> bool {
    let text = ind_html::html_to_text(inner_html).to_ascii_lowercase();
    FOOTER_TEXT_MARKERS.iter().any(|m| text.contains(m))
}

const ARTICLE_CONTENT_TAGS: &[&str] = &["<h1", "<h2", "<h3", "<article", "<main"];

/// Returns true when the block contains an element that signals real article
/// content. Used to keep outer wrappers (e.g. `<table>` around the whole email
/// body) intact even when their inner contains a footer with noise markers.
fn block_contains_article_content(inner_html: &str) -> bool {
    let lower = inner_html.to_ascii_lowercase();
    ARTICLE_CONTENT_TAGS.iter().any(|needle| {
        lower.match_indices(needle).any(|(idx, _)| {
            lower
                .as_bytes()
                .get(idx + needle.len())
                .copied()
                .is_some_and(|b| matches!(b, b' ' | b'>' | b'/' | b'\t' | b'\n' | b'\r'))
        })
    })
}

fn is_noise_link(attrs: &TagAttrs) -> bool {
    let Some(href) = attrs.get("href") else {
        return false;
    };
    let h = href.trim().to_ascii_lowercase();
    if h.starts_with("mailto:") {
        return true;
    }
    if let Some(rest) = h
        .strip_prefix("https://")
        .or_else(|| h.strip_prefix("http://"))
    {
        let host = rest.split(['/', '?', '#']).next().unwrap_or("");
        if host.contains(".trk.") || host.starts_with("trk.") || host.contains(".trk:") {
            return true;
        }
    }
    false
}

/// Removes `<a ...>...</a>` opening and closing tags where `predicate(attrs)`
/// holds, preserving the inner content as text. Non-matching anchors are
/// left intact.
fn unwrap_anchors_where(html: &str, predicate: fn(&TagAttrs) -> bool) -> String {
    let lower = html.to_ascii_lowercase();
    let open_prefix = "<a";
    let close_tag = "</a>";

    let mut out = String::with_capacity(html.len());
    let mut cursor = 0;
    while cursor < html.len() {
        let Some(rel) = lower[cursor..].find(open_prefix) else {
            out.push_str(&html[cursor..]);
            break;
        };
        let open_at = cursor + rel;
        let after_idx = open_at + open_prefix.len();
        let after = lower.as_bytes().get(after_idx).copied();
        let is_tag_boundary = matches!(
            after,
            Some(b' ' | b'>' | b'/' | b'\t' | b'\n' | b'\r') | None
        );
        if !is_tag_boundary {
            out.push_str(&html[cursor..=open_at]);
            cursor = open_at + 1;
            continue;
        }
        let Some(open_end_rel) = html[open_at..].find('>') else {
            out.push_str(&html[cursor..]);
            break;
        };
        let open_tag_end = open_at + open_end_rel + 1;
        let open_tag = &html[open_at..open_tag_end];
        let attrs = TagAttrs::parse(open_tag);
        if predicate(&attrs) {
            // Find the matching </a> (no nested <a> in HTML5).
            let close_rel = lower[open_tag_end..].find(close_tag);
            match close_rel {
                Some(rel) => {
                    out.push_str(&html[cursor..open_at]);
                    let inner_end = open_tag_end + rel;
                    out.push_str(&html[open_tag_end..inner_end]);
                    cursor = inner_end + close_tag.len();
                }
                None => {
                    out.push_str(&html[cursor..open_at]);
                    out.push_str(&html[open_tag_end..]);
                    break;
                }
            }
        } else {
            out.push_str(&html[cursor..open_tag_end]);
            cursor = open_tag_end;
        }
    }
    out
}

fn is_tracking_pixel(attrs: &TagAttrs) -> bool {
    let w = attrs.get("width").map(str::trim);
    let h = attrs.get("height").map(str::trim);
    let pixel_dim = |v: Option<&str>| matches!(v, Some("0" | "1"));
    if pixel_dim(w) && pixel_dim(h) {
        return true;
    }
    if let Some(style) = attrs.get("style") {
        let s = style.to_ascii_lowercase();
        if s.contains("display:none") || s.contains("display: none") {
            return true;
        }
        if (s.contains("width:1px") || s.contains("width: 1px"))
            && (s.contains("height:1px") || s.contains("height: 1px"))
        {
            return true;
        }
    }
    if let Some(src) = attrs.get("src") {
        let s = src.to_ascii_lowercase();
        const TRACKER_PATH_NEEDLES: &[&str] =
            &["/open/", "/open?", "/track/", "/pixel/", "/beacon/"];
        if TRACKER_PATH_NEEDLES.iter().any(|n| s.contains(n)) {
            return true;
        }
    }
    false
}

/// Strips `<img ...>` elements where `predicate(attrs)` returns true.
fn strip_imgs_where(html: &str, predicate: fn(&TagAttrs) -> bool) -> String {
    let lower = html.to_ascii_lowercase();
    let open_prefix = "<img";

    let mut out = String::with_capacity(html.len());
    let mut cursor = 0;
    while cursor < html.len() {
        let Some(rel) = lower[cursor..].find(open_prefix) else {
            out.push_str(&html[cursor..]);
            break;
        };
        let open_at = cursor + rel;
        let after_idx = open_at + open_prefix.len();
        let after = lower.as_bytes().get(after_idx).copied();
        let is_tag_boundary = matches!(
            after,
            Some(b' ' | b'>' | b'/' | b'\t' | b'\n' | b'\r') | None
        );
        if !is_tag_boundary {
            out.push_str(&html[cursor..=open_at]);
            cursor = open_at + 1;
            continue;
        }
        let Some(end_rel) = html[open_at..].find('>') else {
            out.push_str(&html[cursor..]);
            break;
        };
        let tag_end = open_at + end_rel + 1;
        let tag = &html[open_at..tag_end];
        let attrs = TagAttrs::parse(tag);
        if predicate(&attrs) {
            out.push_str(&html[cursor..open_at]);
            cursor = tag_end;
        } else {
            out.push_str(&html[cursor..tag_end]);
            cursor = tag_end;
        }
    }
    out
}

/// Minimal HTML attribute parser. Handles `name="value"`, `name='value'`,
/// `name=value`, and boolean `name`. Not a full HTML5 tokenizer — sufficient
/// for matching `width`/`height`/`href`/`src` on cleaner predicates.
struct TagAttrs<'a> {
    attrs: Vec<(String, &'a str)>,
}

impl<'a> TagAttrs<'a> {
    fn parse(tag: &'a str) -> Self {
        let inner = tag
            .strip_prefix('<')
            .and_then(|t| t.strip_suffix('>'))
            .unwrap_or(tag);
        let after_name = match inner.find(|c: char| c.is_ascii_whitespace()) {
            Some(idx) => &inner[idx..],
            None => return Self { attrs: Vec::new() },
        };
        let bytes = after_name.as_bytes();
        let mut attrs = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() || bytes[i] == b'/' {
                break;
            }
            let name_start = i;
            while i < bytes.len()
                && !bytes[i].is_ascii_whitespace()
                && bytes[i] != b'='
                && bytes[i] != b'/'
            {
                i += 1;
            }
            let name = after_name[name_start..i].to_ascii_lowercase();
            if name.is_empty() {
                i += 1;
                continue;
            }
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b'=' {
                i += 1;
                while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                let value = if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                    let quote = bytes[i];
                    i += 1;
                    let v_start = i;
                    while i < bytes.len() && bytes[i] != quote {
                        i += 1;
                    }
                    let v = &after_name[v_start..i];
                    if i < bytes.len() {
                        i += 1;
                    }
                    v
                } else {
                    let v_start = i;
                    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'/' {
                        i += 1;
                    }
                    &after_name[v_start..i]
                };
                attrs.push((name, value));
            } else {
                attrs.push((name, ""));
            }
        }
        Self { attrs }
    }

    fn get(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(n, _)| n == name).map(|(_, v)| *v)
    }
}

/// Strips `<tag ...>` (the entire opening tag and its attributes) for void
/// elements that have no closing tag. Case-insensitive on the tag name.
fn strip_void_tag(html: &str, tag: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let open_prefix = format!("<{tag}");

    let mut out = String::with_capacity(html.len());
    let mut cursor = 0;
    while cursor < html.len() {
        let Some(rel) = lower[cursor..].find(&open_prefix) else {
            out.push_str(&html[cursor..]);
            break;
        };
        let open_at = cursor + rel;
        let after_idx = open_at + open_prefix.len();
        let after = lower.as_bytes().get(after_idx).copied();
        let is_tag_boundary = matches!(
            after,
            Some(b' ' | b'>' | b'/' | b'\t' | b'\n' | b'\r') | None
        );
        if !is_tag_boundary {
            out.push_str(&html[cursor..=open_at]);
            cursor = open_at + 1;
            continue;
        }
        out.push_str(&html[cursor..open_at]);
        match html[open_at..].find('>') {
            Some(end_rel) => cursor = open_at + end_rel + 1,
            None => break,
        }
    }
    out
}

/// Strips `<tag ...>...</tag>` and everything between, case-insensitively.
/// `tag` must be ASCII lowercase. The match is tag-aware (it won't strip
/// `<styled>` when asked for `style`).
fn strip_raw_tag(html: &str, tag: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let open_prefix = format!("<{tag}");
    let close = format!("</{tag}>");

    let mut out = String::with_capacity(html.len());
    let mut cursor = 0;
    while cursor < html.len() {
        let Some(rel) = lower[cursor..].find(&open_prefix) else {
            out.push_str(&html[cursor..]);
            break;
        };
        let open_at = cursor + rel;
        let after_idx = open_at + open_prefix.len();
        let after = lower.as_bytes().get(after_idx).copied();
        let is_tag_boundary = matches!(
            after,
            Some(b' ' | b'>' | b'/' | b'\t' | b'\n' | b'\r') | None
        );
        if !is_tag_boundary {
            // false positive (e.g. `<styled`) — keep up to here, advance past `<`.
            out.push_str(&html[cursor..=open_at]);
            cursor = open_at + 1;
            continue;
        }
        out.push_str(&html[cursor..open_at]);
        match lower[open_at..].find(&close) {
            Some(close_rel) => cursor = open_at + close_rel + close.len(),
            None => break,
        }
    }
    out
}
