use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use sqlx::sqlite::SqlitePool;
use tokio::sync::{broadcast, RwLock};

use crate::model::{GameState, MarchKind, WorldConfig};
use crate::config;
use crate::db::world_repo;
use crate::pve_world::new_pve;

struct WorldEntry {
    state: Arc<RwLock<GameState>>,
    /// 最後のプレイヤー操作またはバックグラウンド到着処理の時刻
    last_access: Instant,
    broadcast_tx: broadcast::Sender<String>,
}

#[derive(Clone)]
pub(crate) struct WorldManager {
    inner: Arc<RwLock<HashMap<String, WorldEntry>>>,
    pool: SqlitePool,
    world_config: WorldConfig,
}

impl WorldManager {
    pub(crate) fn new(pool: SqlitePool, world_config: WorldConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            pool,
            world_config,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub(crate) async fn get_or_create_world(&self, player_id: &str) -> Arc<RwLock<GameState>> {
        {
            let worlds = self.inner.read().await;
            if let Some(entry) = worlds.get(player_id) {
                return entry.state.clone();
            }
        }

        let mut worlds = self.inner.write().await;
        if let Some(entry) = worlds.get(player_id) {
            return entry.state.clone();
        }

        let state = if let Some(loaded) = crate::persistence::load_player_world(&self.pool, player_id).await {
            Arc::new(RwLock::new(loaded))
        } else {
            let fresh = new_pve(player_id, self.world_config);
            let arc = Arc::new(RwLock::new(fresh));
            if let Ok(guard) = arc.try_read() {
                let _ = crate::persistence::save_player_world(&self.pool, player_id, &guard).await;
            }
            arc
        };

        let (broadcast_tx, _) = broadcast::channel(64);
        worlds.insert(
            player_id.to_string(),
            WorldEntry {
                state: state.clone(),
                last_access: Instant::now(),
                broadcast_tx,
            },
        );
        state
    }

    pub(crate) async fn touch(&self, player_id: &str) {
        let mut worlds = self.inner.write().await;
        if let Some(entry) = worlds.get_mut(player_id) {
            entry.last_access = Instant::now();
        }
    }

    pub(crate) async fn list_saved_player_ids(&self) -> Vec<String> {
        world_repo::list_pve_world_ids(&self.pool).await
    }

    /// メモリ未ロードの保存ワールドに、バックグラウンド処理対象の到着済み遠征があるか（帰還除く）
    pub(crate) async fn saved_world_has_due_outbound_marches(&self, player_id: &str) -> bool {
        let Some(state) = crate::persistence::load_player_world(&self.pool, player_id).await else {
            return false;
        };
        let now = crate::model::default_now_ms();
        state.players.values().flat_map(|p| &p.marches).any(|m| {
            m.arrives_at <= now && m.kind != MarchKind::Return
        })
    }

    /// 遠征到着スケジューラの対象（メモリ上の全ワールド + ディスク上の到着済み攻撃等）
    pub(crate) async fn player_ids_for_march_processing(&self) -> Vec<String> {
        use std::collections::HashSet;
        let mut ids: HashSet<String> = self.active_player_ids().await.into_iter().collect();
        for player_id in self.list_saved_player_ids().await {
            if ids.contains(&player_id) {
                continue;
            }
            if self.saved_world_has_due_outbound_marches(&player_id).await {
                ids.insert(player_id);
            }
        }
        ids.into_iter().collect()
    }

    pub(crate) async fn save_world(&self, player_id: &str, state: &GameState) {
        let _ = crate::persistence::save_player_world(&self.pool, player_id, state).await;
    }

    pub(crate) async fn broadcast(&self, player_id: &str, json: String) {
        let worlds = self.inner.read().await;
        if let Some(entry) = worlds.get(player_id) {
            let _ = entry.broadcast_tx.send(json);
        }
    }

    pub(crate) async fn subscribe(&self, player_id: &str) -> Option<broadcast::Receiver<String>> {
        let worlds = self.inner.read().await;
        worlds.get(player_id).map(|e| e.broadcast_tx.subscribe())
    }

    pub(crate) async fn active_player_ids(&self) -> Vec<String> {
        let worlds = self.inner.read().await;
        worlds.keys().cloned().collect()
    }

    /// ブラウザ切断とは無関係。最終操作（または到着処理）から `EVICT_IDLE_MINUTES` 経過で解放。
    pub(crate) async fn evict_idle(&self) {
        let idle_minutes = config::evict_idle_minutes();
        let threshold = Duration::from_secs(idle_minutes * 60);
        let now = Instant::now();

        let mut worlds = self.inner.write().await;
        let stale: Vec<String> = worlds
            .iter()
            .filter(|(_, e)| now.duration_since(e.last_access) > threshold)
            .map(|(id, _)| id.clone())
            .collect();
        for id in stale {
            worlds.remove(&id);
            println!("[kingdom-server] PVEワールドをメモリから解放: {id}");
        }
    }
}

pub(crate) fn spawn_world_eviction(manager: Arc<WorldManager>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(crate::config::DEFAULT_WORLD_EVICT_POLL_INTERVAL_SEC)).await;
            manager.evict_idle().await;
        }
    });
}
