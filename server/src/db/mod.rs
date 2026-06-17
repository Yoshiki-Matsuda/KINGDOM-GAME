//! SQLite データベース接続・リポジトリ層

pub(crate) mod auth_repo;
pub(crate) mod player_repo;
pub(crate) mod world_repo;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

/// SQLite コネクションプールを作成する
pub(crate) async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    // 外部キー制約を有効化（SQLiteは接続ごとに設定が必要）
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;
    // WALモードで読み取り並行性を向上
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await?;

    Ok(pool)
}
