use chrono::Utc;
use ind_application::error::AppError;
use ind_application::repos::account_purge::{AccountPurgeOutcome, AccountPurgeRepository};
use ind_application::repos::lifecycle_outbox::OutboxEntry;
use ind_domain::{DomainError, UserId, job_types};
use sqlx::PgPool;

use crate::repos::write_helpers::enqueue_outbox_tx;

pub struct PgAccountPurgeRepository {
    pool: PgPool,
}

impl PgAccountPurgeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    AppError::Repository(Box::new(err))
}

#[async_trait::async_trait]
impl AccountPurgeRepository for PgAccountPurgeRepository {
    async fn purge_account(&self, user_id: UserId) -> Result<AccountPurgeOutcome, AppError> {
        let uid = user_id.into_uuid();
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        // Serialize concurrent purges on the user row; unknown users are NotFound.
        let locked: Option<uuid::Uuid> =
            sqlx::query_scalar("SELECT id FROM users WHERE id = $1 FOR UPDATE")
                .bind(uid)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;
        if locked.is_none() {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "user",
                id: user_id.to_string(),
            }));
        }

        // Harvest object-storage keys from rows that are about to cascade away.
        // These deletes are redundant with the cascade; they exist for RETURNING.
        let archive_keys: Vec<String> = sqlx::query_scalar(
            "DELETE FROM archive_assets aa USING documents d \
             WHERE aa.document_id = d.id AND d.user_id = $1 \
             RETURNING aa.s3_key",
        )
        .bind(uid)
        .fetch_all(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        let tts_keys: Vec<String> =
            sqlx::query_scalar("DELETE FROM tts_audio_assets WHERE user_id = $1 RETURNING s3_key")
                .bind(uid)
                .fetch_all(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;

        let import_blobs: Vec<Option<String>> = sqlx::query_scalar(
            "DELETE FROM import_jobs WHERE user_id = $1 RETURNING raw_artifact_key",
        )
        .bind(uid)
        .fetch_all(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        let documents_deleted: i64 =
            sqlx::query_scalar("SELECT count(*) FROM documents WHERE user_id = $1")
                .bind(uid)
                .fetch_one(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;

        // Queue tables have no user foreign key, but their payloads embed the
        // user id (and sometimes user content, e.g. raw inbound email). Remove
        // every row the account owns before the cleanup job is enqueued below.
        let user_tag = user_id.to_string();
        for sql in [
            "DELETE FROM job_outbox WHERE payload->>'user_id' = $1",
            "DELETE FROM background_job_recoveries WHERE payload->>'user_id' = $1",
            "DELETE FROM dead_letter_jobs WHERE original_payload->>'user_id' = $1",
        ] {
            sqlx::query(sql)
                .bind(&user_tag)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;
        }

        // Enqueue the durable object-storage cleanup INSIDE the transaction.
        // job_outbox has no foreign key to users, so this row survives the
        // delete below and the worker retries until the bucket is clean.
        let prefixes = storage_prefixes(user_id);
        let mut keys = archive_keys;
        keys.extend(tts_keys);
        for blob in &import_blobs {
            keys.extend(import_artifact_keys(blob.as_deref()));
        }
        let residual = residual_keys(keys, &prefixes);
        let job = ind_domain::AccountStoragePurgeJob {
            user_id,
            prefixes,
            residual_keys: residual,
        };
        let payload = serde_json::to_value(&job).map_err(|e| AppError::Repository(Box::new(e)))?;
        enqueue_outbox_tx(
            &mut tx,
            &OutboxEntry {
                job_type: job_types::ACCOUNT_STORAGE_PURGE.into(),
                payload,
                dedupe_key: Some(format!("account-storage-purge:{user_id}")),
                available_at: Utc::now(),
            },
        )
        .await?;

        // The purge: every user-owned table rides ON DELETE CASCADE.
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(uid)
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;

        tx.commit().await.map_err(map_sqlx_error)?;

        Ok(AccountPurgeOutcome {
            documents_deleted: documents_deleted.max(0) as u64,
        })
    }
}

/// Every object-storage prefix that can contain objects owned by `user_id`.
///
/// Two spellings of the user id exist because some key builders interpolate
/// `UserId`'s `Display` (`usr_<uuid>`) while others interpolate the bare uuid.
/// The list is derived from the key-building call sites:
/// - `ind-http-api` account avatars: `usr_<uuid>/avatars/…`
/// - `ind-renderer` render/capture artifacts: `usr_<uuid>/itm_<uuid>/…`
/// - worker youtube + prepared content: `documents/<uuid>/<doc uuid>/…`
/// - worker readwise assets: `documents/usr_<uuid>/itm_<uuid>/…`
/// - provided content: `documents/provided/<uuid>/…`
/// - prepared readable html: `documents/prepared/<uuid>/…`
/// - article ToC: `documents/toc/<uuid>/…`
/// - library uploads: `documents/uploads/usr_<uuid>/…`
/// - imports: `imports/<uuid>/…`
/// - TTS audio: `tts/usr_<uuid>/…`
///
/// Missing either spelling orphans objects; keys that predate this scheme are
/// harvested separately from `archive_assets.s3_key` before their rows vanish.
pub(crate) fn storage_prefixes(user_id: UserId) -> Vec<String> {
    let uuid = user_id.into_uuid();
    let disp = user_id.to_string();
    vec![
        format!("{disp}/"),
        format!("documents/{uuid}/"),
        format!("documents/{disp}/"),
        format!("documents/provided/{uuid}/"),
        format!("documents/prepared/{uuid}/"),
        format!("documents/toc/{uuid}/"),
        format!("documents/uploads/{disp}/"),
        format!("imports/{uuid}/"),
        format!("tts/{disp}/"),
    ]
}

/// `import_jobs.raw_artifact_key` holds a JSON object string of the shape
/// `{"csv_key":…,"zip_key":…,"opml_key":…}`. Absent, malformed, or non-object
/// values yield no keys.
pub(crate) fn import_artifact_keys(raw: Option<&str>) -> Vec<String> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Vec::new();
    };
    ["csv_key", "zip_key", "opml_key"]
        .iter()
        .filter_map(|field| map.get(*field).and_then(|v| v.as_str()).map(String::from))
        .collect()
}

