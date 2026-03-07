# Rust のインストール（Windows）

キングダムゲームのバックエンド（`server/`）を動かすために Rust を入れます。

---

## 方法1: rustup（推奨）

**rustup** は Rust の公式インストーラーで、本体・ツールチェーン・バージョン切り替えをまとめて管理できます。

### 手順

1. **公式サイトでインストーラーを取得**
   - https://www.rust-lang.org/tools/install
   - 「DOWNLOAD RUSTUP-INIT.EXE (64-BIT)」をクリックしてダウンロード

2. **実行**
   - ダウンロードした `rustup-init.exe` を実行
   - 画面の指示に従う（通常は「1」を押して標準インストールでOK）

3. **PATH を通す**
   - インストール終了時に「Press the Enter key to continue」と出たら Enter
   - **新しい PowerShell またはコマンドプロンプト**を開く（既に開いている窓は閉じて開き直す）
   - 次のコマンドでバージョンが表示されれば成功です：

   ```powershell
   rustc --version
   cargo --version
   ```

### よくある注意点

- **「cargo が認識されない」**  
  インストール直後は、**新しいターミナル**を開かないと PATH が反映されません。Cursor のターミナルも一度閉じて開き直してください。

- **Visual Studio の C++ ビルドツール**  
  初回で「Visual Studio C++ Build Tools が必要」と出た場合は、表示されるリンクから「Build Tools for Visual Studio」を入れ、  
  「C++ によるデスクトップ開発」ワークロードにチェックを入れてインストールします。

---

## 方法2: winget（コマンドで入れたい場合）

Windows のパッケージマネージャー **winget** でも入れられます。

```powershell
winget install Rustlang.Rustup
```

インストール後、**新しいターミナル**を開いてから：

```powershell
cargo --version
```

で確認してください。

---

## インストール後の確認

プロジェクトのサーバーをビルド・実行して確認します。

```powershell
cd c:\Users\owner\kingdom-game\server
cargo build
cargo run
```

ブラウザまたは別の PowerShell で：

```powershell
Invoke-RestMethod -Uri http://127.0.0.1:3000/health
```

`status: ok` などが返ってくれば Rust 環境は問題ありません。

---

## トラブルシューティング: `linker 'link.exe' not found`

このエラーは **Rust の標準（MSVC）ターゲット** が、Windows の C++ リンカー `link.exe` を探しているが見つからないときに出ます。

### 対処A: Visual Studio Build Tools を入れる（推奨）

MSVC のまま使う場合は、以下のどちらかをインストールします。

1. **Build Tools for Visual Studio（Visual Studio なしでOK）**
   - https://visualstudio.microsoft.com/ja/downloads/
   - 下の方の「**Visual Studio 2022 の Build Tools**」をダウンロード
   - インストーラーで **「C++ によるデスクトップ開発」** ワークロードにチェックを入れてインストール
   - 完了後、**ターミナルを開き直して** `cargo build` を再実行

2. **Visual Studio 2022 本体** を持っている場合  
   - インストーラーで「C++ によるデスクトップ開発」が入っているか確認し、入っていなければ追加

※ VS Code / Cursor は別製品なので、このリンカーは含まれません。

### 対処B: GNU ツールチェーンに切り替える（Visual Studio を入れたくない場合）

Visual Studio を入れずに済ませるには、Rust を **GNU ターゲット** に切り替えます。

1. **GNU ターゲットを追加**
   ```powershell
   rustup target add x86_64-pc-windows-gnu
   ```

2. **デフォルトのツールチェーンを GNU に変更**
   ```powershell
   rustup default stable-x86_64-pc-windows-gnu
   ```

3. **MinGW-w64 が必要**
   - https://www.mingw-w64.org/downloads/ などから 64bit 版を入れる  
   - または **MSYS2** を入れ、その中で `pacman -S mingw-w64-ucrt-x86_64-gcc` を実行してから、その `bin` を PATH に追加

GNU にすると `link.exe` は使わず、GCC のリンカーでビルドされます。MSVC の方が Windows ネイティブでは一般的なので、**まずは対処Aを試す**のがおすすめです。

---

## サーバーの永続化とワイプ

- **永続化**: ゲーム状態は `server/data/state.json` に保存されます。起動時にこのファイルがあれば読み込み、**サーバーを再起動しても前の状態から再開**できます（メンテナンスで落としても大丈夫です）。
- **ワイプ**: 完全に初期化したいときだけ、管理用に **POST /admin/wipe** を呼んでください。ワイプ時のみ全マスが再生成され、新規ゲームとして始まります。通常の再起動ではワイプは呼びません。

**開発で DB/サーバーに繋がない場合**: フロントのみでマップを確認したいときは、`.env` または `.env.local` に `VITE_USE_MOCK_STATE=true` を指定してください。仮の 48x48 マスデータで表示され、WebSocket には接続しません（HUD に「開発用マスデータ」と出ます）。

**マップが重い場合**: 地形イラストをやめて色だけの描画にすると軽くなります。`.env` に `VITE_USE_TERRAIN_IMAGES=false` を指定してください。

---

## 参考リンク

- [Rust 公式インストールページ](https://www.rust-lang.org/tools/install)
- [Rust 日本語ドキュメント](https://doc.rust-jp.rs/)
