use std::path::{Path, PathBuf};

use crate::model::GameState;

/// 状態ファイル（サーバー起動時の CWD 基準）
pub(crate) const STATE_FILE: &str = "data/state.json";

pub(crate) async fn load_state(path: &Path) -> Option<GameState> {
    let data = tokio::fs::read_to_string(path).await.ok()?;
    serde_json::from_str(&data).ok()
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

pub(crate) fn state_path() -> PathBuf {
    PathBuf::from(STATE_FILE)
}
