use anyhow::Result;
use aws_sdk_s3::Client;
use bytes::Bytes;
use secrecy::ExposeSecret;

use crate::config::RendererConfig;

pub struct S3Storage {
    client: Client,
    bucket: String,
}

impl S3Storage {
    pub async fn from_config(config: &RendererConfig) -> Result<Self> {
        let mut s3_config = aws_config::defaults(aws_config::BehaviorVersion::latest());

        if let Some(ref endpoint) = config.s3.endpoint {
            s3_config = s3_config.endpoint_url(endpoint);
        }

        if let (Some(key), Some(secret)) = (&config.s3.access_key, &config.s3.secret_key) {
            s3_config = s3_config.credentials_provider(aws_sdk_s3::config::Credentials::new(
                key.expose_secret(),
                secret.expose_secret(),
                None,
                None,
                "env",
            ));
        }

        s3_config = s3_config.region(aws_sdk_s3::config::Region::new(config.s3.region.clone()));

        let aws_config = s3_config.load().await;
        let mut s3_builder = aws_sdk_s3::config::Builder::from(&aws_config);
        if config.s3.force_path_style {
            s3_builder = s3_builder.force_path_style(true);
        }

        let client = Client::from_conf(s3_builder.build());

        Ok(Self {
            client,
            bucket: config.s3.bucket.clone(),
        })
    }

    pub async fn upload(&self, key: &str, content_type: &str, body: Vec<u8>) -> Result<i64> {
        let size = body.len() as i64;
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(Bytes::from(body).into())
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("s3 upload failed: {e}"))?;

        Ok(size)
    }

    pub async fn download(&self, key: &str) -> Result<Vec<u8>> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("s3 download failed: {e}"))?;

        let body = resp
            .body
            .collect()
            .await
            .map_err(|e| anyhow::anyhow!("s3 body read failed: {e}"))?;

        Ok(body.into_bytes().to_vec())
    }
}
