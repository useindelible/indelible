use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::types::{ManifestItem, ParsedOpf, SpineEntry};
use super::xml::local_name;

pub(super) fn parse_rootfile_path(container_xml: &str) -> Option<String> {
    let mut reader = Reader::from_str(container_xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) if e.name().as_ref() == b"rootfile" => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"full-path" {
                        return String::from_utf8(attr.value.to_vec()).ok();
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

pub(super) fn parse_opf(opf_xml: &str) -> ParsedOpf {
    let mut reader = Reader::from_str(opf_xml);
    let mut buf = Vec::new();

    let mut manifest = Vec::new();
    let mut spine = Vec::new();
    let mut title = None;
    let mut author = None;
    let mut publisher = None;
    let mut language = None;
    let mut isbn = None;

    let mut in_metadata = false;
    let mut current_tag: Option<String> = None;
    let mut current_text = String::new();
    let mut current_identifier_scheme: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                match local {
                    b"metadata" => in_metadata = true,
                    b"title" | b"creator" | b"publisher" | b"language" | b"identifier"
                        if in_metadata =>
                    {
                        current_tag = Some(String::from_utf8_lossy(local).into_owned());
                        current_text.clear();
                        if local == b"identifier" {
                            current_identifier_scheme = e
                                .attributes()
                                .flatten()
                                .find(|a| {
                                    let key = a.key.as_ref();
                                    key == b"opf:scheme" || key.ends_with(b":scheme")
                                })
                                .and_then(|a| String::from_utf8(a.value.to_vec()).ok());
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"metadata" {
                    in_metadata = false;
                }
                if let Some(ref tag) = current_tag {
                    let trimmed = current_text.trim().to_string();
                    if !trimmed.is_empty() {
                        match tag.as_str() {
                            "title" if title.is_none() => title = Some(trimmed),
                            "creator" if author.is_none() => author = Some(trimmed),
                            "publisher" if publisher.is_none() => publisher = Some(trimmed),
                            "language" if language.is_none() => language = Some(trimmed),
                            "identifier" => {
                                let scheme = current_identifier_scheme
                                    .take()
                                    .unwrap_or_default()
                                    .to_lowercase();
                                if scheme.contains("isbn") || looks_like_isbn(&trimmed) {
                                    isbn = Some(trimmed);
                                }
                            }
                            _ => {}
                        }
                    }
                    current_tag = None;
                    current_text.clear();
                }
            }
            Ok(Event::Text(e)) => {
                if current_tag.is_some()
                    && let Ok(t) = e.unescape()
                {
                    current_text.push_str(&t);
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"item" {
                    let mut id = String::new();
                    let mut href = String::new();
                    let mut media_type = String::new();
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"id" => {
                                id = String::from_utf8_lossy(&attr.value).into_owned();
                            }
                            b"href" => {
                                href = String::from_utf8_lossy(&attr.value).into_owned();
                            }
                            b"media-type" => {
                                media_type = String::from_utf8_lossy(&attr.value).into_owned();
                            }
                            _ => {}
                        }
                    }
                    if !id.is_empty() {
                        manifest.push(ManifestItem {
                            id,
                            href,
                            media_type,
                        });
                    }
                } else if local == b"itemref" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"idref" {
                            spine.push(SpineEntry {
                                idref: String::from_utf8_lossy(&attr.value).into_owned(),
                            });
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    ParsedOpf {
        manifest,
        spine,
        title,
        author,
        publisher,
        language,
        isbn,
    }
}

pub(super) fn looks_like_isbn(s: &str) -> bool {
    let digits: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-' || *c == 'X' || *c == 'x')
        .collect();
    let pure_digits: String = digits
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == 'X' || *c == 'x')
        .collect();
    pure_digits.len() == 10 || pure_digits.len() == 13
}
