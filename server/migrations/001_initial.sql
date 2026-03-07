-- キングダム戦略ゲーム 初期スキーマ
-- PvPvE を想定: ゲーム(ワールド)・プレイヤー・領地・状態スナップショット

-- ゲーム（1マッチ = 1ワールド）
CREATE TABLE IF NOT EXISTS games (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  status     TEXT NOT NULL DEFAULT 'active',  -- active | finished
  turn       INTEGER NOT NULL DEFAULT 1,
  phase      TEXT NOT NULL DEFAULT 'idle'
);

-- 領地マスタ（ゲームごとの領地定義。初期配置）
CREATE TABLE IF NOT EXISTS territories (
  id          TEXT NOT NULL,
  game_id     UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
  name        TEXT NOT NULL,
  owner_id    TEXT,           -- プレイヤーID or AI/中立の識別子
  troops      INTEGER NOT NULL DEFAULT 0 CHECK (troops >= 0),
  PRIMARY KEY (game_id, id)
);

CREATE INDEX IF NOT EXISTS idx_territories_game ON territories(game_id);
CREATE INDEX IF NOT EXISTS idx_territories_owner ON territories(game_id, owner_id);

-- プレイヤー参加（PvPvE: 誰がどのゲームに参加しているか）
CREATE TABLE IF NOT EXISTS game_players (
  game_id   UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
  player_id TEXT NOT NULL,
  joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (game_id, player_id)
);

-- 状態スナップショット（ターンごとの永続化用。オプション）
CREATE TABLE IF NOT EXISTS game_state_snapshots (
  id         BIGSERIAL PRIMARY KEY,
  game_id    UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
  turn       INTEGER NOT NULL,
  state_json JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_snapshots_game_turn ON game_state_snapshots(game_id, turn);
