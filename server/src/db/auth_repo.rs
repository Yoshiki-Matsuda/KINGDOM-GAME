//! 認証ユーザーのDB読み書き

use sqlx::sqlite::SqlitePool;
use sqlx::Row;

/// 認証ユーザー（DB対応版）
#[derive(Debug, Clone)]
pub(crate) struct DbAuthUser {
    pub(crate) username: String,
    pub(crate) player_id: String,
    pub(crate) password_hash: String,
}

/// ユーザー名で検索
pub(crate) async fn find_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Option<DbAuthUser> {
    let row = sqlx::query(
        "SELECT username, player_id, password_hash FROM auth_users WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .ok()??;

    Some(DbAuthUser {
        username: row.get("username"),
        player_id: row.get("player_id"),
        password_hash: row.get("password_hash"),
    })
}

/// ユーザーを追加
pub(crate) async fn insert_user(
    pool: &SqlitePool,
    user: &DbAuthUser,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO auth_users (username, player_id, password_hash) VALUES (?, ?, ?)"
    )
    .bind(&user.username)
    .bind(&user.player_id)
    .bind(&user.password_hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// 開発用テストユーザーを確保（存在しなければ作成）
pub(crate) async fn ensure_dev_users(
    pool: &SqlitePool,
    password_hash: &str,
    test_usernames: &[&str],
) -> Result<(), String> {
    for &username in test_usernames {
        if find_user_by_username(pool, username).await.is_some() {
            continue;
        }
        let user = DbAuthUser {
            username: username.to_string(),
            player_id: username.to_string(),
            password_hash: password_hash.to_string(),
        };
        insert_user(pool, &user).await.map_err(|e| e.to_string())?;
        println!("[kingdom-server] テスト用アカウントを作成しました(DB): {username}");
    }
    Ok(())
}
