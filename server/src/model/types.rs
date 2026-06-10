use super::*;

/// デフォルトの操作プレイヤーID
pub const DEFAULT_PLAYER_ID: &str = "player";

/// 本拠地のデフォルト座標
pub const HOME_COL: u8 = 24;
pub const HOME_ROW: u8 = 24;

/// 新規参加プレイヤーの本拠は、既存プレイヤー本拠からこのマンハッタン距離以上離す。
pub(crate) const MIN_HOME_SEPARATION: u8 = 8;

/// 所持魔獣1枠あたりの魔獣数の共通下限（戦闘で0になっても1に戻す）
pub const MIN_MONSTER_COUNT_PER_CARD_SLOT: u32 = 1;

/// 所持魔獣1枠あたりの魔獣数の共通上限（全魔獣共通）
pub const MAX_MONSTER_COUNT_PER_CARD_SLOT: u32 = 9999;

/// KC準拠の4種基本資源 + ゴールド（フリマ用通貨）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resources {
    pub food: u64,
    pub wood: u64,
    pub stone: u64,
    pub iron: u64,
    #[serde(default)]
    pub gold: u64,
}

impl Default for Resources {
    fn default() -> Self {
        Self { food: 500, wood: 500, stone: 500, iron: 500, gold: 1000 }
    }
}

/// フリーマーケット出品物の種別
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MarketItemType {
    #[serde(rename = "card")]
    Card { card_id: u32 },
    #[serde(rename = "item")]
    Item { item_id: String, count: u32 },
    #[serde(rename = "resource")]
    Resource { resource_type: String, amount: u64 },
}

/// フリーマーケットの出品情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketListing {
    pub listing_id: String,
    pub seller_id: String,
    pub item: MarketItemType,
    pub price: u64,
    pub listed_at: u64,
}

/// プレイヤー固有のデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    /// プレイヤーID
    pub player_id: String,
    /// 本拠地の領地ID
    pub home_territory_id: String,
    /// インベントリ
    #[serde(default)]
    pub inventory: Vec<InventoryItem>,
    /// 建設済み施設一覧
    #[serde(default)]
    pub facilities: Vec<BuiltFacility>,
    /// 所持魔獣（各要素は魔獣マスタID・列名は `owned_cards`）
    #[serde(default)]
    pub owned_cards: Vec<u32>,
    #[serde(default)]
    pub card_skill_levels: std::collections::HashMap<usize, [u8; 3]>,
    /// 援軍を送れる他プレイヤーのID（クラン・配下など）
    #[serde(default)]
    pub allied_player_ids: Vec<String>,
    /// 4種基本資源（食料・木材・石材・鉄）
    #[serde(default)]
    pub resources: Resources,
    /// 最後に資源を回収した時刻（Unix timestamp ms）
    #[serde(default = "default_now_ms")]
    pub last_resource_tick: u64,
    /// 所持魔獣スロットごとのレベル（owned_cards と同じ長さ）
    #[serde(default)]
    pub card_levels: Vec<u32>,
    /// 所持魔獣スロットごとの経験値
    #[serde(default)]
    pub card_exp: Vec<u64>,
    /// 所持魔獣スロットごとのスタミナ（KC: 出撃・探索に使用）
    #[serde(default)]
    pub card_stamina: Vec<u32>,
    /// 所持魔獣スロットごとのステータスポイント（KC: Lvアップ・EXCELLENT探索で取得、振り分けでステータス強化）
    #[serde(default)]
    pub card_status_points: Vec<u32>,
    /// 所持魔獣スロットごとの「休息中」解除時刻（ms）。今より未来ならFAILURE後のダウン中
    #[serde(default)]
    pub card_rest_until: Vec<u64>,
    /// 所持魔獣スロットごとの覚醒フラグ（KC: Lv99超え可）
    #[serde(default)]
    pub card_awakened: Vec<bool>,
    /// 所持魔獣スロットごとの強化魔獣フラグ（KC: ★魔獣。ステータス10%増、コスト25%OFF）
    #[serde(default)]
    pub card_enhanced: Vec<bool>,
    /// 所持魔獣スロットごとの現在魔獣数（`owned_cards` と同じ長さ・本拠 `body_monster_counts` と同期）
    #[serde(default)]
    pub card_monster_counts: Vec<u32>,
    /// 探索レベル（同時派遣数に影響）
    #[serde(default)]
    pub exploration_level: u32,
    /// 探索スコア（探索レベルアップ用）
    #[serde(default)]
    pub exploration_score: u64,
    /// ユニットコスト上限（KC: 初期4.0、伝承資料庫等で増加）
    #[serde(default = "default_unit_cost_cap")]
    pub unit_cost_cap: f32,
    /// DP（ダンジョンポイント・合成等）
    #[serde(default)]
    pub dungeon_points: u64,
    /// CP（課金ポイント相当・開発用）
    #[serde(default)]
    pub charge_points: u64,
    #[serde(default)]
    pub explorations: Vec<ExplorationMission>,
}
fn default_unit_cost_cap() -> f32 {
    4.0
}

pub(crate) fn default_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

