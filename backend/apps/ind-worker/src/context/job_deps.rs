use std::sync::Arc;

use ind_ai::{AiActionRunner, EmbeddingIndexer};
use ind_application::handlers::feed::FeedPollScheduleConfig;
use ind_application::renderer::RendererClient;
use ind_application::repos::background_job_recovery::BackgroundJobRecoveryRepository;
use ind_application::repos::collection::CollectionRepository;
use ind_application::repos::document::DocumentRepository;
use ind_application::repos::document_asset::DocumentAssetRepository;
use ind_application::repos::document_lifecycle::DocumentLifecycle;
use ind_application::repos::document_reprocess::DocumentReprocessRepository;
use ind_application::repos::email_ingest::EmailIngestLogRepository;
use ind_application::repos::email_sender::EmailSenderRepository;
use ind_application::repos::email_unsubscribe_target::EmailUnsubscribeTargetRepository;
use ind_application::repos::embedding_backfill::EmbeddingBackfillRepository;
use ind_application::repos::feed::FeedRepository;
use ind_application::repos::feed_delivery::FeedDeliveryRepository;
use ind_application::repos::highlight::HighlightRepository;
use ind_application::repos::import_job::ImportJobRepository;
use ind_application::repos::integrity::IntegrityStatsRepository;
use ind_application::repos::library::LibraryRepository;
use ind_application::repos::maintenance::MaintenanceTaskRepository;
use ind_application::repos::mila_config::MilaConfigRepository;
use ind_application::repos::outbox::JobOutboxRepository;
use ind_application::repos::search_reindex::SearchReindexRepository;
use ind_application::repos::tag::TagRepository;
use ind_application::repos::user::UserRepository;
use ind_application::repos::user_document_state::UserDocumentStateRepository;
use ind_application::repos::user_preferences::UserPreferencesRepository;
use ind_application::repos::webhook::WebhookRepository;
use ind_application::services::tts::synthesis::TtsOrphanSweeper;
use ind_application::storage::ObjectStorage;
use ind_integrations::email::InboundEmailProvider;
use ind_search::SearchIndexer;
use sqlx::PgPool;

use super::{NotionJobDeps, WorkerContext};
use crate::jobs::email_unsubscribe::OneClickPolicy;

#[derive(Clone)]
pub struct AiSearchJobDeps {
    pub document_repo: Arc<dyn DocumentRepository>,
    pub embedding_indexer: Arc<EmbeddingIndexer>,
    pub ai_action_runner: Arc<AiActionRunner>,
    pub outbox_repo: Arc<dyn JobOutboxRepository>,
    pub search_reindex_repo: Arc<dyn SearchReindexRepository>,
    pub search_indexer: Arc<SearchIndexer>,
}

#[derive(Clone)]
pub struct CaptureJobDeps {
    pub document_repo: Arc<dyn DocumentRepository>,
    pub document_asset_repo: Arc<dyn DocumentAssetRepository>,
    pub document_reprocess_repo: Arc<dyn DocumentReprocessRepository>,
    pub object_storage: Option<Arc<dyn ObjectStorage>>,
    pub outbox_repo: Arc<dyn JobOutboxRepository>,
    pub egress_policy: ind_egress::EgressPolicy,
    pub youtube_player_base_url: Option<String>,
    pub feed: FeedJobDeps,
}

#[derive(Clone)]
pub struct EmailJobDeps {
    pub feed_repo: Arc<dyn FeedRepository>,
    pub feed_delivery_repo: Arc<dyn FeedDeliveryRepository>,
    pub object_storage: Option<Arc<dyn ObjectStorage>>,
    pub tag_repo: Arc<dyn TagRepository>,
    pub email_ingest_provider: Option<Arc<dyn InboundEmailProvider>>,
    pub email_ingest_log_repo: Option<Arc<dyn EmailIngestLogRepository>>,
    pub email_sender_repo: Option<Arc<dyn EmailSenderRepository>>,
    pub email_unsubscribe_target_repo: Option<Arc<dyn EmailUnsubscribeTargetRepository>>,
    pub email_unsubscribe_url_policy: OneClickPolicy,
    pub user_repo: Option<Arc<dyn UserRepository>>,
    pub egress_policy: ind_egress::EgressPolicy,
    pub lifecycle: Arc<dyn DocumentLifecycle>,
}

