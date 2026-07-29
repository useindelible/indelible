//! Client-facing asset URL projection.
//!
//! Every asset URL placed in a response body points at the API origin,
//! regardless of `asset_serving_mode`. The mode only changes how the asset
//! proxy answers a GET (stream vs redirect to a presigned URL), so clients
//! never need to know which mode the server runs and response bodies can
//! never leak an internal object-store endpoint. Keeping projection here,
//! out of `ind-application`, is what makes that enforceable.

use ind_domain::{ArchiveAssetKind, DocumentId};

pub fn document_asset_url(
    base_url: &str,
    document_id: DocumentId,
    kind: ArchiveAssetKind,
) -> String {
    format!(
        "{}/api/v1/assets/documents/{document_id}/{kind}",
        base_url.trim_end_matches('/')
    )
}

/// URL for an internal avatar key of the form `usr_<id>/avatars/<file>`.
pub fn avatar_url(base_url: &str, avatar_key: &str) -> String {
    format!(
        "{}/api/v1/assets/{avatar_key}",
        base_url.trim_end_matches('/')
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_asset_url_targets_the_asset_proxy() {
        let id: DocumentId = "doc_019624b8-7a2e-7f52-a5b7-111111111111"
            .parse()
            .expect("valid document id");
        assert_eq!(
            document_asset_url("http://api.example/", id, ArchiveAssetKind::ReadableHtml),
            format!("http://api.example/api/v1/assets/documents/{id}/readable_html")
        );
    }

    #[test]
    fn avatar_url_embeds_the_owner_scoped_key() {
        assert_eq!(
            avatar_url("http://api.example", "usr_abc/avatars/a.png"),
            "http://api.example/api/v1/assets/usr_abc/avatars/a.png"
        );
    }
}
