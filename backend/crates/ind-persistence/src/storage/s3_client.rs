use std::time::Duration;

use aws_sdk_s3::Client;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use ind_application::AppError;
use ind_application::storage::{
    ByteRange, ObjectData, ObjectListEntry, ObjectListPage, ObjectStorage, RangedObjectData,
    UploadResult,
};
use secrecy::ExposeSecret;

use super::config::S3Config;

pub struct S3Client {
    client: Client,
    bucket: String,
}

impl S3Client {
    pub fn from_config(config: S3Config) -> Self {
        let credentials = Credentials::new(
            config.access_key.expose_secret().to_string(),
            config.secret_key.expose_secret().to_string(),
            None,
            None,
            "ind-config",
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .endpoint_url(config.endpoint)
            .region(Region::new(config.region))
            .credentials_provider(credentials)
            .force_path_style(config.force_path_style)
            .build();

        let client = Client::from_conf(s3_config);

        Self {
            client,
            bucket: config.bucket,
        }
    }

    pub fn from_client(client: Client, bucket: String) -> Self {
        Self { client, bucket }
    }
}

fn map_sdk_err(err: impl std::fmt::Display) -> AppError {
    AppError::ExternalService {
        service: "s3".into(),
        message: err.to_string(),
    }
}

#[async_trait::async_trait]
impl ObjectStorage for S3Client {
    async fn upload(
        &self,
        key: &str,
        content_type: &str,
        body: Bytes,
    ) -> Result<UploadResult, AppError> {
        let size_bytes = body.len() as i64;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(body))
            .send()
            .await
            .map_err(map_sdk_err)?;

        Ok(UploadResult {
            key: key.to_string(),
            bucket: self.bucket.clone(),
            size_bytes,
        })
    }

    async fn presigned_url(&self, key: &str, expiry: Duration) -> Result<String, AppError> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(expiry)
            .build()
            .map_err(map_sdk_err)?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(map_sdk_err)?;

        Ok(presigned.uri().to_string())
    }

    async fn delete(&self, key: &str) -> Result<(), AppError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(map_sdk_err)?;

        Ok(())
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, AppError> {
        Ok(self
            .list_objects(prefix)
            .await?
            .into_iter()
            .map(|object| object.key)
            .collect())
    }

    async fn list_objects(&self, prefix: &str) -> Result<Vec<ObjectListEntry>, AppError> {
        let mut objects = Vec::new();
        let mut continuation_token = None;

        loop {
            let page = self
                .list_objects_page(prefix, continuation_token.as_deref(), 1_000)
                .await?;
            objects.extend(page.objects);
            continuation_token = page.next_continuation_token;
            if continuation_token.is_none() {
                break;
            }
        }

        Ok(objects)
    }

    async fn list_objects_page(
        &self,
        prefix: &str,
        continuation_token: Option<&str>,
        max_objects: i32,
    ) -> Result<ObjectListPage, AppError> {
        let mut request = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(prefix)
            .max_keys(max_objects);
        if let Some(token) = continuation_token {
            request = request.continuation_token(token);
        }

        let response = request.send().await.map_err(map_sdk_err)?;
        let objects = response
            .contents()
            .iter()
            .filter_map(|object| {
                let key = object.key()?.to_string();
                Some(ObjectListEntry {
                    key,
                    last_modified: object
                        .last_modified()
                        .and_then(|value| DateTime::<Utc>::from_timestamp(value.secs(), 0)),
                })
            })
            .collect();
        let next_continuation_token = response
            .is_truncated()
            .unwrap_or(false)
            .then(|| response.next_continuation_token().map(str::to_string))
            .flatten();
        Ok(ObjectListPage {
            objects,
            next_continuation_token,
        })
    }

    async fn exists(&self, key: &str) -> Result<bool, AppError> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(err) => {
                let service_err = err.into_service_error();
                if service_err.is_not_found() {
                    Ok(false)
                } else {
                    Err(map_sdk_err(service_err))
                }
            }
        }
    }

    async fn get_object(&self, key: &str) -> Result<ObjectData, AppError> {
        let ranged = self.get_object_range(key, None).await?;
        Ok(ObjectData {
            body: ranged.body,
            content_type: ranged.content_type,
            content_length: ranged.content_length,
        })
    }

    async fn get_object_range(
        &self,
        key: &str,
        range: Option<ByteRange>,
    ) -> Result<RangedObjectData, AppError> {
        let mut req = self.client.get_object().bucket(&self.bucket).key(key);
        if let Some(r) = range {
            req = req.range(format!("bytes={}-{}", r.start, r.end_inclusive));
        }

        let resp = req.send().await.map_err(map_sdk_err)?;

        let content_type = resp
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let content_length = resp.content_length().unwrap_or(0);
        let total_length = resp
            .content_range()
            .and_then(|cr| {
                cr.rsplit_once('/')
                    .and_then(|(_, total)| total.parse().ok())
            })
            .unwrap_or(content_length);

        let async_read = resp.body.into_async_read();
        let stream = tokio_util::io::ReaderStream::new(async_read);

        Ok(RangedObjectData {
            body: Box::pin(stream),
            content_type,
            content_length,
            total_length,
            range,
        })
    }
}
