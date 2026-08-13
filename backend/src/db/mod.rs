use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

/// Creates the SQLite connection pool and runs any pending migrations
/// from `database/migrations`. Called once at startup in main.rs.
///
/// `create_if_missing(true)` is required — sqlx's SQLite driver does NOT
/// create the database file on its own; connecting to a fresh
/// `sqlite://chess_king.db` with just `.connect()` fails with "unable to
/// open database file" on a first-ever run.
pub async fn init_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .busy_timeout(std::time::Duration::from_secs(10));

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await?;

    sqlx::migrate!("../database/migrations").run(&pool).await?;

    tracing::info!("database connected and migrations applied");
    Ok(pool)
}
