use secrecy::SecretString;

#[derive(Debug, Clone)]
pub struct S3Config {
    pub endpoint: String,
    pub region: String,
    pub access_key: SecretString,
    pub secret_key: SecretString,
    pub bucket: String,
    pub force_path_style: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum S3ConfigError {
    #[error("{0} is required when S3_ENABLED=true")]
    Missing(&'static str),
}

impl S3Config {
    pub fn from_required_parts(
        endpoint: Option<String>,
        region: String,
        access_key: Option<SecretString>,
        secret_key: Option<SecretString>,
        bucket: Option<String>,
        force_path_style: bool,
    ) -> Result<Self, S3ConfigError> {
        Ok(Self {
            endpoint: endpoint.ok_or(S3ConfigError::Missing("S3_ENDPOINT"))?,
            region,
            access_key: access_key.ok_or(S3ConfigError::Missing("S3_ACCESS_KEY"))?,
            secret_key: secret_key.ok_or(S3ConfigError::Missing("S3_SECRET_KEY"))?,
            bucket: bucket.ok_or(S3ConfigError::Missing("S3_BUCKET"))?,
            force_path_style,
        })
    }
}
