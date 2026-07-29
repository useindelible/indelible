use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("failed to create connection pool: {0}")]
    PoolCreation(#[source] sqlx::Error),

    #[error("failed to run migrations: {0}")]
    Migration(#[source] sqlx::migrate::MigrateError),
}

pub async fn create_pool(database_url: &str) -> Result<PgPool, DbError> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(std::time::Duration::from_secs(600))
        .test_before_acquire(true)
        .connect(database_url)
        .await
        .map_err(DbError::PoolCreation)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), DbError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(DbError::Migration)
}
