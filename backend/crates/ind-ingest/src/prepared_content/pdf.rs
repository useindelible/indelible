use ind_application::AppError;
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, PreparedContentKind, PreparedItemContent};

use super::HandlerContext;
use super::assets::{find_asset, load_asset_bytes, load_text_asset};
use super::root::build_root_prepared;
use super::upsert_extracted_text_asset;

pub(super) async fn try_pdf(
    ctx: &HandlerContext<'_>,
) -> Result<Option<PreparedItemContent>, AppError> {
    let Some(upload_asset) = find_asset(ctx.assets, ArchiveAssetKind::OriginalUpload) else {
        return Ok(None);
    };
    if upload_asset.content_type != "application/pdf" {
        return Ok(None);
    }

    if let Some(extracted) = ctx.assets.iter().find(|asset| {
        asset.asset_kind == ArchiveAssetKind::ExtractedText
            && asset.status == ArchiveAssetStatus::Completed
            && asset.content_type == "text/plain"
            && !asset.s3_key.is_empty()
    }) {
        let text = load_text_asset(ctx.object_storage, &extracted.s3_key).await?;
        if !text.trim().is_empty() {
            return Ok(Some(build_root_prepared(
                ctx.document,
                PreparedContentKind::Pdf,
                &text,
                ctx.chunking,
            )));
        }
    }

    let bytes = load_asset_bytes(ctx.object_storage, &upload_asset.s3_key).await?;
    let text = match crate::extract_pdf_text(&bytes) {
        Ok(t) if !t.trim().is_empty() => t,
        Ok(_) => {
            upsert_extracted_text_asset(
                ctx,
                None,
                ArchiveAssetStatus::Failed,
                Some("PDF text extraction produced no text".into()),
            )
            .await?;
            return Ok(None);
        }
        Err(err) => {
            upsert_extracted_text_asset(
                ctx,
                None,
                ArchiveAssetStatus::Failed,
                Some(format!("PDF text extraction failed: {err}")),
            )
            .await?;
            return Ok(None);
        }
    };

    upsert_extracted_text_asset(ctx, Some(&text), ArchiveAssetStatus::Completed, None).await?;

    Ok(Some(build_root_prepared(
        ctx.document,
        PreparedContentKind::Pdf,
        &text,
        ctx.chunking,
    )))
}
