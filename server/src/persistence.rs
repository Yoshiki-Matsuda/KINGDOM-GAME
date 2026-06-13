use std::path::{Path, PathBuf};

use crate::model::GameState;
use crate::config;
use crate::paths::resolve_project_path;
use crate::server_mode::ServerMode;

/// 旧単一ワールド保存先（リポジトリルート基準の相対パス）
pub(crate) const LEGACY_STATE_REL: &str = "data/state.json";

pub(crate) async fn load_state(path: &Path) -> Option<GameState> {
    let data = tokio::fs::read_to_string(path).await.ok()?;
    let mut state: GameState = serde_json::from_str(&data).ok()?;
    if crate::model::migrate_legacy_terrain(&mut state) {
        let _ = save_state(path, &state).await;
    }
    if crate::items::migrate_inventory_gold_to_resources(&mut state) {
        let _ = save_state(path, &state).await;
    }
    if crate::model::migrate_unillustrated_cards_state(&mut state) {
        let _ = save_state(path, &state).await;
    }
    if crate::model::migrate_legacy_neutral_enemies(&mut state) {
        let _ = save_state(path, &state).await;
    }
    Some(state)
}

pub(crate) async fn save_state(
    path: &Path,
    state: &GameState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let data = serde_json::to_string_pretty(state)?;
    tokio::fs::write(path, data).await?;
    Ok(())
}

fn default_state_rel(mode: ServerMode) -> &'static str {
    match mode {
        ServerMode::Pvp => "data/pvp/state.json",
        ServerMode::Pve => "data/pve/worlds",
    }
}

pub(crate) fn state_path_for_mode(mode: ServerMode) -> PathBuf {
    let rel = config::env_string(config::ENV_STATE_PATH, default_state_rel(mode));
    resolve_project_path(rel)
}

pub(crate) fn auth_path() -> PathBuf {
    let rel = config::auth_path_rel();
    resolve_project_path(rel)
}

pub(crate) fn world_path(base: &Path, player_id: &str) -> PathBuf {
    base.join(player_id).join("state.json")
}

pub(crate) async fn load_player_world(base: &Path, player_id: &str) -> Option<GameState> {
    load_state(&world_path(base, player_id)).await
}

pub(crate) async fn save_player_world(
    base: &Path,
    player_id: &str,
    state: &GameState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    save_state(&world_path(base, player_id), state).await
}

/// 旧 `data/state.json` を `data/pvp/state.json` へ移行
pub(crate) async fn migrate_legacy_pvp_state(target: &Path) {
    let legacy = resolve_project_path(LEGACY_STATE_REL);
    if target.exists() || !legacy.exists() {
        return;
    }
    if let Some(parent) = target.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    match tokio::fs::rename(&legacy, target).await {
        Ok(()) => println!(
            "[kingdom-server] 旧状態を移行しました: {} -> {}",
            legacy.display(),
            target.display()
        ),
        Err(e) => eprintln!("[kingdom-server] 状態移行に失敗: {e}"),
    }
}
