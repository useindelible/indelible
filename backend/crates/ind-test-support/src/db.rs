// `dtor`'s no-`unsafe` form is deprecated to nudge callers toward `unsafe fn`,
// but the `unsafe` form would trip this crate's `unsafe_code = "deny"`. The
// non-unsafe form is genuinely unsafe-free here, so its deprecation is allowed.
#![allow(deprecated)]

use std::sync::{Arc, Mutex as StdMutex, OnceLock};

use aws_sdk_s3::config::{Credentials, Region};
use ind_application::storage::ObjectStorage;
use ind_persistence::storage::S3Client;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use testcontainers::ContainerAsync;
use testcontainers::core::ContainerPort;
use testcontainers::core::WaitFor;
use testcontainers::core::wait::LogWaitStrategy;
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use tokio::sync::Semaphore;

const MINIO_USER: &str = "minioadmin";
const MINIO_PASSWORD: &str = "minioadmin";
pub const TEST_S3_BUCKET: &str = "indelible-test";

struct SharedContainers {
    pg_container: StdMutex<Option<ContainerAsync<GenericImage>>>,
    minio_container: StdMutex<Option<ContainerAsync<GenericImage>>>,
    /// Dedicated runtime that owns the admin pool and container handles.
    /// Outlives all test runtimes, preventing "runtime is shutting down"
    /// errors when individual `#[tokio::test]` runtimes exit.
    runtime: tokio::runtime::Runtime,
    admin_pool: PgPool,
    pg_host: String,
    pg_port: u16,
    minio_endpoint: String,
    db_create_semaphore: Semaphore,
}

static SHARED: OnceLock<Arc<SharedContainers>> = OnceLock::new();

/// Stops and removes the shared containers at process exit. Rust never runs
/// `Drop` for statics, and `testcontainers` reaps containers via `Drop`, so
/// without this the shared Postgres/MinIO containers outlive the test binary and
/// leak across runs. `dtor` keeps `unsafe_code` denied (the exit-hook FFI is
/// encapsulated in the crate) where a hand-rolled `atexit` could not.
#[dtor::dtor]
fn cleanup_shared_containers() {
    let Some(shared) = SHARED.get() else {
        return;
    };
    let pg = shared.pg_container.lock().ok().and_then(|mut g| g.take());
    let minio = shared
        .minio_container
        .lock()
        .ok()
        .and_then(|mut g| g.take());
    if pg.is_none() && minio.is_none() {
        return;
    }
    shared.runtime.block_on(async {
        shared.admin_pool.close().await;
        if let Some(container) = pg {
            let _ = container.stop().await;
            let _ = container.rm().await;
        }
        if let Some(container) = minio {
            let _ = container.stop().await;
            let _ = container.rm().await;
        }
    });
}

impl SharedContainers {
    fn get_or_init() -> Arc<SharedContainers> {
        SHARED
            .get_or_init(|| {
                // Spawn initialization on a new OS thread to avoid
                // "cannot start a runtime from within a runtime".
                std::thread::spawn(|| {
                    let runtime = tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                        .expect("failed to create shared test runtime");

                    let shared = runtime.block_on(async { Self::init_async().await });

                    Arc::new(SharedContainers {
                        pg_container: StdMutex::new(Some(shared.0)),
                        minio_container: StdMutex::new(Some(shared.1)),
                        runtime,
                        admin_pool: shared.2,
                        pg_host: shared.3,
                        pg_port: shared.4,
                        minio_endpoint: shared.5,
                        db_create_semaphore: Semaphore::new(10),
                    })
                })
                .join()
                .expect("shared container init thread panicked")
            })
            .clone()
    }

    #[allow(clippy::type_complexity)]
    async fn init_async() -> (
        ContainerAsync<GenericImage>,
        ContainerAsync<GenericImage>,
        PgPool,
        String,
        u16,
        String,
    ) {
        let (pg_container, pg_host, pg_port) = Self::start_postgres().await;
        let (minio_container, minio_endpoint) = Self::start_minio().await;

        let admin_url = format!("postgres://test:test@{pg_host}:{pg_port}/postgres");
        let admin_pool = Self::connect_admin_pool(&admin_url).await;

        Self::create_template_db(&admin_pool, &pg_host, pg_port).await;
        Self::create_minio_bucket(&minio_endpoint).await;

        (
            pg_container,
            minio_container,
            admin_pool,
            pg_host,
            pg_port,
            minio_endpoint,
        )
    }

