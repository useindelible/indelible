use std::io::Cursor;

use base64::Engine;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use zip::ZipArchive;

use super::io::read_zip_bytes;
use super::paths::resolve_path_from_chapter;
use super::xml::local_name;
use crate::archive_limits::ArchiveReadBudget;

const ALLOWED_TAGS: &[&[u8]] = &[
    b"p",
    b"br",
    b"hr",
    b"div",
    b"span",
    b"section",
    b"article",
    b"aside",
    b"header",
    b"footer",
    b"nav",
    b"main",
    b"figure",
    b"figcaption",
    b"details",
    b"summary",
    b"h1",
    b"h2",
    b"h3",
    b"h4",
    b"h5",
    b"h6",
    b"a",
    b"em",
    b"strong",
    b"b",
    b"i",
    b"u",
    b"s",
    b"sub",
    b"sup",
    b"small",
    b"mark",
    b"abbr",
    b"cite",
    b"code",
    b"kbd",
    b"samp",
    b"var",
    b"time",
    b"dfn",
    b"q",
    b"blockquote",
    b"pre",
    b"ul",
    b"ol",
    b"li",
    b"dl",
    b"dt",
    b"dd",
    b"table",
    b"thead",
    b"tbody",
    b"tfoot",
    b"tr",
    b"th",
    b"td",
    b"caption",
    b"colgroup",
    b"col",
    b"img",
    b"picture",
    b"source",
    b"math",
    b"ruby",
    b"rt",
    b"rp",
];

fn is_allowed_tag(local: &[u8]) -> bool {
    ALLOWED_TAGS.iter().any(|t| t.eq_ignore_ascii_case(local))
}

/// Attributes preserved on allowed tags. `style` is deliberately excluded:
/// inline CSS enables overlay/clickjacking tricks, `url()` tracking fetches, and
/// legacy script vectors, and the reader applies its own typography regardless.
fn is_allowed_attr(key: &str) -> bool {
    matches!(
        key,
        "class"
            | "id"
            | "name"
            | "lang"
            | "dir"
            | "role"
            | "aria-label"
            | "alt"
            | "title"
            | "href"
            | "src"
            | "colspan"
            | "rowspan"
    )
}

/// Largest image inlined as a `data:` URI. Bounds the DOM bloat any single
/// image (e.g. an oversized SVG) can add to a chapter, below the whole-archive
/// decompression budget. Larger images are dropped rather than embedded.
const INLINE_IMAGE_MAX_BYTES: usize = 8 * 1024 * 1024;

