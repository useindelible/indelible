mod cli;
#[cfg(feature = "db")]
mod commands;
#[cfg(feature = "db")]
mod config;
mod output;

pub const USAGE: &str = "\
ind — Indelible operator CLI

Usage:
  ind jobs dlq list [--limit N] [--json]
  ind jobs dlq show <dead_letter_id> [--json]
  ind jobs dlq replay <dead_letter_id> [--json]
  ind jobs dlq stats [--json]
  ind jobs recovery list [--status S] [--job-type T] [--limit N] [--json]
  ind search reindex [--page-size N] [--json]
  ind embeddings repair [--limit N] [--json]
  ind integrity stats [--json]
";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let command = cli::parse_args(std::env::args().skip(1))?;
    if matches!(command, cli::Command::Help) {
        print!("{USAGE}");
        return Ok(());
    }

    let output = run_db_command(command).await?;
    print!("{output}");
    Ok(())
}

#[cfg(feature = "db")]
async fn run_db_command(command: cli::Command) -> anyhow::Result<String> {
    use commands::CommandContext;
    use ind_persistence::repos::{
        PgBackgroundJobRecoveryRepository, PgDeadLetterRepository, PgEmbeddingBackfillRepository,
        PgIntegrityStatsRepository, PgSearchReindexRepository,
    };

    let config = config::CliConfig::load()?;
    let pool = ind_persistence::db::create_pool(&config.database_url).await?;
    let dead_letters = PgDeadLetterRepository::new(pool.clone());
    let recoveries = PgBackgroundJobRecoveryRepository::new(pool.clone());
    let search_reindex = PgSearchReindexRepository::new(pool.clone());
    let embeddings = PgEmbeddingBackfillRepository::new(pool.clone());
    let integrity = PgIntegrityStatsRepository::new(pool);

    commands::execute(
        command,
        CommandContext {
            dead_letters: &dead_letters,
            recoveries: &recoveries,
            search_reindex: &search_reindex,
            embeddings: &embeddings,
            integrity: &integrity,
        },
    )
    .await
}

#[cfg(not(feature = "db"))]
async fn run_db_command(_command: cli::Command) -> anyhow::Result<String> {
    anyhow::bail!("ind-cli was built without DB adapter support")
}
