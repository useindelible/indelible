use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use bytes::Bytes;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::QName;

const THUMBNAIL_CONTENT_TYPE: &str = "image/png";
const MAX_COVER_IMAGE_BYTES: u64 = 10 * 1024 * 1024;
const MAX_COVER_PAGE_BYTES: u64 = 2 * 1024 * 1024;
const COVER_MAX_WIDTH: u32 = 640;
const COVER_MAX_HEIGHT: u32 = 900;

#[derive(Debug, Clone)]
pub struct ExtractedCoverImage {
    pub bytes: Bytes,
    pub content_type: &'static str,
}

pub async fn extract_cover_image_async(
    content_type: &str,
    data: Bytes,
) -> Option<ExtractedCoverImage> {
    let content_type = content_type.to_string();
    match tokio::task::spawn_blocking(move || extract_cover_image(&content_type, &data)).await {
        Ok(cover) => cover,
        Err(err) => {
            tracing::debug!(%err, "cover extraction task failed");
            None
        }
    }
}

fn extract_cover_image(content_type: &str, data: &[u8]) -> Option<ExtractedCoverImage> {
    match content_type {
        "application/pdf" => extract_pdf_cover_image(data),
        "application/epub+zip" => extract_epub_cover_image(data),
        _ => None,
    }
}

fn extract_pdf_cover_image(data: &[u8]) -> Option<ExtractedCoverImage> {
    use pdf_oxide::document::PdfDocument;
    use pdf_oxide::rendering::{RenderOptions, render_page};

    let doc = PdfDocument::from_bytes(data.to_vec()).ok()?;
    let page_count = doc.page_count().ok()?.min(3);

    for page_index in 0..page_count {
        let rendered = match render_page(&doc, page_index, &RenderOptions::with_dpi(96)) {
            Ok(image) => image,
            Err(err) => {
                tracing::debug!(%err, page_index, "PDF page render failed while extracting cover");
                continue;
            }
        };
        if rendered.data.is_empty() || rendered.width == 0 || rendered.height == 0 {
            continue;
        }
        let Some(cover) = image_bytes_to_thumbnail_png(&rendered.data) else {
            continue;
        };
        if cover.bytes.len() <= MAX_COVER_IMAGE_BYTES as usize {
            return Some(cover);
        }
    }
    None
}

fn extract_epub_cover_image(data: &[u8]) -> Option<ExtractedCoverImage> {
    let mut archive = zip::ZipArchive::new(Cursor::new(data)).ok()?;
    let container_xml =
        read_zip_text(&mut archive, "META-INF/container.xml", MAX_COVER_PAGE_BYTES)?;
    let rootfile_path = parse_rootfile_path(&container_xml)?;
    let opf_xml = read_zip_text(&mut archive, &rootfile_path, MAX_COVER_PAGE_BYTES)?;
    let package = parse_epub_package(&rootfile_path, &opf_xml);

    let mut candidates = cover_candidates(&package);
    candidates.sort_by(|a, b| b.score.cmp(&a.score));

    for candidate in candidates {
        let Some(bytes) = read_zip_bytes(&mut archive, &candidate.path, MAX_COVER_IMAGE_BYTES)
        else {
            continue;
        };
        if let Some(cover) = image_bytes_to_thumbnail_png(&bytes) {
            return Some(cover);
        }
    }
    None
}

#[derive(Debug, Clone)]
struct CoverCandidate {
    path: String,
    score: i32,
}

#[derive(Debug, Default)]
struct EpubPackage {
    opf_path: String,
    manifest: Vec<ManifestItem>,
    meta_cover_ids: Vec<String>,
    guide_cover_hrefs: Vec<String>,
}

#[derive(Debug)]
struct ManifestItem {
    id: String,
    href: String,
    media_type: String,
    properties: String,
    path: String,
}

fn cover_candidates(package: &EpubPackage) -> Vec<CoverCandidate> {
    let mut candidates = Vec::new();
    for item in &package.manifest {
        if !is_supported_raster_image(&item.media_type, &item.href) {
            continue;
        }
        let mut score = name_cover_score(&item.id, &item.href);
        if item
            .properties
            .split_whitespace()
            .any(|property| property.eq_ignore_ascii_case("cover-image"))
        {
            score += 1_000;
        }
        if package
            .meta_cover_ids
            .iter()
            .any(|id| item.id.eq_ignore_ascii_case(id))
        {
            score += 900;
        }
        candidates.push(CoverCandidate {
            path: item.path.clone(),
            score,
        });
    }
    for href in &package.guide_cover_hrefs {
        candidates.push(CoverCandidate {
            path: resolve_epub_path(&package.opf_path, href),
            score: 850,
        });
    }
    candidates
}

fn parse_rootfile_path(container_xml: &str) -> Option<String> {
    let mut reader = Reader::from_str(container_xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(element)) | Ok(Event::Start(element)) => {
                if local_name_is(element.name(), b"rootfile")
                    && let Some(path) = attr_value(&reader, &element, b"full-path")
                {
                    return Some(path);
                }
            }
            Ok(Event::Eof) => return None,
            Err(_) => return None,
            _ => {}
        }
        buf.clear();
    }
}

