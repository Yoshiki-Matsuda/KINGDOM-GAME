# kingdom-server（Rust バックエンド）

キングダム戦略ゲームのサーバー権威バックエンドです。**同一バイナリ**を環境変数 `SERVER_MODE` で切り替え、PVP（共有ワールド）と PVE（プレイヤー別ソロワールド）を**別プロセス・別ポート**で起動します。

## アーキテクチャ概要

```
kingdom-game/
  data/
    auth.json              # 認証（PVP/PVE 共有）
    pvp/
      state.json           # PVP 共有ワールド
    pve/
      worlds/
        {player_id}/
          state.json       # PVE プレイヤー専用ワールド
  server/                  # このクレート
  src/                     # フロントエンド（Vite）
```

| モード | `SERVER_MODE` | 既定ポート | ワールド | ブロードキャスト |
|--------|---------------|-----------|---------|----------------|
| PVP | `pvp`（既定） | 3000 | 全プレイヤー共有1マップ | 全接続者 |
| PVE | `pve` | 3001 | プレイヤーごとに独立マップ | 接続プレイヤーのみ |

**重要:** 1回の `cargo run` で起動するのは **1プロセス（PVP か PVE のどちらか）** だけです。両方使う場合は **ターミナルを2つ** 開いて、それぞれ別の環境変数で起動してください。

---

## 前提

