/// How the asset proxy endpoints under `/api/v1/assets/...` serve bytes.
/// Response bodies carry API-origin asset URLs in either mode; the mode never
/// changes what clients see, only how the proxy answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetServingMode {
    /// Redirect (302) asset GETs to short-lived presigned S3 URLs, offloading
    /// download bandwidth. Only useful when the object store endpoint is
    /// reachable by browsers (e.g. real AWS S3), which the shipped compose
    /// (internal MinIO) is not.
    Presigned,
    /// Default: stream bytes through the API. The object store can stay
    /// fully private. Requires `ind_asset` HMAC cookie or Bearer auth.
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