#[derive(Clone)]
pub struct FeedJobDeps {
    pub renderer: Arc<dyn RendererClient>,
    pub collection_repo: Arc<dyn CollectionRepository>,
    pub feed_repo: Arc<dyn FeedRepository>,
    pub feed_delivery_repo: Arc<dyn FeedDeliveryRepository>,
    pub document_repo: Arc<dyn DocumentRepository>,
    pub document_asset_repo: Arc<dyn DocumentAssetRepository>,
    pub outbox_repo: Arc<dyn JobOutboxRepository>,
    pub object_storage: Option<Arc<dyn ObjectStorage>>,
    pub feed_poll_schedule: FeedPollScheduleConfig,
    pub egress_policy: ind_egress::EgressPolicy,
    pub s3_bucket: String,
    pub lifecycle: Arc<dyn DocumentLifecycle>,
    pub user_preferences_repo: Arc<dyn UserPreferencesRepository>,
    pub worker_id: String,
}

#[derive(Clone)]
pub struct IntegrationJobDeps {
    pub notion_job_deps: Option<Arc<NotionJobDeps>>,
    pub document_repo: Arc<dyn DocumentRepository>,
    pub document_asset_repo: Arc<dyn DocumentAssetRepository>,
    pub mila_config_repo: Arc<dyn MilaConfigRepository>,
    pub object_storage: Option<Arc<dyn ObjectStorage>>,
    pub feed_repo: Arc<dyn FeedRepository>,
    pub import_job_repo: Arc<dyn ImportJobRepository>,
    pub outbox_repo: Arc<dyn JobOutboxRepository>,
    pub library_repo: Arc<dyn LibraryRepository>,
    pub tag_repo: Arc<dyn TagRepository>,
    pub user_document_state_repo: Arc<dyn UserDocumentStateRepository>,
    pub egress_policy: ind_egress::EgressPolicy,
    pub pool: PgPool,
    pub lifecycle: Arc<dyn DocumentLifecycle>,
    pub export_summary_provider: Arc<dyn ind_application::export_summary::ExportSummaryProvider>,
    pub highlight_repo: Option<Arc<dyn HighlightRepository>>,
}

#[derive(Clone)]
pub struct RecoveryJobDeps {
    pub background_recovery_repo: Arc<dyn BackgroundJobRecoveryRepository>,
    pub outbox_repo: Arc<dyn JobOutboxRepository>,
    pub embedding_backfill_repo: Arc<dyn EmbeddingBackfillRepository>,
    pub mila_platform_defaults: ind_domain::MilaPlatformDefaults,
    pub integrity_stats_repo: Arc<dyn IntegrityStatsRepository>,
    pub maintenance_task_repo: Arc<dyn MaintenanceTaskRepository>,
    pub tts_orphan_sweeper: Option<Arc<TtsOrphanSweeper>>,
    pub worker_id: String,
    pub auto_heal_lease_secs: i64,
    pub maintenance_lease_secs: i64,
    pub auto_heal_interval_secs: u64,
    pub auto_heal_batch_size: i64,
    pub embedding_repair_interval_secs: u64,
    pub integrity_interval_secs: u64,
    pub tts_orphan_interval_secs: u64,
    pub tts_orphan_page_size: i32,
    pub job_recovery_max_attempts: i32,
    pub job_recovery_batch_size: i64,
}

#[derive(Clone)]
pub struct WebhookJobDeps {
    pub webhook_repo: Arc<dyn WebhookRepository>,
    pub credential_cipher: Option<Arc<ind_auth::CredentialCipher>>,
    pub webhook_http: ind_egress::GuardedHttpClient,
}

pub trait IndexQueueContext {
    fn document_repo(&self) -> &Arc<dyn DocumentRepository>;
    fn outbox_repo(&self) -> &Arc<dyn JobOutboxRepository>;
}

macro_rules! impl_index_queue_context {
    ($($type:ty),+ $(,)?) => {
        $(
            impl IndexQueueContext for $type {
                fn document_repo(&self) -> &Arc<dyn DocumentRepository> {
                    &self.document_repo
                }

                fn outbox_repo(&self) -> &Arc<dyn JobOutboxRepository> {
                    &self.outbox_repo
                }
            }
        )+
    };
}

impl_index_queue_context!(
    AiSearchJobDeps,
    CaptureJobDeps,
    FeedJobDeps,
    IntegrationJobDeps
);

impl WorkerContext {
    pub fn ai_search_jobs(&self) -> AiSearchJobDeps {
        AiSearchJobDeps {
            document_repo: self.document_repo.clone(),
            embedding_indexer: self.embedding_indexer.clone(),
            ai_action_runner: self.ai_action_runner.clone(),
            outbox_repo: self.outbox_repo.clone(),
            search_reindex_repo: self.search_reindex_repo.clone(),
            search_indexer: self.search_indexer.clone(),
        }
    }

