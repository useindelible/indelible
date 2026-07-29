use std::pin::Pin;
use std::time::Duration;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures::Stream;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub key: String,
    pub bucket: String,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectListEntry {
    pub key: String,
    pub last_modified: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectListPage {
    pub objects: Vec<ObjectListEntry>,
    pub next_continuation_token: Option<String>,
}

/// Streamed object data returned by `get_object`.
pub struct ObjectData {
    pub body: Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>,
    pub content_type: String,
    pub content_length: i64,
}

/// Byte range (inclusive on both ends) for HTTP range requests.
#[derive(Debug, Clone, Copy)]
pub struct ByteRange {
    pub start: u64,
    pub end_inclusive: u64,
}

/// Ranged object data returned by `get_object_range`.
pub struct RangedObjectData {
    pub body: Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>,
    pub content_type: String,
    pub content_length: i64,
    pub total_length: i64,
    pub range: Option<ByteRange>,
}

#[async_trait::async_trait]
pub trait ObjectStorage: Send + Sync {
    async fn upload(
        &self,
        key: &str,
        content_type: &str,
        body: Bytes,
    ) -> Result<UploadResult, AppError>;

    /// Presigned download URL. Only the HTTP asset proxy may surface this to
    /// clients (as a redirect); it must never appear in a response body.
    async fn presigned_url(&self, key: &str, expiry: Duration) -> Result<String, AppError>;

    async fn delete(&self, key: &str) -> Result<(), AppError>;

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, AppError> {
        Ok(self
            .list_objects(prefix)
            .await?
            .into_iter()
            .map(|object| object.key)
            .collect())
    }

    async fn list_objects(&self, prefix: &str) -> Result<Vec<ObjectListEntry>, AppError> {
        let _ = prefix;
        Err(AppError::Repository(
            "object listing is not supported by this storage backend".into(),
        ))
    }

    async fn list_objects_page(
        &self,
        prefix: &str,
        continuation_token: Option<&str>,
        max_objects: i32,
    ) -> Result<ObjectListPage, AppError> {
        if max_objects <= 0 {
            return Err(AppError::Repository(Box::new(std::io::Error::other(
                "object list page size must be positive",
            ))));
        }
        let mut objects = self.list_objects(prefix).await?;
        objects.sort_by(|left, right| left.key.cmp(&right.key));
        let start = continuation_token
            .map(|token| objects.partition_point(|object| object.key.as_str() <= token))
            .unwrap_or(0);
        let page_size = usize::try_from(max_objects).unwrap_or(usize::MAX);
        let end = start.saturating_add(page_size).min(objects.len());
        let next_continuation_token = (end < objects.len())
            .then(|| {
                objects
                    .get(end.saturating_sub(1))
                    .map(|object| object.key.clone())
            })
            .flatten();
        Ok(ObjectListPage {
            objects: objects.into_iter().skip(start).take(page_size).collect(),
            next_continuation_token,
        })
    }

    async fn exists(&self, key: &str) -> Result<bool, AppError>;

    async fn get_object(&self, key: &str) -> Result<ObjectData, AppError>;

    /// Range-aware fetch. Default delegates to `get_object` and marks the
    /// returned data as the full object, so impls that do not need real HTTP
    /// Range support (e.g. in-memory test mocks) do not have to override.
    async fn get_object_range(
        &self,
        key: &str,
        range: Option<ByteRange>,
    ) -> Result<RangedObjectData, AppError> {
        let data = self.get_object(key).await?;
        if range.is_some() {
            return Err(AppError::Repository(
                "range requests not supported by this storage backend".into(),
            ));
        }
        Ok(RangedObjectData {
            body: data.body,
            content_type: data.content_type,
            content_length: data.content_length,
            total_length: data.content_length,
            range: None,
        })
    }
}

/// Download an object and collect it into a UTF-8 string. Shared by content
/// handlers and worker jobs that post-process stored HTML/JSON payloads.
pub async fn get_object_string(storage: &dyn ObjectStorage, key: &str) -> Result<String, AppError> {
    use futures::StreamExt;

    let object = storage.get_object(key).await?;
    let mut stream = object.body;
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|err| AppError::Repository(Box::new(err)))?;
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|err| AppError::Repository(Box::new(err)))
}
