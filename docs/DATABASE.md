# データベース（PostgreSQL）

## スキーマ

- **場所**: `server/migrations/001_initial.sql`
- **用途**: ゲーム(ワールド)・領地・プレイヤー参加・状態スナップショット（PvPvE 想定）

### テーブル概要

| テーブル | 役割 |
|----------|------|
| `games` | 1マッチ = 1行。turn / phase / status。 |
| `territories` | ゲームごとの領地。game_id + id で一意。owner_id, troops。 |
| `game_players` | どのプレイヤーがどのゲームに参加しているか。 |
| `game_state_snapshots` | ターンごとの状態 JSON（リプレイ・ロールバック用のオプション）。 |

## ローカルでマイグレーションする

PostgreSQL が起動している前提で:

```bash
cd server
psql -U postgres -d kingdom -f migrations/001_initial.sql
```

DB を新規作成する場合:

```bash
createdb -U postgres kingdom
psql -U postgres -d kingdom -f server/migrations/001_initial.sql
```

## 次のステップ

- Rust 側で `sqlx` または `diesel` を導入し、起動時に `games` から状態を読み込む／行動適用後に `territories` と `game_state_snapshots` を更新する。
- 現状はメモリのみで動作。永続化は接続後に実装可能。
