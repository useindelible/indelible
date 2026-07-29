use ind_application::AppError;
use ind_application::text::{ChunkingConfig, chunk_text};

use ind_domain::{
    ArchiveAssetKind, Document, PreparedContentKind, PreparedContentLeaf, PreparedContentParent,
    PreparedItemContent, PreparedSectionKind,
};
use ind_html::html_to_text;

use super::HandlerContext;
use super::assets::{find_asset, load_text_asset};

pub(super) fn build_root_prepared(
    document: &Document,
    source_kind: PreparedContentKind,
    plain_text: &str,
    chunking: ChunkingConfig,
) -> PreparedItemContent {
    let parent = PreparedContentParent {
        kind: PreparedSectionKind::Item,
        key: String::new(),
        title: None,
        ordinal: 0,
        text: plain_text.to_string(),
        locator: None,
    };

    let chunks = chunk_text(plain_text, chunking);
    let leaves: Vec<PreparedContentLeaf> = chunks
        .into_iter()
        .map(|chunk| PreparedContentLeaf {
            parent_key: String::new(),
            kind: PreparedSectionKind::Item,
            key: format!(":chunk_{}", chunk.index),
            ordinal: chunk.index,
            text: chunk.content,
            locator: None,
        })
        .collect();

    PreparedItemContent {
        document_id: document.id,
        user_id: document.user_id,
        source_kind,
        title: document.title.clone(),
        root_text: plain_text.to_string(),
        parents: vec![parent],
        leaves,
    }
}

pub(super) async fn try_readable_html(
    ctx: &HandlerContext<'_>,
) -> Result<Option<PreparedItemContent>, AppError> {
    let Some(asset) = find_asset(ctx.assets, ArchiveAssetKind::ReadableHtml) else {
        return Ok(None);
    };
    if asset.content_type != "text/html" {
        return Ok(None);
    }

    let html = load_text_asset(ctx.object_storage, &asset.s3_key).await?;
    let plain_text = html_to_text(&html);
    if plain_text.trim().is_empty() {
        return Ok(None);
    }

    Ok(Some(build_root_prepared(
        ctx.document,
        PreparedContentKind::ReadableHtml,
        &plain_text,
        ctx.chunking,
    )))
}
