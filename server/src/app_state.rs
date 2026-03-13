use std::{path::PathBuf, sync::Arc};

use tokio::sync::{broadcast, RwLock};

use crate::model::GameState;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) game: Arc<RwLock<GameState>>,
    pub(crate) broadcast_tx: broadcast::Sender<String>,
    pub(crate) state_path: PathBuf,
    /// ローカル開発用: true のとき攻撃側10倍有利
    pub(crate) dev_auto_win: bool,
}
