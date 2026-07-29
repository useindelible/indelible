use std::collections::HashSet;

use ind_application::AppError;
use ind_application::text::chunk_text;

use ind_domain::{
    ArchiveAssetKind, ArchiveAssetStatus, PreparedContentKind, PreparedContentLeaf,
    PreparedContentLocator, PreparedContentParent, PreparedItemContent, PreparedSectionKind,
};
use ind_html::html_to_text;
use tracing::warn;

use super::HandlerContext;
use super::assets::{find_asset, load_asset_bytes, load_text_asset};
use super::root::build_root_prepared;
use super::upsert_extracted_text_asset;

pub(super) async fn try_epub(
    ctx: &HandlerContext<'_>,
) -> Result<Option<PreparedItemContent>, AppError> {
    let Some(epub_asset) = find_asset(ctx.assets, ArchiveAssetKind::Epub) else {
        return Ok(None);
    };

    let toc_bytes = load_asset_bytes(ctx.object_storage, &epub_asset.s3_key).await?;
    let prefix = epub_asset
        .s3_key
        .strip_suffix("epub_toc.json")
        .unwrap_or_default()
        .to_string();

    let toc_result = serde_json::from_slice::<crate::epub_processing::EpubTocResponse>(&toc_bytes);

    match toc_result {
        Ok(toc) if !toc.toc.is_empty() => extract_epub_from_toc(ctx, &toc, &prefix).await,
        _ => {
            if let Some(text) = load_reusable_extracted_text(ctx).await? {
                return Ok(Some(build_root_prepared(
                    ctx.document,
                    PreparedContentKind::Epub,
                    &text,
                    ctx.chunking,
                )));
            }
            warn!(
                document_id = %ctx.document.id,
                "EPUB TOC is malformed or empty, probing for chapter HTML files"
            );
            extract_epub_fallback(ctx, &prefix).await
        }
    }
}

async fn load_reusable_extracted_text(
    ctx: &HandlerContext<'_>,
) -> Result<Option<String>, AppError> {
    let Some(extracted) = ctx.assets.iter().find(|asset| {
        asset.asset_kind == ArchiveAssetKind::ExtractedText
            && asset.status == ArchiveAssetStatus::Completed
            && asset.content_type == "text/plain"
            && !asset.s3_key.is_empty()
    }) else {
        return Ok(None);
    };

    let text = load_text_asset(ctx.object_storage, &extracted.s3_key).await?;
    Ok((!text.trim().is_empty()).then_some(text))
}

async fn extract_epub_from_toc(
    ctx: &HandlerContext<'_>,
    toc: &crate::epub_processing::EpubTocResponse,
    prefix: &str,
) -> Result<Option<PreparedItemContent>, AppError> {
    let mut parents = Vec::new();
    let mut leaves = Vec::new();
    let mut root_parts = Vec::new();

    // Deduplicate by spine_index: multiple TOC entries (subsections) can share
    // the same chapter file. Only load and chunk each chapter once.
    let mut seen_spine_indices = HashSet::new();
    let mut ordinal: i32 = 0;
    let mut failed_chapters = 0usize;

    for entry in toc.toc.iter() {
        if !seen_spine_indices.insert(entry.spine_index) {
            continue;
        }

        let chapter_key = format!("{prefix}epub_ch_{}.html", entry.spine_index);
        let chapter_html = match load_text_asset(ctx.object_storage, &chapter_key).await {
            Ok(html) => html,
            Err(_) => {
                failed_chapters += 1;
                continue;
            }
        };
        let chapter_text = html_to_text(&chapter_html);
        if chapter_text.trim().is_empty() {
            failed_chapters += 1;
            continue;
        }

        root_parts.push(chapter_text.clone());

        let chapter_id = if entry.chapter_id.is_empty() {
            entry.id.clone()
        } else {
            entry.chapter_id.clone()
        };

        let locator = PreparedContentLocator {
            chapter_index: Some(ordinal),
            page_number: None,
            spine_index: Some(entry.spine_index),
        };

        parents.push(PreparedContentParent {
            kind: PreparedSectionKind::Chapter,
            key: chapter_id.clone(),
            title: Some(entry.title.clone()),
            ordinal,
            text: chapter_text.clone(),
            locator: Some(locator.clone()),
        });

        let chunks = chunk_text(&chapter_text, ctx.chunking);
        for chunk in chunks {
            leaves.push(PreparedContentLeaf {
                parent_key: chapter_id.clone(),
                kind: PreparedSectionKind::Chapter,
                key: format!("{}:chunk_{}", chapter_id, chunk.index),
                ordinal: chunk.index,
                text: chunk.content,
                locator: Some(locator.clone()),
            });
        }

        ordinal += 1;
    }

    let expected_chapters = seen_spine_indices.len();
    if parents.is_empty() {
        upsert_extracted_text_asset(
            ctx,
            None,
            ArchiveAssetStatus::Failed,
            Some(format!(
                "EPUB text extraction failed: 0 of {expected_chapters} chapters produced text"
            )),
        )
        .await?;
        return Ok(None);
    }

    let root_text = root_parts.join("\n\n");
    let loaded_chapters = parents.len();
    if failed_chapters > 0 {
        upsert_extracted_text_asset(
            ctx,
            Some(&root_text),
            ArchiveAssetStatus::Degraded,
            Some(format!(
                "EPUB text extraction degraded: {loaded_chapters} of {expected_chapters} chapters loaded"
            )),
        )
        .await?;
    } else {
        upsert_extracted_text_asset(ctx, Some(&root_text), ArchiveAssetStatus::Completed, None)
            .await?;
    }

    Ok(Some(PreparedItemContent {
        document_id: ctx.document.id,
        user_id: ctx.document.user_id,
        source_kind: PreparedContentKind::Epub,
        title: ctx.document.title.clone(),
        root_text,
        parents,
        leaves,
    }))
}

async fn extract_epub_fallback(
    ctx: &HandlerContext<'_>,
    prefix: &str,
) -> Result<Option<PreparedItemContent>, AppError> {
    let mut recovered_parts = Vec::new();

    for idx in 0..500 {
        let chapter_key = format!("{prefix}epub_ch_{idx}.html");
        match load_text_asset(ctx.object_storage, &chapter_key).await {
            Ok(html) => {
                let text = html_to_text(&html);
                if !text.trim().is_empty() {
                    recovered_parts.push(text);
                }
            }
            Err(_) => break,
        }
    }

    if recovered_parts.is_empty() {
        upsert_extracted_text_asset(
            ctx,
            None,
            ArchiveAssetStatus::Failed,
            Some("EPUB text extraction failed: no recoverable chapters".into()),
        )
        .await?;
        return Ok(None);
    }

    let combined = recovered_parts.join("\n\n");
    upsert_extracted_text_asset(
        ctx,
        Some(&combined),
        ArchiveAssetStatus::Degraded,
        Some("EPUB text extraction degraded: recovered chapters without a valid TOC".into()),
    )
    .await?;
    Ok(Some(build_root_prepared(
        ctx.document,
        PreparedContentKind::Epub,
        &combined,
        ctx.chunking,
    )))
}
