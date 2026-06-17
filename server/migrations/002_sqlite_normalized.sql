-- キングダム戦略ゲーム SQLite正規化スキーマ
-- state.json / auth.json → SQLite 移行用
-- 実行前に PRAGMA foreign_keys = ON; を推奨

-- ワールド（PVP共有 or PVEプレイヤー別）
CREATE TABLE IF NOT EXISTS worlds (
  id              TEXT PRIMARY KEY,
  mode            TEXT NOT NULL,
  world_owner_id  TEXT,
  cols            INTEGER NOT NULL DEFAULT 48,
  rows            INTEGER NOT NULL DEFAULT 48,
  home_col        INTEGER NOT NULL DEFAULT 24,
  home_row        INTEGER NOT NULL DEFAULT 24,
  terrain_seed    INTEGER NOT NULL DEFAULT 0,
  season_number   INTEGER NOT NULL DEFAULT 1,
  season_started_at   INTEGER NOT NULL DEFAULT 0,
  season_duration_ms  INTEGER NOT NULL DEFAULT 7776000000
);

-- AI勢力（PVE専用）
CREATE TABLE IF NOT EXISTS world_ai_factions (
  world_id          TEXT NOT NULL,
  faction_id        TEXT NOT NULL,
  name              TEXT NOT NULL,
  personality       TEXT NOT NULL,
  home_territory_id TEXT NOT NULL,
  color             INTEGER NOT NULL,
  PRIMARY KEY (world_id, faction_id),
  FOREIGN KEY (world_id) REFERENCES worlds(id) ON DELETE CASCADE
);

