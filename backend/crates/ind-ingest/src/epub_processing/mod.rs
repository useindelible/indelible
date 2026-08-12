use std::collections::HashMap;
use std::io::Cursor;

use zip::ZipArchive;

mod io;
mod opf;
mod paths;
mod sanitize;
mod text;
mod toc;
mod types;
mod xml;

#[cfg(test)]
mod tests;

pub use types::{
    EpubChapter, EpubError, EpubMetadata, EpubTocEntry, EpubTocResponse, ProcessedEpub,
};

use io::read_zip_text;
use opf::{parse_opf, parse_rootfile_path};
use paths::resolve_path;
use sanitize::sanitize_chapter_html;
use text::count_words;
use toc::extract_toc;
use types::ManifestItem;

use crate::archive_limits::{ArchiveLimits, ArchiveReadBudget};

pub(super) const WORDS_PER_PAGE: u32 = 250;

pub fn process_epub(data: &[u8]) -> Result<ProcessedEpub, EpubError> {
    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)?;

    if archive.len() > ArchiveLimits::EPUB.max_entries {
        return Err(EpubError::Invalid("archive has too many entries".into()));
    }
    let mut budget = ArchiveReadBudget::new(ArchiveLimits::EPUB);

    let container_xml = read_zip_text(&mut archive, "META-INF/container.xml", &mut budget)
        .ok_or_else(|| EpubError::Invalid("missing META-INF/container.xml".into()))?;

    let rootfile_path = parse_rootfile_path(&container_xml)
        .ok_or_else(|| EpubError::Invalid("missing rootfile in container.xml".into()))?;

    let opf_dir = rootfile_path
        .rfind('/')
        .map(|i| &rootfile_path[..i + 1])
        .unwrap_or("");

    let opf_content = read_zip_text(&mut archive, &rootfile_path, &mut budget)
        .ok_or_else(|| EpubError::Invalid("missing OPF file".into()))?;

    let opf = parse_opf(&opf_content);

    let manifest_by_id: HashMap<&str, &ManifestItem> =
        opf.manifest.iter().map(|m| (m.id.as_str(), m)).collect();

    let toc_points = extract_toc(&mut archive, &opf, opf_dir, &manifest_by_id, &mut budget);

    let mut chapters = Vec::new();
    let spine_items: Vec<_> = opf
        .spine
        .iter()
        .filter_map(|s| manifest_by_id.get(s.idref.as_str()))
        .collect();

    for (spine_index, item) in spine_items.iter().enumerate() {
        if !item.media_type.contains("html") && !item.media_type.contains("xml") {
            continue;
        }

        let item_path = resolve_path(opf_dir, &item.href);
        let raw_html = match read_zip_text(&mut archive, &item_path, &mut budget) {
            Some(h) => h,
            None => continue,
        };

        let sanitized =
            sanitize_chapter_html(&raw_html, &mut archive, opf_dir, &item.href, &mut budget);

        let word_count = count_words(&sanitized);

        let title = toc_points
            .iter()
            .find(|p| {
                let src_base = p.content_src.split('#').next().unwrap_or("");
                resolve_path(opf_dir, src_base) == item_path
            })
            .map(|p| p.label.clone())
            .unwrap_or_default();

        chapters.push(EpubChapter {
            id: item.id.clone(),
            title,
            html: sanitized,
            word_count,
            spine_index,
        });
    }

    let total_words: u32 = chapters.iter().map(|c| c.word_count).sum();
    let estimated_pages = total_words.div_ceil(WORDS_PER_PAGE);

    // Map resolved file path → (spine_index, chapter_id, word_count, href_basename) for navPoint matching
    let path_to_spine: HashMap<String, (usize, String, u32, String)> = chapters
        .iter()
        .map(|ch| {
            let href = manifest_by_id
                .values()
                .find(|m| m.id == ch.id)
                .map(|m| m.href.as_str())
                .unwrap_or(&ch.id);
            let path = resolve_path(opf_dir, href);
            let basename = href.split('/').next_back().unwrap_or(href).to_string();
            (
                path,
                (ch.spine_index, ch.id.clone(), ch.word_count, basename),
            )
        })
        .collect();

    // Map spine_index → href_basename for the no-TOC fallback path
    let spine_index_to_href: HashMap<usize, String> = path_to_spine
        .values()
        .map(|(idx, _, _, basename)| (*idx, basename.clone()))
        .collect();

    // Cumulative word counts per spine_index for page estimation
    let mut spine_cumulative: HashMap<usize, u32> = HashMap::new();
    let mut running: u32 = 0;
    for ch in &chapters {
        spine_cumulative.insert(ch.spine_index, running);
        running += ch.word_count;
    }

    let toc: Vec<EpubTocEntry> = if toc_points.is_empty() {
        // No nav document — fall back to one entry per spine item
        let mut cumulative_words: u32 = 0;
        chapters
            .iter()
            .map(|ch| {
                let start_page = cumulative_words / WORDS_PER_PAGE + 1;
                cumulative_words += ch.word_count;
                let spine_href = spine_index_to_href
                    .get(&ch.spine_index)
                    .cloned()
                    .unwrap_or_default();
                EpubTocEntry {
                    id: ch.id.clone(),
                    title: if ch.title.is_empty() {
                        format!("Chapter {}", ch.spine_index + 1)
                    } else {
                        ch.title.clone()
                    },
                    depth: 1,
                    spine_index: ch.spine_index,
                    chapter_id: ch.id.clone(),
                    fragment: None,
                    word_count: ch.word_count,
                    start_page,
                    spine_href,
                }
            })
            .collect()
    } else {
        // Build TOC from navPoints — preserves hierarchy and subsections
        let mut seen_ids: HashMap<String, u32> = HashMap::new();
        let mut entries: Vec<EpubTocEntry> = Vec::with_capacity(toc_points.len());
        for p in &toc_points {
            let src_base = p.content_src.split('#').next().unwrap_or("");
            let resolved = resolve_path(opf_dir, src_base);
            let Some((spine_index, chapter_id, word_count, spine_href)) =
                path_to_spine.get(&resolved)
            else {
                continue;
            };
            let cumulative = spine_cumulative.get(spine_index).copied().unwrap_or(0);
            let start_page = cumulative / WORDS_PER_PAGE + 1;
            let fragment = p.content_src.split('#').nth(1).map(|f| f.to_string());
            let base_id = fragment.clone().unwrap_or_else(|| chapter_id.clone());
            // Deduplicate: if the same id appears more than once (e.g. duplicate NCX entries),
            // append _2, _3, ... to keep each TOC entry's id unique.
            let id = {
                let count = seen_ids.entry(base_id.clone()).or_insert(0);
                *count += 1;
                if *count == 1 {
                    base_id
                } else {
                    format!("{}_{}", base_id, count)
                }
            };
            entries.push(EpubTocEntry {
                id,
                title: p.label.clone(),
                depth: p.depth,
                spine_index: *spine_index,
                chapter_id: chapter_id.clone(),
                fragment,
                word_count: *word_count,
                start_page,
                spine_href: spine_href.clone(),
            });
        }
        entries
    };

    let metadata = EpubMetadata {
        title: opf.title,
        author: opf.author,
        publisher: opf.publisher,
        language: opf.language,
        isbn: opf.isbn,
        total_chapters: chapters.len(),
        total_words,
        estimated_pages,
    };

    Ok(ProcessedEpub {
        toc,
        chapters,
        metadata,
    })
}