fn parse_epub_package(opf_path: &str, opf_xml: &str) -> EpubPackage {
    let mut package = EpubPackage {
        opf_path: opf_path.to_string(),
        ..Default::default()
    };
    let mut reader = Reader::from_str(opf_xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(element)) | Ok(Event::Start(element)) => {
                parse_opf_element(&reader, &element, &mut package);
            }
            Ok(Event::Eof) => break,
            Err(err) => {
                tracing::debug!(%err, "failed to parse EPUB package metadata for cover");
                break;
            }
            _ => {}
        }
        buf.clear();
    }
    package
}

fn parse_opf_element(reader: &Reader<&[u8]>, element: &BytesStart<'_>, package: &mut EpubPackage) {
    if local_name_is(element.name(), b"item") {
        let Some(id) = attr_value(reader, element, b"id") else {
            return;
        };
        let Some(href) = attr_value(reader, element, b"href") else {
            return;
        };
        let media_type = attr_value(reader, element, b"media-type").unwrap_or_default();
        let properties = attr_value(reader, element, b"properties").unwrap_or_default();
        let path = resolve_epub_path(&package.opf_path, &href);
        package.manifest.push(ManifestItem {
            id,
            href,
            media_type,
            properties,
            path,
        });
    } else if local_name_is(element.name(), b"meta") {
        if attr_value(reader, element, b"name")
            .is_some_and(|name| name.eq_ignore_ascii_case("cover"))
            && let Some(content) = attr_value(reader, element, b"content")
        {
            package.meta_cover_ids.push(content);
        }
    } else if local_name_is(element.name(), b"reference")
        && attr_value(reader, element, b"type").is_some_and(|value| {
            value
                .split_whitespace()
                .any(|part| part.eq_ignore_ascii_case("cover"))
        })
        && let Some(href) = attr_value(reader, element, b"href")
    {
        package.guide_cover_hrefs.push(href);
    }
}

fn read_zip_text(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    name: &str,
    max_bytes: u64,
) -> Option<String> {
    let bytes = read_zip_bytes(archive, name, max_bytes)?;
    String::from_utf8(bytes).ok()
}

fn read_zip_bytes(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    name: &str,
    max_bytes: u64,
) -> Option<Vec<u8>> {
    let mut file = archive.by_name(name).ok()?;
    if file.size() > max_bytes {
        return None;
    }
    let mut bytes = Vec::with_capacity(file.size().try_into().ok()?);
    file.read_to_end(&mut bytes).ok()?;
    Some(bytes)
}

fn attr_value(reader: &Reader<&[u8]>, element: &BytesStart<'_>, name: &[u8]) -> Option<String> {
    element
        .attributes()
        .with_checks(false)
        .flatten()
        .find(|attr| local_name_is(attr.key, name))
        .and_then(|attr| {
            attr.decode_and_unescape_value(reader.decoder())
                .ok()
                .map(|value| value.into_owned())
        })
}

fn local_name_is(name: QName<'_>, expected: &[u8]) -> bool {
    let bytes = name.as_ref();
    bytes.rsplit(|byte| *byte == b':').next().unwrap_or(bytes) == expected
}

fn is_supported_raster_image(media_type: &str, href: &str) -> bool {
    let media_type = media_type.to_ascii_lowercase();
    matches!(
        media_type.as_str(),
        "image/jpeg" | "image/jpg" | "image/png" | "image/gif" | "image/webp"
    ) || {
        let href = href.to_ascii_lowercase();
        href.ends_with(".jpg")
            || href.ends_with(".jpeg")
            || href.ends_with(".png")
            || href.ends_with(".gif")
            || href.ends_with(".webp")
    }
}

fn name_cover_score(id: &str, href: &str) -> i32 {
    let name = format!("{id} {href}").to_ascii_lowercase();
    let mut score = 100;
    if name.contains("cover") {
        score += 80;
    }
    if id.eq_ignore_ascii_case("cover") || id.eq_ignore_ascii_case("cover-image") {
        score += 30;
    }
    if name.contains("front") || name.contains("jacket") {
        score += 20;
    }
    if name.contains("ad") || name.contains("advert") || name.contains("banner") {
        score -= 120;
    }
    score
}

fn resolve_epub_path(base_file: &str, href: &str) -> String {
    let href = href.split('#').next().unwrap_or(href);
    let href = href.split('?').next().unwrap_or(href);
    let base = Path::new(base_file)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new(""));
    normalize_epub_path(base.join(href))
}

fn normalize_epub_path(path: PathBuf) -> String {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            std::path::Component::ParentDir => {
                parts.pop();
            }
            _ => {}
        }
    }
    parts.join("/")
}

fn image_bytes_to_thumbnail_png(data: &[u8]) -> Option<ExtractedCoverImage> {
    let image = image::load_from_memory(data).ok()?;
    let thumbnail = image.thumbnail(COVER_MAX_WIDTH, COVER_MAX_HEIGHT);
    let mut output = Cursor::new(Vec::new());
    thumbnail
        .write_to(&mut output, image::ImageFormat::Png)
        .ok()?;
    Some(ExtractedCoverImage {
        bytes: Bytes::from(output.into_inner()),
        content_type: THUMBNAIL_CONTENT_TYPE,
    })
}
