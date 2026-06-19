-- ログのフル構造化: game_logs -> event_logs
-- ワイプ前提。旧テーブル削除 + 新テーブル作成

DROP TABLE IF EXISTS game_logs;

CREATE TABLE IF NOT EXISTS event_logs (
  world_id   TEXT    NOT NULL,
  id         INTEGER NOT NULL,
  timestamp  INTEGER NOT NULL,
  actor_id   TEXT,
  event_type TEXT    NOT NULL,
  data       TEXT    NOT NULL DEFAULT '{}',
  message    TEXT    NOT NULL DEFAULT '',
  PRIMARY KEY (world_id, id),
  FOREIGN KEY (world_id) REFERENCES worlds(id) ON DELETE CASCADE
);
