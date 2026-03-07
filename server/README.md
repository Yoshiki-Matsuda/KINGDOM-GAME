# kingdom-server（Rust バックエンド）

- HTTP: `/health`, `/api`, `/api/state`（ゲーム状態 JSON）
- WebSocket: `/ws` — 接続時に状態を1回送信。クライアントから `{"action":"end_turn"}` を送るとターン進行し、全接続に状態を配信。

## 前提

- [Rust](https://www.rust-lang.org/tools/install) をインストールし、`cargo` が PATH に通っていること。

## ビルド・実行

```bash
cd server
cargo run
```

- **ローカル開発で戦闘を有利にする**: 環境変数 `DEV_AUTO_WIN=1` を付けて起動すると、攻撃側が10倍有利になります（送ったエナジー1＝敵10相当）。戦闘の計算・ログは通常どおり出るので、挙動を確認しやすいです。
  ```bash
  # Windows (PowerShell)
  $env:DEV_AUTO_WIN="1"; cargo run
  # Linux / macOS
  DEV_AUTO_WIN=1 cargo run
  ```

- 起動後: `http://127.0.0.1:3000/health` でヘルスチェック
- `http://127.0.0.1:3000/api` で API 情報
- `http://127.0.0.1:3000/api/state` でゲーム状態（JSON）
- `ws://127.0.0.1:3000/ws` で WebSocket（接続時に状態を1回受信）

## 疎通確認

- **ブラウザ**: アドレスバーに `http://127.0.0.1:3000/health` を入力して開く（JSON が表示されればOK）

- **PowerShell**:
  ```powershell
  Invoke-RestMethod -Uri http://127.0.0.1:3000/health
  Invoke-RestMethod -Uri http://127.0.0.1:3000/api
  ```

- **コマンドプロンプト (cmd)**（`Invoke-RestMethod` は PowerShell 専用なので使えない）:
  ```cmd
  curl http://127.0.0.1:3000/health
  curl http://127.0.0.1:3000/api
  ```
  （Windows 10 以降は `curl` が標準で使えます）

- **WebSocket（ブラウザ開発者ツールのコンソールで）**  
  サーバー起動後、ブラウザで `http://127.0.0.1:3000` を開き、F12 → コンソールで:
  ```javascript
  const ws = new WebSocket("ws://127.0.0.1:3000/ws");
  ws.onmessage = (e) => console.log("受信:", e.data);
  ws.send(JSON.stringify({ action: "end_turn" })); // ターン進行（全クライアントに配信）
  ```
  `受信: {"turn":1,"phase":"idle"}` の後、`end_turn` 送信で `{"turn":2,"phase":"idle"}` が届けばOK。

## 次のステップ

- PostgreSQL スキーマと永続化
- Redis によるターンロック（複数ワールド対応時）
- 領地・エナジー・戦闘ロジックのデータモデル追加