    pub fn capture_jobs(&self) -> CaptureJobDeps {
        CaptureJobDeps {
            document_repo: self.document_repo.clone(),
            document_asset_repo: self.document_asset_repo.clone(),
            document_reprocess_repo: self.document_reprocess_repo.clone(),
            object_storage: self.object_storage.clone(),
            outbox_repo: self.outbox_repo.clone(),
            egress_policy: self.egress_policy.clone(),
            youtube_player_base_url: self.youtube_player_base_url.clone(),
            feed: self.feed_jobs(),
        }
    }

    pub fn email_jobs(&self) -> EmailJobDeps {
        EmailJobDeps {
            feed_repo: self.feed_repo.clone(),
            feed_delivery_repo: self.feed_delivery_repo.clone(),
            object_storage: self.object_storage.clone(),
            tag_repo: self.tag_repo.clone(),
            email_ingest_provider: self.email_ingest_provider.clone(),
            email_ingest_log_repo: self.email_ingest_log_repo.clone(),
            email_sender_repo: self.email_sender_repo.clone(),
            email_unsubscribe_target_repo: self.email_unsubscribe_target_repo.clone(),
            email_unsubscribe_url_policy: self.email_unsubscribe_url_policy,
            user_repo: self.user_repo.clone(),
            egress_policy: self.egress_policy.clone(),
            lifecycle: self.lifecycle.clone(),
        }
    }

    pub fn feed_jobs(&self) -> FeedJobDeps {
        FeedJobDeps {
            renderer: self.renderer.clone(),
            collection_repo: self.collection_repo.clone(),
            feed_repo: self.feed_repo.clone(),
            feed_delivery_repo: self.feed_delivery_repo.clone(),
            document_repo: self.document_repo.clone(),
            document_asset_repo: self.document_asset_repo.clone(),
            outbox_repo: self.outbox_repo.clone(),
            object_storage: self.object_storage.clone(),
            feed_poll_schedule: self.feed_poll_schedule,
            egress_policy: self.egress_policy.clone(),
            s3_bucket: self.s3_bucket.clone(),
            lifecycle: self.lifecycle.clone(),
            user_preferences_repo: self.user_preferences_repo.clone(),
            worker_id: self.worker_id.clone(),
        }
    }

    pub fn integration_jobs(&self) -> IntegrationJobDeps {
        IntegrationJobDeps {
            notion_job_deps: self.notion_job_deps.clone(),
            document_repo: self.document_repo.clone(),
            document_asset_repo: self.document_asset_repo.clone(),
            mila_config_repo: self.mila_config_repo.clone(),
            object_storage: self.object_storage.clone(),
            feed_repo: self.feed_repo.clone(),
            import_job_repo: self.import_job_repo.clone(),
            outbox_repo: self.outbox_repo.clone(),
            library_repo: self.library_repo.clone(),
            tag_repo: self.tag_repo.clone(),
            user_document_state_repo: self.user_document_state_repo.clone(),
            egress_policy: self.egress_policy.clone(),
            pool: self.pool.clone(),
            lifecycle: self.lifecycle.clone(),
            export_summary_provider: self.export_summary_provider.clone(),
            highlight_repo: self.highlight_repo.clone(),
        }
    }

    pub fn recovery_jobs(&self) -> RecoveryJobDeps {
        RecoveryJobDeps {
            background_recovery_repo: self.background_recovery_repo.clone(),
            outbox_repo: self.outbox_repo.clone(),
            embedding_backfill_repo: self.embedding_backfill_repo.clone(),
            mila_platform_defaults: self.mila_platform_defaults.clone(),
            integrity_stats_repo: self.integrity_stats_repo.clone(),
            maintenance_task_repo: self.maintenance_task_repo.clone(),
            tts_orphan_sweeper: self.tts_orphan_sweeper.clone(),
            worker_id: self.worker_id.clone(),
            auto_heal_lease_secs: self.auto_heal_lease_secs,
            maintenance_lease_secs: self.maintenance_lease_secs,
            auto_heal_interval_secs: self.auto_heal_interval_secs,
            auto_heal_batch_size: self.auto_heal_batch_size,
            embedding_repair_interval_secs: self.embedding_repair_interval_secs,
            integrity_interval_secs: self.integrity_interval_secs,
            tts_orphan_interval_secs: self.tts_orphan_interval_secs,
            tts_orphan_page_size: self.tts_orphan_page_size,
            job_recovery_max_attempts: self.job_recovery_max_attempts,
            job_recovery_batch_size: self.job_recovery_batch_size,
        }
    }

    pub fn webhook_jobs(&self) -> WebhookJobDeps {
        WebhookJobDeps {
            webhook_repo: self.webhook_repo.clone(),
            credential_cipher: self.credential_cipher.clone(),
            webhook_http: self.webhook_http.clone(),
        }
    }
}
