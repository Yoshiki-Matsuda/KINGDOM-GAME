# クライアント UI 設計

フロントエンド（`src/ui/`）の画面構成・ナビゲーション・表示ルールをまとめる。ゲームルール本体は [KC_SPEC.md](./KC_SPEC.md)、技術構成は [ARCHITECTURE.md](./ARCHITECTURE.md) を参照。

---

## 1. ボトムメニュー

画面下部の固定メニュー（`bottom-menu.ts`）。各項目は `public/icons/menu-*.png` と対応する。

| data-action | アイコン | ツールチップ | 遷移先 |
|---|---|---|---|
| `home` | `menu-home.png` | 城下町 | 本拠地画面 |
| `map` | `menu-map.png` | 世界地図 | マップ |
| `formation` | `menu-formation.png` | 編成 | **編成ハブ**（§2） |
| `alliance` | `menu-alliance.png` | 同盟 | 同盟画面（PvE モードでは非表示） |
| `market` | `menu-market.png` | 取引所 | フリマ画面 |
| `history` | `menu-history.png` | 戦歴 | 戦闘履歴 |
| `status` | `menu-status.png` | ステータス | ステータス画面 |
| `inventory` | `menu-inventory.png` | 所持品 | インベントリ |

---

## 2. 編成フロー

ボトムメニュー「編成」は **いきなりユニット編成を開かない**。中間の行き先選択（編成ハブ）を挟む。

```
ボトムメニュー「編成」
    └─ 編成ハブ（魔獣一覧 / ユニット編成）
           ├─ 魔獣一覧 … 所持魔獣の一覧。タップで詳細・生産
           └─ ユニット編成 … 3枠編成・コスト上限・ユニット追加
```

- 各画面の「戻る」で編成ハブに戻る。ハブの「閉じる」でオーバーレイ全体を閉じる。
- ユニット選択画面などからの「編成を開く」は、ハブを経由せず **ユニット編成に直行**（`showFormationScreen()`）。

実装: `src/ui/formation-screen.ts`（`showFormationHub` / `showFormationScreen`）

編成詳細（魔獣一覧タップ）では Lv・EXP・スタミナ・生産 UI を表示（`formation-card-detail.ts`）。遠征中スロットは生産ボタンを無効化する。

---

## 3. 画面ヘッダーとアイコン

サブ画面のヘッダーは、ボトムメニューと同じ PNG アイコン + タイトルテキストで統一する（絵文字は使わない）。

| 画面 | タイトル | アイコン |
|---|---|---|
| 本拠地 | 本拠地 | `menu-home.png` |
| 編成ハブ | 編成 | `menu-formation.png` |
| ユニット編成 | ユニット編成 | `menu-formation.png` |
| 同盟 | 同盟 | `menu-alliance.png` |
| 取引所 | 取引所 | `menu-market.png` |
| 戦歴 | 戦歴 | `menu-history.png` |
| ステータス | ステータス | `menu-status.png` |
| 所持品 | 所持品 | `menu-inventory.png` |

共通ヘルパー: `src/ui/screen-header.ts`（`renderScreenHeaderTitle`）

---

## 4. 資源・通貨の表示

食料・木材・石材・鉄・ゴールドは **アイコン + 数値のみ**（ラベルや「所持金:」「G」表記は使わない）。

| 種別 | アイコン |
|---|---|
| 食料 | 🌾 |
| 木材 | 🪵 |
| 石材 | 🪨 |
| 鉄 | ⛏ |
| ゴールド | 💰 |

- 数値は `toLocaleString()` で桁区切り。
- ステータス画面の資源行は、アイコン列を固定幅にし数値の開始位置を揃える。
- 共通ヘルパー: `src/ui/resource-display.ts`

### HUD（マップ画面）

- 左上 HUD に接続状態 + 5 種資源（ゴールド含む）を表示。
- **マップ以外の画面ではグローバル HUD を非表示**（各画面ヘッダーと重なるため）。
- 本拠地画面ヘッダーには資源バーを埋め込む。

### フリマ

- ヘッダー右: `💰` + 所持ゴールド
- 出品価格・出品カードの価格表示も同形式

---

## 5. レイアウト上の注意

- マップ以外の画面ヘッダーは、右上の設定ボタン（歯車）と重ならないよう `padding-right: 3.5rem` を確保（`#app[data-screen]` による CSS）。
- フリマ「出品する」ボタンはコンテンツ幅（全幅に伸ばさない）。

---

## 6. 文言

- ユーザー向け画面に開発者向けメッセージ（JSON アクション名など）は表示しない。
- 例: 探索未実施時は「進行中の遠征はありません。」（ステータス画面の `marches` 一覧）

---

## 7. 実装状況メモ（サーバー連携）

| 機能 | UI | サーバー |
|---|---|---|
| 攻撃・援軍・探索派遣 | マップ右クリック → ユニット選択 → `start_march` | `StartMarch` + `tick_marches` |
| 探索 | 自領マスから派遣。**戦闘なし・確実成功** | `apply_explore_arrival` |
| 遠征中の魔獣生産 | 編成詳細でボタン無効 | `produce_monsters` が遠征スロットを拒否 |
| 魔獣 Lv/Exp/スタミナ | 編成詳細モーダルに表示 | `card_levels` / `card_exp` / `card_stamina` |
| ステータス振り分け | 詳細の「育成」ボタン（未配分ptがあるときのみ）→ 振り分けダイアログ | `allocate_card_stats` / `card_stat_bonuses` |
| 行軍の旗・線 | `marches` / `visible_marches` から描画 | `PlayerData.marches` 永続 |

### 編成詳細（`formation-card-detail.ts`）

表示項目: Lv・EXP・スタミナ・魔獣数・**実効ステータス（基礎 (+振り分け)）**・スキル・**育成（未配分pt時）**・生産。**遠征中スロットは生産不可**。
