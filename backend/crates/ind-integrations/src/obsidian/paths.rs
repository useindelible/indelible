use chrono::{DateTime, Utc};
use ind_domain::ObsidianExportSettings;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use url::Url;

use super::category::category_for_item_type;
use super::hash::sha256_hex;
use super::template::render_template;
use super::types::{ObsidianRenderDocument, ObsidianRenderError};

pub const SERVER_BASE_FOLDER: &str = "Indelible";
const DEFAULT_FILE_NAME_TEMPLATE: &str = "{{title}}";
const OBSIDIAN_MARKDOWN_LINK_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'[')
    .add(b']')
    .add(b'(')
    .add(b')')
    .add(b'`')
    .add(b'?')
    .add(b'&')
    .add(b'\'')
    .add(b'{')
    .add(b'}')
    .add(b'|')
    .add(b'\\')
    .add(b'^');

pub(super) fn render_file_path(
    settings: &ObsidianExportSettings,
    doc: &ObsidianRenderDocument,
    now: DateTime<Utc>,
) -> Result<String, ObsidianRenderError> {
    let configured_file_name_template = settings
        .file_name_template
        .as_deref()
        .filter(|s| !s.trim().is_empty());
    let file_name_template = configured_file_name_template.unwrap_or(DEFAULT_FILE_NAME_TEMPLATE);
    let raw_file_name =
        render_template("file_name", file_name_template, doc, &[], true, now, None)?;
    let file_name =
        default_file_name_for_document(configured_file_name_template, doc).unwrap_or(raw_file_name);
    let file_name = sanitize_path_segment(&file_name);
    let category = category_for_item_type(doc.item_type);
    let path = if settings.group_files_in_category_folders {
        let folder_template = settings
            .category_folder_templates
            .get(category)
            .map(String::as_str)
            .unwrap_or(category);
        let folder = render_template("folder_name", folder_template, doc, &[], true, now, None)?;
        format!(
            "{}/{}/{}.md",
            SERVER_BASE_FOLDER,
            sanitize_path_segment(&folder),
            file_name
        )
    } else {
        format!("{}/{}.md", SERVER_BASE_FOLDER, file_name)
    };
    Ok(path)
}

fn default_file_name_for_document(
    configured_file_name_template: Option<&str>,
    doc: &ObsidianRenderDocument,
) -> Option<String> {
    if !uses_default_file_name_template(configured_file_name_template) {
        return None;
    }
    if !is_generic_export_title(&doc.title) {
        return None;
    }

    let suffix = stable_subject_path_suffix(&doc.subject_id);
    let title = doc.title.trim();
    Some(match doc.url.as_deref().and_then(url_host_label) {
        Some(host) if !host.eq_ignore_ascii_case(title) => {
            format!("{host} - {title} - {suffix}")
        }
        _ => format!("{title} - {suffix}"),
    })
}

fn uses_default_file_name_template(configured_file_name_template: Option<&str>) -> bool {
    configured_file_name_template
        .map(|template| {
            template
                .chars()
                .filter(|ch| !ch.is_whitespace())
                .collect::<String>()
        })
        .is_none_or(|template| template == DEFAULT_FILE_NAME_TEMPLATE)
}

fn is_generic_export_title(title: &str) -> bool {
    matches!(
        normalize_generic_title(title).as_str(),
        "just a moment"
            | "untitled"
            | "x"
            | "404"
            | "404 page not found"
            | "403"
            | "403 forbidden"
            | "page not found"
            | "reddit the heart of the internet"
            | "attention required cloudflare"
    )
}

fn normalize_generic_title(title: &str) -> String {
    let mut normalized = String::new();
    for ch in title.chars() {
        if ch.is_alphanumeric() {
            normalized.extend(ch.to_lowercase());
        } else {
            normalized.push(' ');
        }
    }
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn url_host_label(raw_url: &str) -> Option<String> {
    let url = Url::parse(raw_url).ok()?;
    let host = url.host_str()?.trim().trim_start_matches("www.").trim();
    (!host.is_empty()).then(|| host.to_string())
}

pub fn stable_subject_path_suffix(subject_id: &str) -> String {
    let compact = subject_id
        .split_once('_')
        .map(|(_, rest)| rest)
        .unwrap_or(subject_id)
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    let start = compact.len().saturating_sub(12);
    format!("subject-{}", &compact[start..])
}

pub fn full_document_path_for_note_path(note_path: &str) -> String {
    let without_ext = note_path.strip_suffix(".md").unwrap_or(note_path);
    format!("{without_ext} Full Text.md")
}

pub const MAX_PATH_SEGMENT_BYTES: usize = 180;

fn sanitize_path_segment(input: &str) -> String {
    let sanitized = input
        .trim()
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => '-',
            c => c,
        })
        .collect::<String>();
    let collapsed = sanitized
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches('.')
        .to_string();
    if collapsed.is_empty() {
        "Untitled".to_string()
    } else if collapsed.len() <= MAX_PATH_SEGMENT_BYTES {
        collapsed
    } else {
        truncate_path_segment(&collapsed)
    }
}

pub fn obsidian_link_for_path(path: &str) -> String {
    utf8_percent_encode(path, OBSIDIAN_MARKDOWN_LINK_ENCODE_SET).to_string()
}

fn truncate_path_segment(segment: &str) -> String {
    let hash = &sha256_hex(segment.as_bytes())[..12];
    let suffix = format!("-{hash}");
    let limit = MAX_PATH_SEGMENT_BYTES.saturating_sub(suffix.len());
    let mut end = 0;
    for (idx, ch) in segment.char_indices() {
        let next = idx + ch.len_utf8();
        if next > limit {
            break;
        }
        end = next;
    }
    let prefix = segment[..end].trim_end_matches([' ', '.', '-']);
    if prefix.is_empty() {
        format!("Untitled{suffix}")
    } else {
        format!("{prefix}{suffix}")
    }
}
