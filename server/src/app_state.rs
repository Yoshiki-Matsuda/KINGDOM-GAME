use std::sync::Arc;

use sqlx::sqlite::SqlitePool;
use tokio::sync::{broadcast, Mutex, Notify, RwLock};

use crate::model::{GameState, WorldConfig};
use crate::server_mode::ServerMode;
use crate::world_manager::WorldManager;

#[derive(Clone)]
pub(crate) enum GameStore {
    Shared(Arc<RwLock<GameState>>),
    PerPlayer(Arc<WorldManager>),
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) server_mode: ServerMode,
    pub(crate) store: GameStore,
    pub(crate) mutation_lock: Arc<Mutex<()>>,
    /// PVPモード用グローバルブロードキャスト
    pub(crate) broadcast_tx: broadcast::Sender<String>,
    pub(crate) db_pool: SqlitePool,
    pub(crate) jwt_secret: Arc<Vec<u8>>,
    pub(crate) dev_auto_win: bool,
    pub(crate) world_config: WorldConfig,
    /// 遠征到着スケジューラを再起動（新規出発・到着予定変更時）
    pub(crate) march_wake: Arc<Notify>,
}

impl AppState {
    pub(crate) fn wake_march_scheduler(&self) {
        self.march_wake.notify_waiters();
    }

    pub(crate) fn wake_march_scheduler_if_active(&self, game: &GameState) {
        if game.players.values().any(|p| !p.marches.is_empty()) {
            self.wake_march_scheduler();
        }
    }
    pub(crate) fn broadcast_json(&self, player_id: Option<&str>, json: String) {
        match &self.store {
            GameStore::Shared(_) => {
                let _ = self.broadcast_tx.send(json);
            }
            GameStore::PerPlayer(mgr) => {
                if let Some(pid) = player_id {
                    let mgr = mgr.clone();
                    let json = json.clone();
                    let pid = pid.to_string();
                    tokio::spawn(async move {
                        mgr.broadcast(&pid, json).await;
                    });
                }
            }
        }
    }

    /// PVP用ワールドID
    pub(crate) fn pvp_world_id(&self) -> &'static str {
        crate::db::world_repo::pvp_world_id()
    }
}