    async fn connect_admin_pool(admin_url: &str) -> PgPool {
        const ATTEMPTS: usize = 25;
        let mut last_error = None;

        for attempt in 0..ATTEMPTS {
            match PgPoolOptions::new()
                .max_connections(10)
                .acquire_timeout(std::time::Duration::from_secs(120))
                .connect(admin_url)
                .await
            {
                Ok(pool) => return pool,
                Err(error) => last_error = Some(error),
            }
            if attempt + 1 < ATTEMPTS {
                // Container log readiness can precede stable host-port routing
                // when several test binaries start Docker services concurrently.
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        }

        panic!(
            "failed to connect admin pool to postgres after {ATTEMPTS} attempts: {}",
            last_error.expect("at least one connection attempt")
        );
    }

    async fn start_postgres() -> (ContainerAsync<GenericImage>, String, u16) {
        let image = GenericImage::new("pgvector/pgvector", "pg17")
            .with_exposed_port(ContainerPort::Tcp(5432))
            .with_wait_for(WaitFor::Log(LogWaitStrategy::stderr(
                "database system is ready to accept connections",
            )));

        let container = image
            .with_env_var("POSTGRES_DB", "postgres")
            .with_env_var("POSTGRES_USER", "test")
            .with_env_var("POSTGRES_PASSWORD", "test")
            .with_cmd([
                "postgres",
                "-c",
                "max_connections=300",
                "-c",
                "shared_buffers=128MB",
            ])
            .with_startup_timeout(std::time::Duration::from_secs(60))
            .start()
            .await
            .expect("failed to start pgvector container");

        let port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("failed to get pg mapped port");

        let host = container
            .get_host()
            .await
            .expect("failed to get pg container host");

        (container, host.to_string(), port)
    }

    async fn start_minio() -> (ContainerAsync<GenericImage>, String) {
        let image = GenericImage::new("minio/minio", "latest")
            .with_exposed_port(ContainerPort::Tcp(9000))
            .with_wait_for(WaitFor::Log(LogWaitStrategy::stderr("API:")));

        let container = image
            .with_env_var("MINIO_ROOT_USER", MINIO_USER)
            .with_env_var("MINIO_ROOT_PASSWORD", MINIO_PASSWORD)
            .with_cmd(["server", "/data"])
            .with_startup_timeout(std::time::Duration::from_secs(60))
            .start()
            .await
            .expect("failed to start minio container");

        // MinIO logs "API:" (the wait condition) a moment before Docker reliably
        // publishes the port mapping, so an immediate query can race and return a
        // transient PortNotExposed. Retry briefly to absorb that race.
        let mut minio_port = None;
        for _ in 0..25 {
            match container.get_host_port_ipv4(9000).await {
                Ok(p) => {
                    minio_port = Some(p);
                    break;
                }
                Err(_) => tokio::time::sleep(std::time::Duration::from_millis(200)).await,
            }
        }
        let port = minio_port.expect("failed to get minio mapped port after retries");

        let host = container
            .get_host()
            .await
            .expect("failed to get minio container host");

        let endpoint = format!("http://{host}:{port}");
        (container, endpoint)
    }

    async fn create_template_db(admin_pool: &PgPool, host: &str, port: u16) {
        sqlx::query("CREATE DATABASE template_ind")
            .execute(admin_pool)
            .await
            .expect("failed to create template database");

        let template_url = format!("postgres://test:test@{host}:{port}/template_ind");
        let template_pool = PgPoolOptions::new()
            .max_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&template_url)
            .await
            .expect("failed to connect to template database");

        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(&template_pool)
            .await
            .expect("failed to create vector extension in template");

        ind_persistence::run_migrations(&template_pool)
            .await
            .expect("failed to run migrations on template database");

        template_pool.close().await;
    }

    async fn create_minio_bucket(endpoint: &str) {
        Self::s3_client(endpoint)
            .await
            .create_bucket()
            .bucket(TEST_S3_BUCKET)
            .send()
            .await
            .expect("failed to create minio test bucket");
    }

    async fn s3_client(endpoint: &str) -> aws_sdk_s3::Client {
        let credentials = Credentials::new(MINIO_USER, MINIO_PASSWORD, None, None, "test");
        let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region(Region::new("us-east-1"))
            .credentials_provider(credentials)
            .load()
            .await;
        let s3_config = aws_sdk_s3::config::Builder::from(&sdk_config)
            .force_path_style(true)
            .build();
        aws_sdk_s3::Client::from_conf(s3_config)
    }

    async fn build_s3_client(&self) -> S3Client {
        S3Client::from_client(
            Self::s3_client(&self.minio_endpoint).await,
            TEST_S3_BUCKET.to_string(),
        )
    }
}

pub struct TestDb {
    shared: Arc<SharedContainers>,
    pool: PgPool,
    db_name: String,
}

impl TestDb {
    pub async fn new() -> Self {
        let shared = SharedContainers::get_or_init();
        let id = uuid::Uuid::now_v7().simple().to_string();
        let db_name = format!("test_{id}");

        // Limit concurrent CREATE DATABASE to avoid overwhelming PG
        let _permit = shared
            .db_create_semaphore
            .acquire()
            .await
            .expect("semaphore closed unexpectedly");

        // Run CREATE DATABASE on the shared runtime so the admin pool
        // connections stay alive regardless of which test runtime we're on.
        let admin_pool = shared.admin_pool.clone();
        let db_name_clone = db_name.clone();
        shared
            .runtime
            .spawn(async move {
                let create_sql =
                    format!("CREATE DATABASE \"{db_name_clone}\" TEMPLATE template_ind",);
                sqlx::query(&create_sql)
                    .execute(&admin_pool)
                    .await
                    .unwrap_or_else(|e| {
                        panic!("failed to create test database {db_name_clone}: {e}")
                    });
            })
            .await
            .expect("admin task panicked");

        drop(_permit);

        let url = format!(
            "postgres://test:test@{}:{}/{db_name}",
            shared.pg_host, shared.pg_port
        );

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&url)
            .await
            .unwrap_or_else(|e| panic!("failed to connect to test database {db_name}: {e}"));

        Self {
            shared,
            pool,
            db_name,
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn storage(&self) -> Arc<dyn ObjectStorage> {
        Arc::new(self.shared.build_s3_client().await)
    }

    pub fn s3_endpoint(&self) -> &str {
        &self.shared.minio_endpoint
    }

    pub fn bucket(&self) -> &'static str {
        TEST_S3_BUCKET
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        let admin_pool = self.shared.admin_pool.clone();
        let db_name = self.db_name.clone();
        let pool = self.pool.clone();
        let runtime_handle = self.shared.runtime.handle().clone();

        // Fire-and-forget cleanup on the shared runtime.
        runtime_handle.spawn(async move {
            pool.close().await;
            let drop_sql = format!("DROP DATABASE IF EXISTS \"{db_name}\" WITH (FORCE)");
            let _ = sqlx::query(&drop_sql).execute(&admin_pool).await;
        });
    }
}