impl PlayerData {
    pub fn new(player_id: String, home_territory_id: String) -> Self {
        Self {
            player_id,
            home_territory_id,
            inventory: default_dev_inventory(),
            facilities: vec![],
            owned_cards: default_owned_cards(),
            card_skill_levels: std::collections::HashMap::new(),
            allied_player_ids: vec![],
            resources: Resources::default(),
            last_resource_tick: default_now_ms(),
            card_levels: vec![],
            card_exp: vec![],
            card_stamina: vec![],
            card_status_points: vec![],
            card_rest_until: vec![],
            card_awakened: vec![],
            card_enhanced: vec![],
            card_monster_counts: initial_card_monster_counts_for_owned(&default_owned_cards()),
            exploration_level: 1,
            exploration_score: 0,
            unit_cost_cap: default_unit_cost_cap(),
            dungeon_points: 0,
            charge_points: 0,
            explorations: vec![],
        }
    }
}

/// KC準拠の同盟データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alliance {
    pub id: String,
    pub name: String,
    pub leader_id: String,
    pub member_ids: Vec<String>,
    /// 同盟の保有ポイント（領地レベル合計で毎ターン加算）
    #[serde(default)]
    pub territory_points: u64,
    #[serde(default = "default_alliance_level")]
    pub level: u32,
    #[serde(default)]
    pub donated_total: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_alliance_id: Option<String>,
    #[serde(default)]
    pub child_alliance_ids: Vec<String>,
}
fn default_alliance_level() -> u32 {
    1
}

/// 探索派遣（KC準拠・ホーム外領地への時間経過ミッション）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationMission {
    pub mission_id: String,
    pub territory_id: String,
    pub started_at: u64,
    pub completes_at: u64,
    #[serde(default)]
    pub card_indices: Vec<usize>,
}

/// 施設の配置座標（ホームマップ上）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct FacilityPosition {
    pub col: u8,
    pub row: u8,
}

/// 建設済み施設
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltFacility {
    pub facility_id: String,
    pub level: u8,
    /// 建設完了時刻（Unix timestamp ms）。Noneなら完了済み
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_complete_at: Option<u64>,
    /// 配置座標（ホームマップ上）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<FacilityPosition>,
}

/// 1つの領地。所有者なしは中立（PvE の敵または未占拠）。
/// レベルは地形とリンクし（1=平原〜5=山岳）、PvE 敵の強さにも使う。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Territory {
    pub id: String,
    pub name: String,
    /// マスのレベル（1〜5）。地形見た目・PvE 難易度と連動。
    #[serde(default = "default_level")]
    pub level: u8,
    /// 所有者 ID。None は中立
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
    /// 配置魔獣数（体数）
    pub troops: u32,
    /// 体ごとのモンスター数。len() == troops のときのみ有効。未設定時は戦闘で各体を1として扱う。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_monster_counts: Option<Vec<u32>>,
    /// 体ごとの表示名（戦闘ログ用）。len() == troops のときのみ有効。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_names: Option<Vec<String>>,
    /// 遺跡情報（存在する場合）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ruin: Option<RuinInfo>,
    /// 前線基地フラグ（KC準拠: 占領地に建設して前線を拡大）
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_base: bool,
    /// 現在の耐久値（PvP拠点・塔）。0なら中立即占領互換
    #[serde(default)]
    pub durability: u32,
    /// 最大耐久値
    #[serde(default)]
    pub max_durability: u32,
    /// 塔レベル（1-7）。0は通常マス
    #[serde(default)]
    pub tower_level: u8,
}

fn default_level() -> u8 {
    1
}

/// territory ID (例: "c_10_5") から座標 (col, row) を抽出
pub(crate) fn parse_territory_coords(id: &str) -> Option<(i32, i32)> {
    if id.starts_with("c_") {
        let parts: Vec<&str> = id[2..].split('_').collect();
        if parts.len() == 2 {
            if let (Ok(col), Ok(row)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                return Some((col, row));
            }
        }
    }
    None
}

/// ゲーム全体の状態。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub turn: u32,
    pub phase: String,
    pub territories: Vec<Territory>,
    /// バックエンドで発生した行動のログ。ユーザーはこれを読むだけ。
    #[serde(default)]
    pub log: Vec<String>,
    /// 全プレイヤーのデータ（プレイヤーID -> PlayerData）
    pub players: HashMap<String, PlayerData>,
    /// 同盟一覧（KC準拠: 複数プレイヤーが同盟を結成）
    #[serde(default)]
    pub alliances: Vec<Alliance>,
    /// シーズン情報
    #[serde(default)]
    pub season: SeasonInfo,
    /// フリーマーケット出品一覧
    #[serde(default)]
    pub market_listings: Vec<MarketListing>,
}

/// KC準拠のシーズン情報（一定期間でマップ・領地リセット）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonInfo {
    pub season_number: u32,
    /// シーズン開始時刻（Unix timestamp ms）
    pub started_at: u64,
    /// シーズン期間（ms）。デフォルト90日
    pub duration_ms: u64,
}

impl Default for SeasonInfo {
    fn default() -> Self {
        Self {
            season_number: 1,
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            duration_ms: 90 * 24 * 60 * 60 * 1000,
        }
    }
}

impl GameState {
}