pub(super) fn sanitize_chapter_html(
    html: &str,
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    opf_dir: &str,
    chapter_href: &str,
    budget: &mut ArchiveReadBudget,
) -> String {
    let chapter_dir = chapter_href
        .rfind('/')
        .map(|i| &chapter_href[..i + 1])
        .unwrap_or("");

    let mut result = String::with_capacity(html.len());
    let mut reader = Reader::from_str(html);
    reader.config_mut().check_end_names = false;
    let mut buf = Vec::new();
    let mut skip_depth: u32 = 0;
    let mut in_body = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"body" {
                    in_body = true;
                } else if skip_depth > 0 {
                    skip_depth += 1;
                } else if local == b"script"
                    || local == b"style"
                    || local == b"iframe"
                    || local == b"object"
                    || local == b"embed"
                    || local == b"form"
                    || local == b"input"
                    || local == b"textarea"
                    || local == b"button"
                    || local == b"select"
                    || local == b"link"
                    || local == b"svg"
                {
                    // svg can carry active content (scripts, foreignObject, event
                    // handlers, use/animate); drop the whole subtree.
                    skip_depth = 1;
                } else if in_body && is_allowed_tag(local) {
                    let tag_name = String::from_utf8_lossy(local).into_owned();
                    result.push('<');
                    result.push_str(&tag_name);

                    for attr in e.attributes().flatten() {
                        let raw_key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
                        // Normalize xml:id to id so fragment anchors work
                        let key = if raw_key == "xml:id" {
                            "id".to_string()
                        } else {
                            raw_key
                        };
                        let val = String::from_utf8_lossy(&attr.value).into_owned();

                        if (key == "src" || key == "xlink:href")
                            && let Some(data_uri) = resolve_image_as_data_uri(
                                archive,
                                opf_dir,
                                chapter_dir,
                                &val,
                                budget,
                            )
                        {
                            result.push(' ');
                            result.push_str(&key);
                            result.push_str("=\"");
                            result.push_str(&data_uri);
                            result.push('"');
                            continue;
                        }

                        let is_url_attr = key == "href" || key == "src";
                        if is_allowed_attr(&key) && (!is_url_attr || is_safe_uri(&val, &key)) {
                            result.push(' ');
                            result.push_str(&key);
                            result.push_str("=\"");
                            result.push_str(&escape_attr(&val));
                            result.push('"');
                        }
                    }
                    result.push('>');
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"body" {
                    in_body = false;
                } else if skip_depth > 0 {
                    skip_depth -= 1;
                } else if in_body && is_allowed_tag(local) {
                    let tag_name = String::from_utf8_lossy(local).into_owned();
                    result.push_str("</");
                    result.push_str(&tag_name);
                    result.push('>');
                }
            }
            Ok(Event::Empty(ref e)) => {
                if skip_depth > 0 || !in_body {
                    // skip
                } else {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    if !is_allowed_tag(local) {
                        // skip disallowed self-closing tags
                    } else {
                        let tag_name = String::from_utf8_lossy(local).into_owned();
                        result.push('<');
                        result.push_str(&tag_name);

                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
                            let val = String::from_utf8_lossy(&attr.value).into_owned();

                            if (key == "src" || key == "xlink:href")
                                && let Some(data_uri) = resolve_image_as_data_uri(
                                    archive,
                                    opf_dir,
                                    chapter_dir,
                                    &val,
                                    budget,
                                )
                            {
                                result.push(' ');
                                result.push_str(&key);
                                result.push_str("=\"");
                                result.push_str(&data_uri);
                                result.push('"');
                                continue;
                            }

                            let is_url_attr = key == "href" || key == "src";
                            if is_allowed_attr(&key) && (!is_url_attr || is_safe_uri(&val, &key)) {
                                result.push(' ');
                                result.push_str(&key);
                                result.push_str("=\"");
                                result.push_str(&escape_attr(&val));
                                result.push('"');
                            }
                        }
                        result.push_str(" />");
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if skip_depth == 0
                    && in_body
                    && let Ok(t) = e.unescape()
                {
                    result.push_str(&escape_html(&t));
                }
            }
            Ok(Event::CData(ref e)) => {
                if skip_depth == 0 && in_body {
                    let text = String::from_utf8_lossy(e.as_ref());
                    result.push_str(&escape_html(&text));
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    result
}

fn resolve_image_as_data_uri(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    opf_dir: &str,
    chapter_dir: &str,
    href: &str,
    budget: &mut ArchiveReadBudget,
) -> Option<String> {
    if href.starts_with("data:") || href.starts_with("http://") || href.starts_with("https://") {
        return None;
    }

    let resolved = resolve_path_from_chapter(opf_dir, chapter_dir, href);
    // Do not inline SVG: it is the one raster-image format that can carry active
    // content. Dropping the src (returning None) is safer than embedding an
    // unsanitized image/svg+xml data URI.
    if resolved.to_ascii_lowercase().ends_with(".svg") {
        return None;
    }
    let bytes = read_zip_bytes(archive, &resolved, budget)?;
    if bytes.len() > INLINE_IMAGE_MAX_BYTES {
        return None;
    }

    let mime = guess_image_mime(&resolved);
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{};base64,{}", mime, encoded))
}
fn guess_image_mime(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else {
        "application/octet-stream"
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn is_safe_uri(val: &str, attr_name: &str) -> bool {
    let trimmed = val.trim().to_lowercase();
    if trimmed.is_empty() {
        return true;
    }
    // Block dangerous URI schemes.
    if trimmed.starts_with("javascript:")
        || trimmed.starts_with("vbscript:")
        || trimmed.starts_with("data:text/html")
    {
        return false;
    }
    // Never let SVG through a url attribute: inline `<svg>` is dropped by the tag
    // allowlist and `.svg` files are not inlined, but an SVG could still be
    // referenced directly via `data:image/svg+xml` or a `.svg`/`.svgz` URL. SVG
    // is the one image format that can carry active content, so reject it here
    // (the attribute is then dropped rather than preserved).
    if trimmed.starts_with("data:image/svg") || url_path_has_svg_extension(&trimmed) {
        return false;
    }
    // For src attributes (which auto-load resources), block remote URLs.
    // Images are already inlined as data: URIs during processing;
    // any remaining http(s) src values are off-origin resource loads.
    if attr_name == "src" && (trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return false;
    }
    true
}

fn url_path_has_svg_extension(trimmed_lower: &str) -> bool {
    let path = trimmed_lower
        .split(['?', '#'])
        .next()
        .unwrap_or(trimmed_lower);
    path.ends_with(".svg") || path.ends_with(".svgz")
}