-- 領地
CREATE TABLE IF NOT EXISTS territories (
  world_id          TEXT NOT NULL,
  id                TEXT NOT NULL,
  name              TEXT NOT NULL,
  level             INTEGER NOT NULL DEFAULT 1,
  owner_id          TEXT,
  troops            INTEGER NOT NULL DEFAULT 0,
  body_monster_counts TEXT,
  body_names        TEXT,
  ruin              TEXT,
  is_base           INTEGER NOT NULL DEFAULT 0,
  durability        INTEGER NOT NULL DEFAULT 0,
  max_durability    INTEGER NOT NULL DEFAULT 0,
  tower_level       INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (world_id, id),
  FOREIGN KEY (world_id) REFERENCES worlds(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_territories_owner ON territories(world_id, owner_id);

-- プレイヤー
CREATE TABLE IF NOT EXISTS players (
  world_id          TEXT NOT NULL,
  player_id         TEXT NOT NULL,
  home_territory_id TEXT NOT NULL,
  resource_food     INTEGER NOT NULL DEFAULT 500,
  resource_wood     INTEGER NOT NULL DEFAULT 500,
  resource_stone    INTEGER NOT NULL DEFAULT 500,
  resource_iron     INTEGER NOT NULL DEFAULT 500,
  resource_gold     INTEGER NOT NULL DEFAULT 1000,
  last_resource_tick INTEGER NOT NULL DEFAULT 0,
  last_stamina_tick INTEGER NOT NULL DEFAULT 0,
  exploration_level INTEGER NOT NULL DEFAULT 1,
  exploration_score INTEGER NOT NULL DEFAULT 0,
  unit_cost_cap     REAL NOT NULL DEFAULT 4.0,
  dungeon_points    INTEGER NOT NULL DEFAULT 0,
  charge_points     INTEGER NOT NULL DEFAULT 0,
  ai_recover_until  INTEGER NOT NULL DEFAULT 0,
  ai_last_attack_target TEXT,
  PRIMARY KEY (world_id, player_id),
  FOREIGN KEY (world_id) REFERENCES worlds(id) ON DELETE CASCADE
);

-- プレイヤー所持魔獣
CREATE TABLE IF NOT EXISTS player_cards (
  world_id          TEXT NOT NULL,
  player_id         TEXT NOT NULL,
  slot_index        INTEGER NOT NULL,
  card_id           INTEGER NOT NULL,
  monster_count     INTEGER NOT NULL DEFAULT 1,
  level             INTEGER NOT NULL DEFAULT 1,
  exp               INTEGER NOT NULL DEFAULT 0,
  stamina           INTEGER NOT NULL DEFAULT 100,
  status_points     INTEGER NOT NULL DEFAULT 0,
  bonus_speed       INTEGER NOT NULL DEFAULT 0,
  bonus_attack      INTEGER NOT NULL DEFAULT 0,
  bonus_intelligence INTEGER NOT NULL DEFAULT 0,
  bonus_defense     INTEGER NOT NULL DEFAULT 0,
  bonus_magic_defense INTEGER NOT NULL DEFAULT 0,
  rest_until        INTEGER NOT NULL DEFAULT 0,
  awakened          INTEGER NOT NULL DEFAULT 0,
  enhanced          INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (world_id, player_id, slot_index),
  FOREIGN KEY (world_id, player_id) REFERENCES players(world_id, player_id) ON DELETE CASCADE
);

-- プレイヤー魔獣スキルレベル
CREATE TABLE IF NOT EXISTS player_card_skill_levels (
  world_id          TEXT NOT NULL,
  player_id         TEXT NOT NULL,
  slot_index        INTEGER NOT NULL,
  skill_0           INTEGER NOT NULL DEFAULT 0,
  skill_1           INTEGER NOT NULL DEFAULT 0,
  skill_2           INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (world_id, player_id, slot_index),
  FOREIGN KEY (world_id, player_id) REFERENCES players(world_id, player_id) ON DELETE CASCADE
);

-- プレイヤーインベントリ
CREATE TABLE IF NOT EXISTS player_inventory (
  world_id          TEXT NOT NULL,
  player_id         TEXT NOT NULL,
  item_id           TEXT NOT NULL,
  count             INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (world_id, player_id, item_id),
  FOREIGN KEY (world_id, player_id) REFERENCES players(world_id, player_id) ON DELETE CASCADE
);

-- プレイヤー施設
CREATE TABLE IF NOT EXISTS player_facilities (
  world_id          TEXT NOT NULL,
  player_id         TEXT NOT NULL,
  idx               INTEGER NOT NULL,
  facility_id       TEXT NOT NULL,
  level             INTEGER NOT NULL DEFAULT 1,
  build_complete_at INTEGER,
  pos_col           INTEGER,
  pos_row           INTEGER,
  PRIMARY KEY (world_id, player_id, idx),
  FOREIGN KEY (world_id, player_id) REFERENCES players(world_id, player_id) ON DELETE CASCADE
);

-- プレイヤー同盟関係（援軍送り先）
CREATE TABLE IF NOT EXISTS player_allied_ids (
  world_id          TEXT NOT NULL,
  player_id         TEXT NOT NULL,
  allied_id         TEXT NOT NULL,
  PRIMARY KEY (world_id, player_id, allied_id),
  FOREIGN KEY (world_id, player_id) REFERENCES players(world_id, player_id) ON DELETE CASCADE
);

-- プレイヤーAI攻撃クールダウン
CREATE TABLE IF NOT EXISTS player_ai_cooldowns (
  world_id          TEXT NOT NULL,
  player_id         TEXT NOT NULL,
  territory_id      TEXT NOT NULL,
  expire_at         INTEGER NOT NULL,
  PRIMARY KEY (world_id, player_id, territory_id),
  FOREIGN KEY (world_id, player_id) REFERENCES players(world_id, player_id) ON DELETE CASCADE
);

-- プレイヤー遠征
CREATE TABLE IF NOT EXISTS player_marches (
  world_id          TEXT NOT NULL,
  player_id         TEXT NOT NULL,
  march_id          TEXT NOT NULL,
  kind              TEXT NOT NULL,
  from_territory_id TEXT NOT NULL,
  to_territory_id   TEXT NOT NULL,
  started_at        INTEGER NOT NULL,
  arrives_at        INTEGER NOT NULL,
  count             INTEGER NOT NULL DEFAULT 1,
  unit_name         TEXT,
  owned_card_indices TEXT,
  formed_unit_id    TEXT,
  PRIMARY KEY (world_id, player_id, march_id),
  FOREIGN KEY (world_id, player_id) REFERENCES players(world_id, player_id) ON DELETE CASCADE
);

-- 遠征のbody詳細
CREATE TABLE IF NOT EXISTS player_march_bodies (
  world_id          TEXT NOT NULL,
  player_id         TEXT NOT NULL,
  march_id          TEXT NOT NULL,
  body_index        INTEGER NOT NULL,
  monster_count     INTEGER,
  body_name         TEXT,
  speed             INTEGER,
  skills            TEXT,
  stats             TEXT,
  PRIMARY KEY (world_id, player_id, march_id, body_index),
  FOREIGN KEY (world_id, player_id, march_id) REFERENCES player_marches(world_id, player_id, march_id) ON DELETE CASCADE
);

-- プレイヤーユニット編成
CREATE TABLE IF NOT EXISTS player_formed_units (
  world_id          TEXT NOT NULL,
  player_id         TEXT NOT NULL,
  unit_id           TEXT NOT NULL,
  unit_name         TEXT NOT NULL,
  slot_0            INTEGER NOT NULL DEFAULT -1,
  slot_1            INTEGER NOT NULL DEFAULT -1,
  slot_2            INTEGER NOT NULL DEFAULT -1,
  PRIMARY KEY (world_id, player_id, unit_id),
  FOREIGN KEY (world_id, player_id) REFERENCES players(world_id, player_id) ON DELETE CASCADE
);

-- 同盟
CREATE TABLE IF NOT EXISTS alliances (
  world_id          TEXT NOT NULL,
  alliance_id       TEXT NOT NULL,
  name              TEXT NOT NULL,
  leader_id         TEXT NOT NULL,
  territory_points  INTEGER NOT NULL DEFAULT 0,
  level             INTEGER NOT NULL DEFAULT 1,
  donated_total     INTEGER NOT NULL DEFAULT 0,
  parent_alliance_id TEXT,
  PRIMARY KEY (world_id, alliance_id),
  FOREIGN KEY (world_id) REFERENCES worlds(id) ON DELETE CASCADE
);

-- 同盟メンバー
CREATE TABLE IF NOT EXISTS alliance_members (
  world_id          TEXT NOT NULL,
  alliance_id       TEXT NOT NULL,
  player_id         TEXT NOT NULL,
  PRIMARY KEY (world_id, alliance_id, player_id),
  FOREIGN KEY (world_id, alliance_id) REFERENCES alliances(world_id, alliance_id) ON DELETE CASCADE
);

-- 同盟の子同盟ID
CREATE TABLE IF NOT EXISTS alliance_children (
  world_id          TEXT NOT NULL,
  alliance_id       TEXT NOT NULL,
  child_alliance_id TEXT NOT NULL,
  PRIMARY KEY (world_id, alliance_id, child_alliance_id),
  FOREIGN KEY (world_id, alliance_id) REFERENCES alliances(world_id, alliance_id) ON DELETE CASCADE
);

-- フリーマーケット出品
CREATE TABLE IF NOT EXISTS market_listings (
  world_id          TEXT NOT NULL,
  listing_id        TEXT NOT NULL,
  seller_id         TEXT NOT NULL,
  item_type         TEXT NOT NULL,
  item_card_id      INTEGER,
  item_item_id      TEXT,
  item_count        INTEGER,
  item_resource_type TEXT,
  item_amount       INTEGER,
  price             INTEGER NOT NULL,
  listed_at         INTEGER NOT NULL,
  PRIMARY KEY (world_id, listing_id),
  FOREIGN KEY (world_id) REFERENCES worlds(id) ON DELETE CASCADE
);

-- ゲームログ
CREATE TABLE IF NOT EXISTS game_logs (
  world_id          TEXT NOT NULL,
  idx               INTEGER NOT NULL,
  message           TEXT NOT NULL,
  PRIMARY KEY (world_id, idx),
  FOREIGN KEY (world_id) REFERENCES worlds(id) ON DELETE CASCADE
);

-- 認証ユーザー
CREATE TABLE IF NOT EXISTS auth_users (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  username        TEXT NOT NULL UNIQUE,
  player_id       TEXT NOT NULL,
  password_hash   TEXT NOT NULL
);