- [Rust](https://www.rust-lang.org/tools/install) がインストールされ、`cargo` が PATH に通っていること
- **カレントディレクトリ（CWD）はどこでも可**（リポジトリルートでも `server/` でも同じ `data/` を使う）
- 永続化の相対パス（`data/auth.json` など）は **リポジトリルート** 基準で解決される（`server/` クレートの親ディレクトリ）

---

## ビルド

```bash
# リポジトリルートから
cargo build --manifest-path server/Cargo.toml

# server/ ディレクトリから
cargo build
```

```bash
# テスト（どちらの CWD でも可）
cargo test --manifest-path server/Cargo.toml
# または server/ 内で: cargo test
```

---

## 起動

CWD が **リポジトリルート** でも **`server/`** でも、同じ `data/` を参照します。好きな方で実行してください。

### PVP のみ（最もシンプル）

`SERVER_MODE` を省略すると PVP としてポート **3000** で起動します。

**リポジトリルートから**

```powershell
cargo run --manifest-path server/Cargo.toml
```

**`server/` ディレクトリから**

```powershell
cargo run
```

### PVP と PVE を同時に動かす

**ターミナル1 — PVP（ポート 3000）**

```powershell
# ルートから
$env:SERVER_MODE="pvp"; $env:PORT="3000"; cargo run --manifest-path server/Cargo.toml

# server/ から
$env:SERVER_MODE="pvp"; $env:PORT="3000"; cargo run
```

**ターミナル2 — PVE（ポート 3001）**

```powershell
# ルートから
$env:SERVER_MODE="pve"; $env:PORT="3001"; cargo run --manifest-path server/Cargo.toml
$env:SERVER_MODE="pve"; $env:PORT="3001"; $env:DEV_AUTO_WIN=1; cargo run --manifest-path server/Cargo.toml

# server/ から
$env:SERVER_MODE="pve"; $env:PORT="3001"; cargo run
```

起動ログに `project_root=...` と `mode=pvp` / `mode=pve` が出れば、正しいデータディレクトリを掴めています。

### 起動確認

```powershell
# PVP
Invoke-RestMethod http://127.0.0.1:3000/health
Invoke-RestMethod http://127.0.0.1:3000/api

# PVE
Invoke-RestMethod http://127.0.0.1:3001/health
Invoke-RestMethod http://127.0.0.1:3001/api
```

`/health` の例:

```json
{
  "status": "ok",
  "service": "kingdom-server",
  "version": "0.1.0",
  "mode": "pvp",
  "world_cols": 48,
  "world_rows": 48
}
```

`/api` の `mode` が `pvp` / `pve` になっていることを確認してください。

---

## 環境変数一覧

環境変数名と既定値は `server/src/config.rs` に集約しています。コードとドキュメントを変更するときはそちらを正とします。

### サーバーモード・ネットワーク

| 変数 | 既定値 | 説明 |
|------|--------|------|
| `PROJECT_ROOT` | `server/` の親ディレクトリ | 相対パス解決の基準（通常は未設定でよい） |
| `SERVER_MODE` | `pvp` | `pvp` または `pve` |
| `PORT` | PVP: `3000` / PVE: `3001` | 待ち受けポート |
| `STATE_PATH` | `data/pvp/state.json` / `data/pve/worlds/` | ゲーム状態（**リポジトリルート基準**の相対パス） |
| `AUTH_PATH` | `data/auth.json` | 認証ファイル（同上） |

### マップサイズ

| 変数 | 既定値 | 説明 |
|------|--------|------|
| `WORLD_COLS` | `48` | マップ横マス数 |
| `WORLD_ROWS` | `48` | マップ縦マス数 |
| `WORLD_HOME_COL` | `cols / 2` | プレイヤー本拠の列 |
| `WORLD_HOME_ROW` | `rows / 2` | プレイヤー本拠の行 |

開発用は 48×48。本番想定は 240〜480 など `WORLD_COLS` / `WORLD_ROWS` で拡張します。

### ゲームバランス

| 変数 | 既定値 | 説明 |
|------|--------|------|
| `STAMINA_RECOVERY_PER_MIN` | `5` | 魔獣スタミナの時間回復（1分あたり・スロット単位） |
| `MAX_CARD_STAMINA` | `100` | 魔獣スタミナ上限 |
| `STAMINA_ATTACK` | `50` | 攻撃遠征の出発時スタミナ消費 |
| `STAMINA_EXPLORATION` | `5` | 探索遠征の出発時スタミナ消費 |
| `WORLD_TICK_SEC` | `60` | バックグラウンド `tick_world` の間隔（秒） |
| `MARCH_IDLE_POLL_MS` | `5000` | 進行中遠征がないときの到着スケジューラ待機（ミリ秒） |
| `FACILITY_RESOURCE_TICK_SEC` | `600` | 施設資源生産の加算間隔（秒・10分） |
| `AI_FACTION_MAX` | 上限なし | PVE の AI 勢力数上限（面積比例算出のキャップ） |
| `AI_TICK_INTERVAL_SEC` | `10` | PVE AI ターン間隔（秒） |
| `EVICT_IDLE_MINUTES` | `30` | PVE 非アクティブワールドのメモリ解放までの時間 |

### 認証・管理

| 変数 | 既定値 | 説明 |
|------|--------|------|
| `AUTH_JWT_SECRET` | 開発用固定文字列 | JWT 署名鍵（本番では必ず変更） |
| `DEV_AUTH_PASSWORD` | `test12345` | テストアカウント初回作成時のパスワード |
| `ADMIN_PLAYER_ID` | `admin` | `/admin/wipe` を実行できるプレイヤー ID |

### ローカル開発用（PVP のみ）

| 変数 | 説明 |
|------|------|
| `DEV_AUTO_WIN=1` | 攻撃側を 10 倍有利に + 人間プレイヤーの所持魔獣スタミナ無限（戦闘ログは通常表示） |
| `DEV_BOT=1` | `player` アカウントが WS 経由で自動攻撃（`offline_test` へ） |
| `DEV_BOT_USERNAME` | BOT ログイン名（既定 `player`） |
| `DEV_BOT_PASSWORD` | BOT パスワード |
| `DEV_BOT_TARGET` | 攻撃対象（既定 `offline_test`） |
| `DEV_BOT_INTERVAL_SEC` | 攻撃間隔（秒） |
| `DEV_BOT_HTTP_ORIGIN` | BOT 用 HTTP オリジン |
| `DEV_BOT_WS_URL` | BOT 用 WebSocket URL |

`DEV_AUTO_WIN` と `DEV_BOT` は **PVP モードのみ** 有効です。

---

## 永続化

### 初回起動時

- **PVP:** `data/pvp/state.json` がなければ新規ワールドを生成して保存
- **PVE:** ディレクトリ `data/pve/worlds/` を用意。ワールドは **初回 WebSocket 接続時** にプレイヤーごと生成
- **認証:** `data/auth.json` がなければテスト用アカウントを自動作成

### 旧データの移行

以前の `data/state.json` がある場合、PVP 初回起動時に `data/pvp/state.json` へ自動リネーム移行します。

### PVE のワールドレイアウト

```
data/pve/worlds/
  offline_test/
    state.json
  another_user/
    state.json
```

---

## HTTP API

| メソッド | パス | 認証 | 説明 |
|---------|------|------|------|
| GET | `/health` | 不要 | ヘルスチェック（`mode`, `world_cols`, `world_rows` を含む） |
| GET | `/api` | 不要 | API メタ情報 |
| GET | `/api/state` | Bearer JWT | ゲーム状態 JSON |
| GET | `/api/whoami` | Bearer JWT | トークンに紐づく `player_id` |
| POST | `/auth/register` | 不要 | 新規登録（PVE ではワールドは作らない） |
| POST | `/auth/login` | 不要 | ログイン |
| POST | `/auth/exchange` | Bearer JWT（任意モード） | このサーバーの mode 付き JWT を再発行（HUD 切替用） |
| POST | `/admin/wipe` | Bearer JWT + 管理者 | ワールド完全初期化（`{"confirm":"WIPE"}`） |

認証ヘッダー例:

```
Authorization: Bearer <JWTトークン>
```

### JWT とモード縛り

- ログイン/登録で発行される JWT には **`mode` クレーム**（`"pvp"` または `"pve"`）が含まれる
- 各サーバーは **自分の mode と一致するトークンのみ** `/api/state`・`/ws` で受理する
- PVE トークンを PVP サーバーに直接送ると **401**（逆も同様）
- **HUD の PVP/PVE 切替**はフロントが `POST /auth/exchange` を呼び、パスワードなしでトークンを再発行する
- PVP/PVE は **`data/auth.json` と `AUTH_JWT_SECRET` を共有**（同一アカウントで両モード利用可）

### 登録・ログインの挙動

- **PVP:** 登録/ログイン時に共有ワールドへプレイヤーを追加（`ensure_player_in_game`）
- **PVE:** 登録/ログインは **認証のみ**。ワールド生成は WS 接続時

### PVP のクライアント向け state

- `GET /api/state` および PVP WebSocket の送信 JSON は、**閲覧者本人の `players` エントリのみ**含む（他プレイヤーの所持品・編成は送らない）
- `territories`・`market_listings`・`log` はマップ/市場/戦歴用に共有
- サーバー内部の完全な `GameState` はメモリ上に保持（ブロードキャストチャネルはフル state、各クライアント送信時にフィルタ）

---

## WebSocket（`/ws`）

### 接続手順

1. `new WebSocket("ws://127.0.0.1:3000/ws")`（PVE は 3001）
2. **最初のメッセージ**で認証トークンを送信:

```json
{ "type": "auth", "token": "<JWT>" }
```

3. サーバーから `GameState` JSON が1回届く
4. 以降、行動 JSON を送信すると状態が更新され、ブロードキャストされる

### 行動の送信例

攻撃・援軍・探索は **`start_march`** で出発し、到着処理は **`march_scheduler`**（到着予定時刻に即時）と `tick_world` の両方で `tick_marches` が走る。

```json
{
  "action": "start_march",
  "kind": "attack",
  "from_territory_id": "c_24_24",
  "to_territory_id": "c_25_24",
  "count": 3,
  "owned_card_indices": [0, 1, 2],
  "monsters_per_body": [10, 10, 10],
  "speed_per_body": [5, 5, 5],
  "unit_name": "第一軍"
}
```

本拠編成の遠征では `owned_card_indices` / `stats_per_body` / `skills_per_body` 等を `count` と同数で送る。援軍は `formed_unit_id` を付与できる。

即時戦闘（移動なし・レガシー）の `attack` も残っているが、クライアントは `start_march` を使用する。

利用可能な `action` 一覧（抜粋）:

| action | 概要 |
|--------|------|
| `start_march` | 遠征開始（`kind`: `attack` / `deploy` / `explore`） |
| `deploy` | 領地へ即時援軍（到着済み扱い） |
| `attack` | 隣接マスへ即時攻撃（レガシー） |
| `build_base` | 前線基地建設 |
| `build_facility` | 施設建設 |
| `produce_monsters` | 魔獣生産（遠征中スロットは拒否） |
| `synthesize_card` | 魔獣合成 |
| `set_formed_units` | 編成ユニット保存 |
| `list_on_flea_market` | フリマ出品 |
| `buy_from_flea_market` | フリマ購入 |
| `cancel_flea_market_listing` | 出品取消 |
| `create_alliance` / `join_alliance` / `leave_alliance` / `donate_alliance` | 同盟（**PVP のみ**） |

> **注意:** 旧ターン制の `end_turn` は削除済み。スタミナは `tick_stamina` による時間経過回復。旧 `start_exploration` / `collect_exploration` も廃止（`start_march` に統合）。

### 時間経過（`tick_world`）のトリガー

`tick_world` は以下をまとめて実行する（`server/src/model/state.rs`）:

| 処理 | 内容 |
|------|------|
| `tick_facility_resources` | 施設による4資源加算（10分単位・オフライン catch-up） |
| `tick_stamina` | 魔獣スタミナ回復（1分単位） |
| `tick_ruins` | 遺跡期限切れ・ランダムスポーン |
| `tick_marches` | 遠征到着（攻撃戦闘・援軍配備・探索報酬・帰還ログ） |

**走るタイミング:**

- WebSocket 接続時（[`realtime.rs`](src/realtime.rs)）
- 行動適用後（同上）
- PVE: AI ターン開始時（[`ai_actions.rs`](src/ai_actions.rs)）
- バックグラウンド: [`world_scheduler.rs`](src/world_scheduler.rs)（`WORLD_TICK_SEC` ごと・資源・スタミナ・遺跡）
- 遠征到着: [`march_scheduler.rs`](src/march_scheduler.rs)（次の `arrives_at` までスリープし、到着後すぐ `tick_marches`）

**走らない例:** `GET /api/state` のみ、アイドル中に WS 未接続かつスケジューラ待ちの間は tick が遅延し得る（接続・次回スケジューラで catch-up）。

### 遠征到着（March arrival）

攻撃・援軍・探索の **到着処理はサーバー側で `arrives_at` 到達直後に実行** する。クライアントの表示待ちではない。

| 経路 | 役割 |
|------|------|
| [`march_scheduler.rs`](src/march_scheduler.rs) | 次の `arrives_at` までスリープし、時刻到達で `tick_march_arrivals`（戦闘・帰還ログ・broadcast） |
| 遠征出発・接続時 | `wake_march_scheduler` でスケジューラを再起動（待機中の古いタイマーを無効化） |
| `tick_world` | 接続時・アクション時の catch-up（資源・スタミナ・遺跡と同時に到着済み分も処理） |

`WORLD_TICK_SEC`（既定60秒）は資源回復等用。**到着だけ最大60秒待つことはない**（サーバー再起動後に有効）。

**PVE メモリ管理:**

- ブラウザを閉じる・ログアウトしても **サーバー側メモリ** は即解放しない（プレイヤー端末にはゲーム状態を保持しない）
- バックグラウンドは **攻撃・援軍・探索の到着のみ** 処理（帰還は次回接続時の `tick_world` で可）
- 到着処理またはプレイヤー操作のあと、**`EVICT_IDLE_MINUTES`（既定30分）** 操作がなければメモリ解放

- 進行状態は `PlayerData.marches`（`MarchMission`）にサーバー保存
- 攻撃到着時に `apply_attack_action`、探索到着時に `apply_explore_arrival`（**戦闘なし**）
- 占領成功時は自動で `Return` March を生成
- PVP では他プレイヤーの進行中遠征を `visible_marches` でマップ表示

### ブラウザコンソールでの疎通例

```javascript
const token = "..."; // /auth/login で取得
const ws = new WebSocket("ws://127.0.0.1:3000/ws");
ws.onopen = () => ws.send(JSON.stringify({ type: "auth", token }));
ws.onmessage = (e) => console.log("受信:", JSON.parse(e.data));
```

---

## PVP と PVE のゲームルール差

| 機能 | PVP | PVE |
|------|-----|-----|
| ワールド | 全員共有 | プレイヤーごと独立 |
| 他プレイヤーへの攻撃 | 可 | **不可**（AI 勢力 `ai_*` への攻撃は可） |
| 同盟 | 可 | **不可** |
| フリマ（プレイヤー） | 可 | 可 |
| フリマ（AI 出品） | — | AI は**購入のみ**（出品不可） |
| シーズンリセット | 有効 | 無効 |
| AI 王国 | なし | サーバー内部スケジューラで自動行動 |
| 遺跡スポーン | 共有ワールド1つ | ロード中の各 PVE ワールド |

PVE の AI 勢力数はマップ面積に比例（48×48 → 約5勢力）。`AI_FACTION_MAX` で上限を設定できます。

---

## テスト用アカウント

初回起動時に `data/auth.json` へ自動作成されます（`DEV_AUTH_PASSWORD` 既定: `test12345`）。

| ユーザー名 | 用途 |
|-----------|------|
| `offline_test` | 人間プレイヤー（開発用） |
| `player` | 敵 BOT（`DEV_BOT=1` 時） |

---

## フロントエンドとの接続

リポジトリルートの `.env`（または `.env.local`）例:

```env
VITE_PVP_API_ORIGIN=http://127.0.0.1:3000
VITE_PVE_API_ORIGIN=http://127.0.0.1:3001
```

クライアントはログイン時またはメニューから PVP/PVE を切り替え、該当オリジンへ再接続します。

フロント起動:

```bash
npm run dev
```

---

## トラブルシューティング

### ポートが使用中

```
エラー: 127.0.0.1:3000 は既に使用中です
```

- 既に起動中の `kingdom-server` を終了する
- または `PORT=3002` など別ポートを指定

### PVE を起動したつもりが PVP になる

- `SERVER_MODE=pve` を設定した**同じターミナル**で `cargo run` しているか確認
- `/health` の `mode` フィールドで実際のモードを確認

### `data/` が見つからない・状態が毎回リセットされる

- 起動ログの `project_root=` が意図したリポジトリルートか確認
- カスタム配置の場合は `PROJECT_ROOT` を明示（例: `$env:PROJECT_ROOT="C:\path\to\kingdom-game"`）
- `STATE_PATH` / `AUTH_PATH` に絶対パスを渡すことも可能

### WebSocket で `auth_required` / `auth_invalid`

- 接続後最初に `{ "type": "auth", "token": "..." }` を送っているか
- トークン期限切れ（7日）の場合は再ログイン

---

## 開発メモ

- 状態更新は純粋関数 `apply_action` が権威
- 戦闘内ターン（`turn_order_speed` 等）と、旧ゲーム全体ターン（`end_turn` / `GameState.turn`）は別概念。後者は削除済み
- バックグラウンド定数: `WORLD_TICK_SEC=60`, `FACILITY_RESOURCE_TICK_SEC=600`, `AI_TICK_INTERVAL_SEC=10`, 遺跡スポーンは `tick_ruins` 内（最大3、30%）
- PostgreSQL / Redis 連携は将来拡張（`migrations/` にスケルトンあり）
