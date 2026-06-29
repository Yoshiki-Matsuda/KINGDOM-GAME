# キングダムゲーム — コーディング規約

## Rust バックエンド (`server/src/`)

### モジュール構造
- `main.rs` で `mod` 宣言し、関心事ごとにディレクトリ化: `ai_actions/`, `model_actions/`, `db/`, `skills/`
- サブモジュールは `mod.rs` で再エクスポートし、上位モジュールから `pub(crate) use` する
- 大規模モジュールはサブディレクトリ分割する例: `model_actions/combat/`（`bonus.rs`, `damage.rs`, `resolve.rs`）, `ai_actions/`（`formation.rs`, `combat.rs`, `exploration.rs`）

### 命名規則
- `snake_case`: 関数、変数、モジュール名
- `PascalCase`: 型、enum、const
- `ALL_CAPS`: 定数
- 環境変数名は `ENV_` プレフィックス付き `pub const`（例: `ENV_SERVER_MODE`）

### エラーハンドリング
- エントリポイントでは `unwrap_or_else`, `unwrap_or`, `expect("serve")` を使用
- `?` chaining は避け、`match` + `eprintln!` + `std::process::exit(1)` で明示的

### ロギング
- すべての `println!` / `eprintln!` に `[kingdom-server]` プレフィックス

### 状態管理
- 共有ワールド: `Arc<RwLock<GameState>>` + `Arc<Mutex<()>>` ミューテーションロック
- プレイヤー別ワールド: `Arc<WorldManager>`
- 変更後は `persistence::save_state` / `save_player_world` でDBに保存

### 設定
- `config.rs` で環境変数名定数とデフォルト値を一元管理
- `env_string(key, default)` で環境変数取得（未設定時はデフォルト）

### サーバーモード
- `ServerMode::Pvp` / `ServerMode::Pve` はコアな区別
- 多くの関数が `server_mode` をパラメータとして受け取る
- クライアントビューは PVP 時に閲覧者スコープでフィルタリング、PVE 時はそのまま返す

### データベース
- SQLite (sqlx) を使用
- `db/mod.rs` でコネクションプール作成（`PRAGMA foreign_keys = ON`, `PRAGMA journal_mode = WAL`）
- リポジトリは `db/auth_repo.rs`, `db/player_repo.rs`, `db/world_repo.rs` に分離

### 並行性
- `tokio::spawn` でバックグラウンドタスク起動（スケジューラ等）
- `broadcast::channel` で PVP モードのクライアントへブロードキャスト
- ミューテーションロック内で状態変更＋永続化を atomic に実行

---

## TypeScript フロントエンド (`src/`)

### TypeScript コンパイラ設定
- `strict: true`, `verbatimModuleSyntax: true`, `moduleDetection: "force"`
- `noUnusedLocals: true`, `noUnusedParameters: true`
- `erasableSyntaxOnly: true`, `noFallthroughCasesInSwitch: true`

### 状態管理 (Valtio)
- 中央ストア: `store.ts` で `proxy()` を使用
- 全リアクティブ状態を `proxy` に集約し `subscribe()` で自動 `render()` 発火
- 非リアクティブ値（`WebSocket`, `Map` インスタンス等）は proxy 外で管理
- `export let` + setter 関数で外部から参照・更新可能
- 更新時は proxy 内の値と変数へ両方代入する

### インポート規則
- 相対パス使用
- `import type` で型のみインポート
- `import * as X` 禁止、名前付きインポートのみ
- 循環依存を防ぐため、型定義は利用側のファイル内に定義

### UI 構造
- 画面は `src/ui/` に配置（例: `home-screen.ts`, `formation-screen.ts`）
- 共通コンポーネントは `src/components/` に配置
- DOM 構築とロジックは分離（`app-dom.ts` で DOM 要素作成、`app-render.ts` でレンダリング）

### ネットワーク
- 専用モジュールに分離: `network/ws-client.ts`, `network/auth-client.ts`, `network/server-discovery.ts`, `network/mode-switch.ts`
- WebSocket 再接続ロジックは世代管理で古いソケットの上書きを防ぐ

### 定数
- サーバーアドレスは `config.ts` で管理（`VITE_*` 環境変数経由）
- `DEFAULT_*` 定数でデフォルト値を定義

---

## 両言語共通
- 日本語のコメントとドキュメントを使用
- 関心事ごとのモジュール分割を重視
- 既存コードとの互換性を維持した上での改修を優先
