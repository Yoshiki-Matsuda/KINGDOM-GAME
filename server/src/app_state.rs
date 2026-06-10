use std::{path::PathBuf, sync::Arc};

use tokio::sync::{broadcast, Mutex, RwLock};

use crate::model::GameState;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) game: Arc<RwLock<GameState>>,
    pub(crate) mutation_lock: Arc<Mutex<()>>,
    pub(crate) broadcast_tx: broadcast::Sender<String>,
    pub(crate) state_path: PathBuf,
    pub(crate) auth_path: PathBuf,
    pub(crate) jwt_secret: Arc<Vec<u8>>,
    /// ローカル開発用: true のとき攻撃側10倍有利
    pub(crate) dev_auto_win: bool,
}
