//! サーバー環境変数と既定値の一元管理

use crate::server_mode::ServerMode;

// --- 環境変数名 ---

pub const ENV_SERVER_MODE: &str = "SERVER_MODE";
pub const ENV_PORT: &str = "PORT";
pub const ENV_DEV_AUTO_WIN: &str = "DEV_AUTO_WIN";
pub const ENV_DEV_BOT: &str = "DEV_BOT";
pub const ENV_AUTH_JWT_SECRET: &str = "AUTH_JWT_SECRET";
pub const ENV_STAMINA_RECOVERY_PER_MIN: &str = "STAMINA_RECOVERY_PER_MIN";
pub const ENV_WORLD_TICK_SEC: &str = "WORLD_TICK_SEC";
pub const ENV_FACILITY_RESOURCE_TICK_SEC: &str = "FACILITY_RESOURCE_TICK_SEC";
pub const ENV_MAX_CARD_STAMINA: &str = "MAX_CARD_STAMINA";
pub const ENV_STAMINA_ATTACK: &str = "STAMINA_ATTACK";
pub const ENV_STAMINA_EXPLORATION: &str = "STAMINA_EXPLORATION";
pub const ENV_AI_TICK_INTERVAL_SEC: &str = "AI_TICK_INTERVAL_SEC";
pub const ENV_AI_FACTION_MAX: &str = "AI_FACTION_MAX";
pub const ENV_EVICT_IDLE_MINUTES: &str = "EVICT_IDLE_MINUTES";
pub const ENV_WORLD_COLS: &str = "WORLD_COLS";
pub const ENV_WORLD_ROWS: &str = "WORLD_ROWS";
pub const ENV_WORLD_HOME_COL: &str = "WORLD_HOME_COL";
pub const ENV_WORLD_HOME_ROW: &str = "WORLD_HOME_ROW";
pub const ENV_WORLD_TERRAIN_SEED: &str = "WORLD_TERRAIN_SEED";
pub const ENV_DATABASE_URL: &str = "DATABASE_URL";
pub const DEFAULT_DATABASE_URL: &str = "sqlite://kingdom.db?mode=rwc";
pub const ENV_ADMIN_PLAYER_ID: &str = "ADMIN_PLAYER_ID";
pub const ENV_DEV_AUTH_PASSWORD: &str = "DEV_AUTH_PASSWORD";
pub const ENV_DEV_BOT_USERNAME: &str = "DEV_BOT_USERNAME";
pub const ENV_DEV_BOT_PASSWORD: &str = "DEV_BOT_PASSWORD";
pub const ENV_DEV_BOT_TARGET: &str = "DEV_BOT_TARGET";
pub const ENV_DEV_BOT_INTERVAL_SEC: &str = "DEV_BOT_INTERVAL_SEC";
pub const ENV_DEV_BOT_HTTP_ORIGIN: &str = "DEV_BOT_HTTP_ORIGIN";
pub const ENV_DEV_BOT_WS_URL: &str = "DEV_BOT_WS_URL";
pub const ENV_PROJECT_ROOT: &str = "PROJECT_ROOT";

// --- 既定値 ---

pub const DEFAULT_SERVER_MODE: &str = "pvp";
pub const DEFAULT_PORT_PVP: u16 = 3000;
pub const DEFAULT_PORT_PVE: u16 = 3001;
pub const DEFAULT_STAMINA_RECOVERY_PER_MIN: u32 = 5;
pub const DEFAULT_WORLD_TICK_SEC: u64 = 60;
pub const DEFAULT_MARCH_IDLE_POLL_MS: u64 = 5_000;
pub const ENV_MARCH_IDLE_POLL_MS: &str = "MARCH_IDLE_POLL_MS";
pub const DEFAULT_FACILITY_RESOURCE_TICK_SEC: u64 = 600;
pub const DEFAULT_MAX_CARD_STAMINA: u32 = 100;
pub const DEFAULT_STAMINA_ATTACK: u32 = 50;
pub const DEFAULT_STAMINA_EXPLORATION: u32 = 5;
pub const DEFAULT_AI_TICK_INTERVAL_SEC: u64 = 10;
pub const DEFAULT_EVICT_IDLE_MINUTES: u64 = 30;
pub const DEFAULT_WORLD_EVICT_POLL_INTERVAL_SEC: u64 = 300;
pub const DEFAULT_WORLD_COLS: u16 = 48;
pub const DEFAULT_WORLD_ROWS: u16 = 48;
pub const DEFAULT_HOME_COL: u8 = 24;
pub const DEFAULT_HOME_ROW: u8 = 24;
pub const DEFAULT_JWT_SECRET: &str = "dev-only-change-this-secret";
pub const DEFAULT_ADMIN_PLAYER_ID: &str = "admin";
/// フロントの `VITE_DEV_PASSWORD` 既定値と揃える
pub const DEFAULT_DEV_AUTH_PASSWORD: &str = "test12345";
pub const DEFAULT_DEV_BOT_USERNAME: &str = "player";
pub const DEFAULT_DEV_BOT_TARGET: &str = "offline_test";
pub const DEFAULT_DEV_BOT_INTERVAL_SEC: u64 = 45;
pub const MIN_DEV_BOT_INTERVAL_SEC: u64 = 10;

