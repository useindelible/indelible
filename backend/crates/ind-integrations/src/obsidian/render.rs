use chrono::{DateTime, Utc};
use ind_domain::ObsidianExportSettings;
use minijinja::context;

use super::hash::{obsidian_content_hash, sha256_hex};
use super::paths::{full_document_path_for_note_path, obsidian_link_for_path, render_file_path};
use super::template::{format_date, obsidian_template_environment, render_template};
use super::types::{
    ObsidianArtifactEntry, ObsidianRenderCursor, ObsidianRenderDocument, ObsidianRenderError,
    ObsidianRenderHighlight, RenderedObsidianDocument,
};

pub fn render_document(
    settings: &ObsidianExportSettings,
    doc: &ObsidianRenderDocument,
    cursor: &ObsidianRenderCursor,
    now: DateTime<Utc>,
) -> Result<Option<RenderedObsidianDocument>, ObsidianRenderError> {
    let is_new_page = cursor.force_full || !cursor.has_delivered;
    let mut ordered_highlights = doc.highlights.clone();
    ordered_highlights.sort_by(|a, b| {
        a.created_at
            .cmp(&b.created_at)
            .then_with(|| a.id.cmp(&b.id))
    });
    let new_highlights = if is_new_page {
        ordered_highlights
    } else {
        ordered_highlights
            .into_iter()
            .filter(|h| {
                let Some(ts) = cursor.last_highlight_created_at else {
                    return true;
                };
                h.created_at > ts
                    || (h.created_at == ts
                        && cursor
                            .last_highlight_id
                            .as_deref()
                            .is_none_or(|last_id| h.id.as_str() > last_id))
            })
            .collect()
    };

    let should_emit_note = is_new_page || !new_highlights.is_empty();
    let full_document_hash = doc
        .full_document_text
        .as_ref()
        .filter(|text| !text.trim().is_empty())
        .map(|text| sha256_hex(text.as_bytes()));
    let has_full_document_text =
        settings.export_all_reader_documents && full_document_hash.is_some();
    let should_emit_full_document = full_document_hash.as_deref().is_some_and(|hash| {
        settings.export_all_reader_documents
            && (cursor.force_full || cursor.last_full_document_hash.as_deref() != Some(hash))
    });

    if !should_emit_note && !should_emit_full_document {
        return Ok(None);
    }

    let rendered_file_path = render_file_path(settings, doc, now)?;
    let file_path = cursor.generated_path.clone().unwrap_or(rendered_file_path);
    let full_document_text_path = has_full_document_text.then(|| {
        cursor
            .generated_full_document_path
            .clone()
            .unwrap_or_else(|| full_document_path_for_note_path(&file_path))
    });

    let full_document_link = full_document_text_path
        .as_ref()
        .map(|path| format!("\n\n[Full document text]({})", obsidian_link_for_path(path)));
    let should_append_full_document_link =
        should_emit_full_document && !is_new_page && cursor.last_full_document_hash.is_none();

    let body = if should_emit_note || should_append_full_document_link {
        if is_new_page {
            let rendered_highlights =
                render_highlights(settings, doc, &new_highlights, is_new_page, now)?;
            let mut parts = Vec::new();
            if let Some(properties) = settings.properties_template.as_deref() {
                let rendered =
                    render_template("properties", properties, doc, &[], is_new_page, now, None)?;
                if !rendered.trim().is_empty() {
                    parts.push(format!("---\n{}\n---", rendered.trim()));
                }
            }
            parts.push(render_template(
                "page_title",
                &settings.page_title_template,
                doc,
                &[],
                is_new_page,
                now,
                None,
            )?);
            parts.push(render_template(
                "metadata",
                &settings.metadata_template,
                doc,
                &[],
                is_new_page,
                now,
                None,
            )?);
            if let Some(link) = full_document_link.as_ref() {
                parts.push(link.trim().to_string());
            }
            parts.push(rendered_highlights);
            Some(join_markdown(parts))
        } else {
            let mut parts = Vec::new();
            if should_append_full_document_link && let Some(link) = full_document_link.as_ref() {
                parts.push(link.trim().to_string());
            }
            if should_emit_note {
                parts.push(render_highlights(
                    settings,
                    doc,
                    &new_highlights,
                    is_new_page,
                    now,
                )?);
            }
            Some(join_markdown(parts))
        }
    } else {
        None
    };

    let (full_content, append_only_content) = if is_new_page {
        (body, None)
    } else {
        (None, body)
    };

    let last_content_hash = full_content
        .as_ref()
        .or(append_only_content.as_ref())
        .map(|content| obsidian_content_hash(content));
    let last_full_document_hash = should_emit_full_document
        .then(|| full_document_hash.clone())
        .flatten();
    let last_highlight = new_highlights.last();

    Ok(Some(RenderedObsidianDocument {
        entry: ObsidianArtifactEntry {
            subject_id: doc.subject_id.clone(),
            subject_kind: doc.subject_kind.clone(),
            book_id: doc.subject_id.clone(),
            file_path,
            full_content,
            append_only_content,
            last_content_hash,
            last_highlight_created_at: last_highlight.map(|h| h.created_at),
            last_highlight_id: last_highlight.map(|h| h.id.clone()),
            full_document_text_path,
            full_document_text: doc
                .full_document_text
                .clone()
                .filter(|_| should_emit_full_document),
            last_full_document_hash,
        },
    }))
}

pub fn render_sync_notification(
    settings: &ObsidianExportSettings,
    document_count: usize,
    now: DateTime<Utc>,
) -> Result<String, ObsidianRenderError> {
    if !settings.sync_notifications {
        return Ok(String::new());
    }
    let env = obsidian_template_environment();
    env.template_from_str(&settings.sync_notification_template)
        .map_err(|source| ObsidianRenderError::Template {
            name: "sync_notification",
            source,
        })?
        .render(context! {
            date => format_date(now, "F j, Y"),
            time => now.format("%H:%M").to_string(),
            document_count => document_count,
        })
        .map_err(|source| ObsidianRenderError::Template {
            name: "sync_notification",
            source,
        })
}

fn render_highlights(
    settings: &ObsidianExportSettings,
    doc: &ObsidianRenderDocument,
    highlights: &[ObsidianRenderHighlight],
    is_new_page: bool,
    now: DateTime<Utc>,
) -> Result<String, ObsidianRenderError> {
    if highlights.is_empty() {
        return Ok(String::new());
    }
    let header = render_template(
        "highlight_header",
        &settings.highlight_header_template,
        doc,
        highlights,
        is_new_page,
        now,
        None,
    )?;
    let rendered = highlights
        .iter()
        .map(|h| {
            render_template(
                "highlight",
                &settings.highlight_template,
                doc,
                highlights,
                is_new_page,
                now,
                Some(h),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(join_markdown(
        std::iter::once(header)
            .chain(rendered)
            .filter(|part| !part.trim().is_empty())
            .collect(),
    ))
}

fn join_markdown(parts: Vec<String>) -> String {
    parts
        .into_iter()
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}