/// Keys not covered by any user prefix (legacy shapes). Keeping only these in
/// the cleanup-job payload keeps it bounded: for accounts created under the
/// current key scheme this is empty.
pub(crate) fn residual_keys(keys: Vec<String>, prefixes: &[String]) -> Vec<String> {
    keys.into_iter()
        .filter(|key| !prefixes.iter().any(|prefix| key.starts_with(prefix)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_prefixes_cover_both_user_id_spellings() {
        let user_id: UserId = "usr_01890000-0000-7000-8000-000000000001".parse().unwrap();
        let prefixes = storage_prefixes(user_id);
        let uuid = "01890000-0000-7000-8000-000000000001";
        let expected = [
            format!("usr_{uuid}/"),
            format!("documents/{uuid}/"),
            format!("documents/usr_{uuid}/"),
            format!("documents/provided/{uuid}/"),
            format!("documents/prepared/{uuid}/"),
            format!("documents/toc/{uuid}/"),
            format!("documents/uploads/usr_{uuid}/"),
            format!("imports/{uuid}/"),
            format!("tts/usr_{uuid}/"),
        ];
        assert_eq!(prefixes, expected);
    }

    #[test]
    fn import_artifact_keys_parses_the_json_string_and_tolerates_absence() {
        let raw = r#"{"csv_key":"imports/u/j/library.csv","zip_key":"imports/u/j/archive.zip","opml_key":"imports/u/j/feeds.opml"}"#;
        assert_eq!(
            import_artifact_keys(Some(raw)),
            vec![
                "imports/u/j/library.csv",
                "imports/u/j/archive.zip",
                "imports/u/j/feeds.opml"
            ]
        );
        assert_eq!(
            import_artifact_keys(Some(
                r#"{"csv_key":"imports/u/j/library.csv","zip_key":null}"#
            )),
            vec!["imports/u/j/library.csv"]
        );
        assert!(import_artifact_keys(None).is_empty());
        assert!(import_artifact_keys(Some("not json")).is_empty());
        assert!(import_artifact_keys(Some(r#""a-string""#)).is_empty());
    }

    #[test]
    fn residual_keys_drops_keys_already_covered_by_a_prefix() {
        let prefixes = vec!["tts/usr_a/".to_string()];
        let keys = vec![
            "tts/usr_a/x.mp3".to_string(),
            "legacy/doc/readable.html".to_string(),
        ];
        assert_eq!(
            residual_keys(keys, &prefixes),
            vec!["legacy/doc/readable.html"]
        );
    }
}
