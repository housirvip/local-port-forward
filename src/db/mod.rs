use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};
use std::str::FromStr;

const MIGRATION_001: &str = include_str!("migrations/001_init.sql");

pub async fn open(path: &str) -> Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{path}"))?
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await?;
    sqlx::query(MIGRATION_001).execute(&pool).await?;
    Ok(pool)
}
