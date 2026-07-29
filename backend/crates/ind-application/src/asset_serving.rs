/// Serving mode for asset URLs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetServingMode {
    /// Self-hosted default: each URL is a presigned S3 URL with HMAC in query params.
    Presigned,
    /// API passthrough: URLs point to the API server, which streams from S3.
    /// Requires `ind_asset` HMAC cookie or Bearer auth.
    Passthrough,
}

impl AssetServingMode {
    pub fn from_str_with_validation(s: &str) -> Result<Self, String> {
        match s {
            "presigned" => Ok(Self::Presigned),
            "passthrough" => Ok(Self::Passthrough),
            "cdn" => Err("ASSET_SERVING_MODE=cdn is not yet supported. \
                 The CDN mode requires a Cloudflare Worker that has not been deployed. \
                 Use 'presigned' or 'passthrough'."
                .to_string()),
            other => Err(format!(
                "invalid ASSET_SERVING_MODE '{other}': expected 'presigned' or 'passthrough'"
            )),
        }
    }
}
