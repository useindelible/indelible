use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliConfig {
    pub database_url: String,
}

impl CliConfig {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL")
            .context("DATABASE_URL must be set for operator CLI commands")?;
        Ok(Self { database_url })
    }
}