pub(crate) fn default_world_area() -> u32 {
    DEFAULT_WORLD_COLS as u32 * DEFAULT_WORLD_ROWS as u32
}

pub(crate) fn default_port_for_mode(mode: ServerMode) -> u16 {
    match mode {
        ServerMode::Pvp => DEFAULT_PORT_PVP,
        ServerMode::Pve => DEFAULT_PORT_PVE,
    }
}

pub(crate) fn env_string(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

pub(crate) fn env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(v) => matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"),
        Err(_) => default,
    }
}

pub(crate) fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

pub(crate) fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

pub(crate) fn env_u16(key: &str, default: u16) -> u16 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

pub(crate) fn env_optional_u32(key: &str) -> Option<u32> {
    std::env::var(key).ok().and_then(|s| s.parse().ok())
}

pub(crate) fn optional_terrain_seed_from_env() -> Option<u64> {
    std::env::var(ENV_WORLD_TERRAIN_SEED)
        .ok()
        .and_then(|s| s.parse().ok())
}

pub(crate) fn dev_auto_win_enabled() -> bool {
    env_bool(ENV_DEV_AUTO_WIN, false)
}

pub(crate) fn stamina_recovery_per_min() -> u32 {
    env_u32(
        ENV_STAMINA_RECOVERY_PER_MIN,
        DEFAULT_STAMINA_RECOVERY_PER_MIN,
    )
}

pub(crate) fn world_tick_secs() -> u64 {
    env_u64(ENV_WORLD_TICK_SEC, DEFAULT_WORLD_TICK_SEC)
}

pub(crate) fn march_idle_poll_ms() -> u64 {
    env_u64(ENV_MARCH_IDLE_POLL_MS, DEFAULT_MARCH_IDLE_POLL_MS)
}

pub(crate) fn facility_resource_tick_ms() -> u64 {
    env_u64(ENV_FACILITY_RESOURCE_TICK_SEC, DEFAULT_FACILITY_RESOURCE_TICK_SEC) * 1000
}

pub(crate) fn max_card_stamina() -> u32 {
    env_u32(ENV_MAX_CARD_STAMINA, DEFAULT_MAX_CARD_STAMINA)
}

pub(crate) fn stamina_attack_cost() -> u32 {
    env_u32(ENV_STAMINA_ATTACK, DEFAULT_STAMINA_ATTACK)
}

pub(crate) fn stamina_exploration_cost() -> u32 {
    env_u32(ENV_STAMINA_EXPLORATION, DEFAULT_STAMINA_EXPLORATION)
}

pub(crate) fn ai_tick_interval_secs() -> u64 {
    env_u64(ENV_AI_TICK_INTERVAL_SEC, DEFAULT_AI_TICK_INTERVAL_SEC)
}

pub(crate) fn evict_idle_minutes() -> u64 {
    env_u64(ENV_EVICT_IDLE_MINUTES, DEFAULT_EVICT_IDLE_MINUTES)
}

pub(crate) fn ai_faction_max_cap() -> Option<u32> {
    env_optional_u32(ENV_AI_FACTION_MAX)
}

pub(crate) fn listen_port(mode: ServerMode) -> u16 {
    env_u16(ENV_PORT, default_port_for_mode(mode))
}

pub(crate) fn jwt_secret_bytes() -> Vec<u8> {
    env_string(ENV_AUTH_JWT_SECRET, DEFAULT_JWT_SECRET).into_bytes()
}

pub(crate) fn admin_player_id() -> String {
    env_string(ENV_ADMIN_PLAYER_ID, DEFAULT_ADMIN_PLAYER_ID)
}

pub(crate) fn database_url() -> String {
    env_string(ENV_DATABASE_URL, DEFAULT_DATABASE_URL)
}

pub(crate) fn dev_auth_password() -> String {
    env_string(ENV_DEV_AUTH_PASSWORD, DEFAULT_DEV_AUTH_PASSWORD)
}

pub(crate) fn dev_bot_enabled() -> bool {
    env_bool(ENV_DEV_BOT, false) || dev_auto_win_enabled()
}

pub(crate) fn dev_bot_interval_secs() -> u64 {
    env_u64(ENV_DEV_BOT_INTERVAL_SEC, DEFAULT_DEV_BOT_INTERVAL_SEC)
        .max(MIN_DEV_BOT_INTERVAL_SEC)
}
