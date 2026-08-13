pub mod github_store;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

pub use github_store::{GitHubStore, SharedGitHubStore, StoreError};

/// Legacy sqlx pool — on Render use `sqlite::memory:` so **no file** is written.
/// Durable data goes through [`GitHubStore`] into the private GitHub repo.
pub async fn init_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    let url = if database_url.is_empty() {
        "sqlite::memory:".to_string()
    } else {
        database_url.to_string()
    };

    let options = SqliteConnectOptions::from_str(&url)?
        .create_if_missing(true)
        .busy_timeout(std::time::Duration::from_secs(10));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // In-memory still needs schema for code paths not yet migrated to GitHubStore
    if let Err(e) = sqlx::migrate!("../database/migrations").run(&pool).await {
        tracing::warn!("migrate skipped/failed (ok for pure GitHub mode): {e}");
    }

    tracing::info!("ephemeral sqlx pool ready (durable store = GitHub private repo when configured)");
    Ok(pool)
}

pub fn init_github_store_from_env() -> Option<SharedGitHubStore> {
    let token = std::env::var("GITHUB_DATA_TOKEN").ok()?;
    if token.is_empty() {
        return None;
    }
    let owner = std::env::var("GITHUB_DATA_OWNER").unwrap_or_else(|_| "web-coder-lab".into());
    let repo = std::env::var("GITHUB_DATA_REPO").unwrap_or_else(|_| "genius-clan-database".into());
    let branch = std::env::var("GITHUB_DATA_BRANCH").unwrap_or_else(|_| "main".into());
    tracing::info!("GitHub data store enabled: {owner}/{repo}@{branch}");
    Some(std::sync::Arc::new(GitHubStore::new(owner, repo, token, branch)))
}
