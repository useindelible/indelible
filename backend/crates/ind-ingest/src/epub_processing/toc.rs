use std::collections::HashMap;
use std::io::Cursor;

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use zip::ZipArchive;

use super::io::read_zip_text;
use super::paths::resolve_path;
use super::types::{ManifestItem, NavPoint, ParsedOpf};
use super::xml::local_name;
use crate::archive_limits::ArchiveReadBudget;

pub(super) fn extract_toc(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    opf: &ParsedOpf,
    opf_dir: &str,
    manifest_by_id: &HashMap<&str, &ManifestItem>,
    budget: &mut ArchiveReadBudget,
) -> Vec<NavPoint> {
    // Try EPUB3 nav document first
    let nav_item = opf.manifest.iter().find(|m| {
        m.media_type.contains("html")
            && opf
                .manifest
                .iter()
                .any(|_| m.id.contains("nav") || m.href.contains("nav"))
    });

    if let Some(nav) = nav_item {
        let nav_path = resolve_path(opf_dir, &nav.href);
        if let Some(nav_html) = read_zip_text(archive, &nav_path, budget) {
            let points = parse_nav_html(&nav_html);
            if !points.is_empty() {
                return points;
            }
        }
    }

    // Fall back to NCX (EPUB2)
    let ncx_item = opf
        .manifest
        .iter()
        .find(|m| m.media_type == "application/x-dtbncx+xml");

    if let Some(ncx) = ncx_item {
        let ncx_path = resolve_path(opf_dir, &ncx.href);
        if let Some(ncx_xml) = read_zip_text(archive, &ncx_path, budget) {
            return parse_ncx(&ncx_xml);
        }
    }

    // Check spine toc attribute
    for item in manifest_by_id.values() {
        if item.media_type == "application/x-dtbncx+xml" {
            let ncx_path = resolve_path(opf_dir, &item.href);
            if let Some(ncx_xml) = read_zip_text(archive, &ncx_path, budget) {
                return parse_ncx(&ncx_xml);
            }
        }
    }

    Vec::new()
}

fn parse_ncx(ncx_xml: &str) -> Vec<NavPoint> {
    let mut reader = Reader::from_str(ncx_xml);
    let mut buf = Vec::new();
    let mut points = Vec::new();
    // Stack of (label, depth) — one frame per open navPoint.
    // Emit eagerly on <content> to preserve document order (parents before children).
    let mut stack: Vec<(String, u32)> = Vec::new();
    let mut in_text = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"navPoint" {
                    let depth = stack.len() as u32 + 1;
                    stack.push((String::new(), depth));
                } else if local == b"text" && !stack.is_empty() {
                    in_text = true;
                }
            }
            Ok(Event::End(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"navPoint" {
                    stack.pop();
                } else if local == b"text" {
                    in_text = false;
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"content" {
                    let frame = stack.last().map(|(l, d)| (l.clone(), *d));
                    if let Some((label, depth)) = frame
                        && !label.is_empty()
                    {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"src" {
                                let src = String::from_utf8_lossy(&attr.value).into_owned();
                                points.push(NavPoint {
                                    label,
                                    content_src: src,
                                    depth,
                                });
                                break;
                            }
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if in_text
                    && let Some((label, _)) = stack.last_mut()
                    && let Ok(t) = e.unescape()
                {
                    label.push_str(&t);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    points
}

fn parse_nav_html(nav_html: &str) -> Vec<NavPoint> {
    let mut points = Vec::new();
    // Simple extraction: find <a href="...">text</a> within <nav> <ol> <li> structure
    // This is a lightweight approach that handles most EPUB3 nav documents
    let mut reader = Reader::from_str(nav_html);
    let mut buf = Vec::new();
    let mut in_nav = false;
    let mut li_depth: u32 = 0;
    let mut in_a = false;
    let mut current_href = String::new();
    let mut current_text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"nav" {
                    let is_toc = e.attributes().flatten().any(|a| {
                        a.key.as_ref() == b"epub:type"
                            && String::from_utf8_lossy(&a.value).contains("toc")
                    });
                    if is_toc {
                        in_nav = true;
                    }
                } else if local == b"li" && in_nav {
                    li_depth += 1;
                } else if local == b"a" && in_nav && li_depth > 0 {
                    in_a = true;
                    current_text.clear();
                    current_href.clear();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"href" {
                            current_href = String::from_utf8_lossy(&attr.value).into_owned();
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"nav" && in_nav {
                    in_nav = false;
                } else if local == b"li" && in_nav {
                    li_depth = li_depth.saturating_sub(1);
                } else if local == b"a" && in_a {
                    let label = current_text.trim().to_string();
                    if !label.is_empty() && !current_href.is_empty() {
                        points.push(NavPoint {
                            label,
                            content_src: current_href.clone(),
                            depth: li_depth,
                        });
                    }
                    in_a = false;
                }
            }
            Ok(Event::Text(e)) => {
                if in_a && let Ok(t) = e.unescape() {
                    current_text.push_str(&t);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    points
}
