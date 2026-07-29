use std::sync::Arc;

use crate::config::ServerConfig;
use ind_application::ports::ExtensionSaveOperations;
use ind_application::storage::ObjectStorage;
use ind_persistence::repos::{
    PgDocumentAssetRepository, PgDocumentLifecycle, PgUserPreferencesRepository,
};

pub(super) struct StorageServices {
    pub storage: Option<Arc<dyn ObjectStorage>>,
}

pub(super) async fn build_storage_services(
    config: &ServerConfig,
) -> anyhow::Result<StorageServices> {
    if !config.storage.s3_enabled {
        tracing::info!("S3 storage disabled by configuration");
        return Ok(StorageServices { storage: None });
    }

    let s3 = ind_persistence::storage::S3Client::from_config(config.storage.s3_config()?);
    let storage: Arc<dyn ObjectStorage> = Arc::new(s3);
    tracing::info!(
        mode = ?config.storage.asset_serving_mode,
        "S3 storage configured with asset serving"
    );
    Ok(StorageServices {
        storage: Some(storage),
    })
}

pub(super) fn build_extension_save_ops(
    pool: &sqlx::PgPool,
    storage: Option<&Arc<dyn ObjectStorage>>,
    url_guard: Arc<dyn ind_application::ports::OutboundUrlGuard>,
) -> Option<Arc<dyn ExtensionSaveOperations>> {
    if let Some(s3_client) = storage {
        let save_service = ind_application::ExtensionSaveService::new(
            Arc::new(PgDocumentLifecycle::new(pool.clone())),
            Arc::new(PgDocumentAssetRepository::new(pool.clone())),
            s3_client.clone(),
            Arc::new(PgUserPreferencesRepository::new(pool.clone())),
            url_guard,
        );
        tracing::info!("extension save service initialized with S3");
        Some(Arc::new(save_service))
    } else {
        tracing::warn!("S3 not configured — extension save endpoints will return 400");
        None
    }
}
