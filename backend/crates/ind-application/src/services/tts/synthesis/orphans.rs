use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use crate::AppError;
use crate::repos::tts_audio_asset::TtsAudioAssetRepository;
use crate::storage::{ObjectListEntry, ObjectStorage};

const TTS_OBJECT_PREFIX: &str = "tts/";
const TTS_ORPHAN_MIN_AGE: Duration = Duration::from_secs(30 * 60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtsOrphanSweepReport {
    pub scanned_objects: usize,
    pub referenced_objects: usize,
    pub deleted_objects: usize,
    pub failed_deletes: usize,
    pub next_continuation_cursor: Option<String>,
}

pub struct TtsOrphanSweeper {
    audio_assets: Arc<dyn TtsAudioAssetRepository>,
    storage: Arc<dyn ObjectStorage>,
}

impl TtsOrphanSweeper {
    pub fn new(
        audio_assets: Arc<dyn TtsAudioAssetRepository>,
        storage: Arc<dyn ObjectStorage>,
    ) -> Self {
        Self {
            audio_assets,
            storage,
        }
    }

    pub async fn sweep_page(
        &self,
        continuation_cursor: Option<&str>,
        max_objects: i32,
    ) -> Result<TtsOrphanSweepReport, AppError> {
        let page = self
            .storage
            .list_objects_page(TTS_OBJECT_PREFIX, continuation_cursor, max_objects)
            .await?;
        let objects = page.objects;
        let keys = objects
            .iter()
            .map(|object| object.key.clone())
            .collect::<Vec<_>>();
        let referenced = self
            .audio_assets
            .filter_existing_s3_keys(&keys)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();

        let referenced_objects = objects
            .iter()
            .filter(|object| referenced.contains(&object.key))
            .count();
        let mut deleted_objects = 0_usize;
        let mut failed_deletes = 0_usize;
        let now = chrono::Utc::now();

        for object in objects
            .iter()
            .filter(|object| should_delete_object(object, &referenced, now))
        {
            match self.storage.delete(&object.key).await {
                Ok(()) => deleted_objects += 1,
                Err(err) => {
                    failed_deletes += 1;
                    tracing::warn!(s3_key = %object.key, error = %err, "failed to delete orphaned TTS object");
                }
            }
        }

        Ok(TtsOrphanSweepReport {
            scanned_objects: objects.len(),
            referenced_objects,
            deleted_objects,
            failed_deletes,
            next_continuation_cursor: page.next_continuation_token,
        })
    }
}

fn should_delete_object(
    object: &ObjectListEntry,
    referenced: &HashSet<String>,
    now: chrono::DateTime<chrono::Utc>,
) -> bool {
    if referenced.contains(&object.key) {
        return false;
    }

    object
        .last_modified
        .and_then(|last_modified| now.signed_duration_since(last_modified).to_std().ok())
        .is_some_and(|age| age >= TTS_ORPHAN_MIN_AGE)
}
