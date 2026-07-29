use std::collections::HashMap;

use futures::StreamExt;
use ind_application::AppError;
use ind_ingest::archive_limits::{ArchiveLimits, ArchiveReadBudget};

use super::types::ZipEntry;
use crate::context::IntegrationJobDeps;

pub(super) async fn download_bytes(
    storage: &dyn ind_application::storage::ObjectStorage,
    key: &str,
) -> Result<Vec<u8>, AppError> {
    let obj = storage.get_object(key).await?;
    let mut body = obj.body;
    let mut buf = Vec::new();
    while let Some(chunk) = body.next().await {
        let chunk = chunk.map_err(|e| AppError::Repository(Box::new(e)))?;
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

pub(super) async fn download_zip(
    _ctx: &IntegrationJobDeps,
    storage: &dyn ind_application::storage::ObjectStorage,
    key: &str,
) -> Result<HashMap<String, ZipEntry>, AppError> {
    let bytes = download_bytes(storage, key).await?;
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| AppError::Repository(format!("ZIP open error: {e}").into()))?;

    if archive.len() > ArchiveLimits::IMPORT.max_entries {
        return Err(AppError::Repository(
            format!("ZIP has too many entries ({})", archive.len()).into(),
        ));
    }
    let mut budget = ArchiveReadBudget::new(ArchiveLimits::IMPORT);

    let mut map = HashMap::new();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::Repository(format!("ZIP entry error: {e}").into()))?;

        if file.is_dir() {
            continue;
        }

        let path = file.name().to_string();
        let ulid =
            extract_ulid_from_filename(&path).unwrap_or_else(|| stable_zip_external_id(&path));
        let ext = std::path::Path::new(&path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        let title = title_from_zip_path(&path, &ulid);

        // An over-budget entry in an import archive is a zip bomb (or corrupt
        // archive), not a benign condition: fail the whole import rather than
        // silently producing a partial result.
        let bytes = budget.read_capped(&mut file).map_err(|e| {
            AppError::Repository(format!("ZIP entry '{path}' exceeds import limit: {e}").into())
        })?;

        map.insert(
            ulid,
            ZipEntry {
                path,
                title,
                extension: ext,
                bytes,
            },
        );
    }

    Ok(map)
}

/// Extracts the trailing ULID/ID from a Readwise ZIP filename like
/// `Library/Some Title (01ABCDEFG).html`.
pub(super) fn extract_ulid_from_filename(name: &str) -> Option<String> {
    let stem = std::path::Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())?;

    let open = stem.rfind('(')?;
    let close = stem.rfind(')')?;
    if close <= open {
        return None;
    }
    let id = stem[open + 1..close].trim().to_string();
    if id.is_empty() { None } else { Some(id) }
}

pub(super) fn title_from_zip_path(path: &str, ulid: &str) -> String {
    let stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path);

    let title = stem
        .rfind('(')
        .map(|open| stem[..open].trim())
        .unwrap_or(stem)
        .trim_matches(|c: char| c == '-' || c == '_' || c.is_whitespace())
        .to_string();

    if title.is_empty() {
        format!("Readwise import: {ulid}")
    } else {
        title
    }
}

pub(super) fn stable_zip_external_id(path: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in path.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("zip:{hash:016x}")
}
