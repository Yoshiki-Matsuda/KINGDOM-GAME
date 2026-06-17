//! データベース永続化レイヤー（旧 JSON ファイル永続化からの移行）

use sqlx::sqlite::SqlitePool;

use crate::model::GameState;
use crate::db::world_repo;

/// DBからゲーム状態を読み込む
pub(crate) async fn load_state(pool: &SqlitePool, world_id: &str) -> Option<GameState> {
    world_repo::load_world(pool, world_id).await
}

/// DBにゲーム状態を保存する
pub(crate) async fn save_state(
    pool: &SqlitePool,
    world_id: &str,
    mode: &str,
    state: &GameState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    world_repo::save_world(pool, world_id, mode, state).await?;
    Ok(())
}

/// PVEワールドをDBから読み込む
pub(crate) async fn load_player_world(pool: &SqlitePool, player_id: &str) -> Option<GameState> {
    let world_id = world_repo::pve_world_id(player_id);
    world_repo::load_world(pool, &world_id).await
}

/// PVEワールドをDBに保存する
pub(crate) async fn save_player_world(
    pool: &SqlitePool,
    player_id: &str,
    state: &GameState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let world_id = world_repo::pve_world_id(player_id);
    world_repo::save_world(pool, &world_id, "pve", state).await?;
    Ok(())
}
