//! ゲーム状態・行動の共通データモデル（フロントと同一構造で JSON 化）
//!
//! 設計: サーバー権威・データ駆動。状態更新は純粋関数 `apply_action` のみ。
//! 最終形 PvPvE を想定し、owner_id でプレイヤー／中立を区別。

use std::collections::{hash_map::Entry, HashMap};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::skills::SkillData;
use crate::items::InventoryItem;
use crate::ruins::RuinInfo;
use crate::cards::get_card;

/// デフォルトのプレイヤーID（シングルプレイ時）
pub const DEFAULT_PLAYER_ID: &str = "player";

/// 本拠地のデフォルト座標
pub const HOME_COL: u8 = 24;
pub const HOME_ROW: u8 = 24;

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

/// 建設済み施設
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltFacility {
    pub facility_id: String,
    pub level: u8,
    /// 建設完了時刻（Unix timestamp ms）。Noneなら完了済み
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_complete_at: Option<u64>,
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
    #[serde(default)]
    pub players: HashMap<String, PlayerData>,
    
    // === 後方互換性のため残す（シングルプレイ時はここに直接入る） ===
    /// 援軍を送れる領の owner_id 一覧（自領 "player" に加え、クラン・配下など）。空なら自領のみ。
    #[serde(default)]
    pub deployable_owner_ids: Vec<String>,
    /// プレイヤーのインベントリ（シングルプレイ用、マルチでは players を参照）
    #[serde(default)]
    pub inventory: Vec<InventoryItem>,
    /// 建設済み施設一覧（シングルプレイ用）
    #[serde(default)]
    pub facilities: Vec<BuiltFacility>,
    /// プレイヤーの所持魔獣（シングルプレイ用・`owned_cards`）
    #[serde(default)]
    pub owned_cards: Vec<u32>,
    #[serde(default)]
    pub card_skill_levels: std::collections::HashMap<usize, [u8; 3]>,
    /// 4種基本資源（シングルプレイ用、players からのコピー）
    #[serde(default)]
    pub resources: Resources,
    /// 同盟一覧（KC準拠: 複数プレイヤーが同盟を結成）
    #[serde(default)]
    pub alliances: Vec<Alliance>,
    /// シーズン情報
    #[serde(default)]
    pub season: SeasonInfo,
    /// フリーマーケット出品一覧
    #[serde(default)]
    pub market_listings: Vec<MarketListing>,
    /// シングルプレイ用: 魔獣スロットごとのレベル（owned_cards と同じ長さ）
    #[serde(default)]
    pub card_levels: Vec<u32>,
    #[serde(default)]
    pub card_exp: Vec<u64>,
    #[serde(default)]
    pub card_stamina: Vec<u32>,
    #[serde(default)]
    pub exploration_level: u32,
    #[serde(default)]
    pub exploration_score: u64,
    #[serde(default = "default_unit_cost_cap")]
    pub unit_cost_cap: f32,
    #[serde(default)]
    pub dungeon_points: u64,
    #[serde(default)]
    pub charge_points: u64,
    #[serde(default)]
    pub explorations: Vec<ExplorationMission>,
    /// シングルプレイ用: 魔獣スロットごとの魔獣数（`players` 内とミラー）
    #[serde(default)]
    pub card_monster_counts: Vec<u32>,
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

pub(crate) fn build_game_state(
    state: &GameState,
    turn: u32,
    territories: Vec<Territory>,
    log: Vec<String>,
    players: HashMap<String, PlayerData>,
    inventory: Vec<InventoryItem>,
    facilities: Vec<BuiltFacility>,
    owned_cards: Vec<u32>,
    card_skill_levels: std::collections::HashMap<usize, [u8; 3]>,
) -> GameState {
    let resources = players.get(DEFAULT_PLAYER_ID)
        .map(|p| p.resources.clone())
        .unwrap_or_default();
    let p = players.get(DEFAULT_PLAYER_ID).cloned();
    GameState {
        turn,
        phase: state.phase.clone(),
        territories,
        log,
        players,
        deployable_owner_ids: state.deployable_owner_ids.clone(),
        inventory,
        facilities,
        owned_cards,
        card_skill_levels,
        resources,
        alliances: state.alliances.clone(),
        season: state.season.clone(),
        market_listings: state.market_listings.clone(),
        card_levels: p.as_ref().map(|x| x.card_levels.clone()).unwrap_or_else(|| state.card_levels.clone()),
        card_exp: p.as_ref().map(|x| x.card_exp.clone()).unwrap_or_else(|| state.card_exp.clone()),
        card_stamina: p.as_ref().map(|x| x.card_stamina.clone()).unwrap_or_else(|| state.card_stamina.clone()),
        exploration_level: p.as_ref().map(|x| x.exploration_level).unwrap_or(state.exploration_level),
        exploration_score: p.as_ref().map(|x| x.exploration_score).unwrap_or(state.exploration_score),
        unit_cost_cap: p.as_ref().map(|x| x.unit_cost_cap).unwrap_or(state.unit_cost_cap),
        dungeon_points: p.as_ref().map(|x| x.dungeon_points).unwrap_or(state.dungeon_points),
        charge_points: p.as_ref().map(|x| x.charge_points).unwrap_or(state.charge_points),
        explorations: p.as_ref().map(|x| x.explorations.clone()).unwrap_or_else(|| state.explorations.clone()),
        card_monster_counts: p
            .as_ref()
            .map(|x| x.card_monster_counts.clone())
            .unwrap_or_else(|| state.card_monster_counts.clone()),
    }
}

pub(crate) const MAX_LOG_LINES: usize = 2000;

impl Default for GameState {
    fn default() -> Self {
        let home_territory_id = format!("c_{}_{}", HOME_COL, HOME_ROW);
        let default_player = PlayerData::new(
            DEFAULT_PLAYER_ID.to_string(),
            home_territory_id,
        );
        let mut players = HashMap::new();
        players.insert(DEFAULT_PLAYER_ID.to_string(), default_player.clone());
        
        Self {
            turn: 1,
            phase: "idle".to_string(),
            territories: default_territories(),
            log: vec![],
            players,
            deployable_owner_ids: vec![],
            inventory: default_player.inventory.clone(),
            owned_cards: default_player.owned_cards.clone(),
            card_skill_levels: default_player.card_skill_levels.clone(),
            facilities: default_player.facilities.clone(),
            resources: default_player.resources,
            alliances: vec![],
            season: SeasonInfo::default(),
            market_listings: vec![],
            card_levels: vec![],
            card_exp: vec![],
            card_stamina: vec![],
            exploration_level: 1,
            exploration_score: 0,
            unit_cost_cap: default_unit_cost_cap(),
            dungeon_points: 0,
            charge_points: 0,
            explorations: vec![],
            card_monster_counts: default_player.card_monster_counts.clone(),
        }
    }
}

/// 開発用: 初期アイテムを追加
fn default_dev_inventory() -> Vec<InventoryItem> {
    vec![
        // 基本素材（大量）
        InventoryItem { item_id: "ancient_stone".to_string(), count: 500 },
        InventoryItem { item_id: "rusty_gear".to_string(), count: 200 },
        InventoryItem { item_id: "rotten_wood".to_string(), count: 300 },
        InventoryItem { item_id: "broken_brick".to_string(), count: 200 },
        // 中級素材
        InventoryItem { item_id: "mystic_crystal".to_string(), count: 100 },
        InventoryItem { item_id: "magic_shard".to_string(), count: 150 },
        InventoryItem { item_id: "refined_iron".to_string(), count: 100 },
        InventoryItem { item_id: "reinforced_fiber".to_string(), count: 80 },
        InventoryItem { item_id: "ancient_blueprint".to_string(), count: 30 },
        // 上級素材
        InventoryItem { item_id: "shining_magicstone".to_string(), count: 50 },
        InventoryItem { item_id: "golden_gear".to_string(), count: 20 },
        // レア素材
        InventoryItem { item_id: "guardian_core".to_string(), count: 10 },
        InventoryItem { item_id: "ancient_kings_seal".to_string(), count: 5 },
        InventoryItem { item_id: "dragon_scale".to_string(), count: 3 },
    ]
}

/// 初期所持魔獣（北欧神話キャラ）
fn default_owned_cards() -> Vec<u32> {
    // 魔獣マスタID 0〜9: 初期所持（各1枠）
    vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
}

pub(crate) fn default_monster_count_for_card_id(card_id: u32) -> u32 {
    get_card(card_id)
        .map(|c| c.stats.monster_count)
        .filter(|&m| m > 0)
        .unwrap_or(1)
        .clamp(MIN_MONSTER_COUNT_PER_CARD_SLOT, MAX_MONSTER_COUNT_PER_CARD_SLOT)
}

pub(crate) fn initial_card_monster_counts_for_owned(owned: &[u32]) -> Vec<u32> {
    owned
        .iter()
        .copied()
        .map(default_monster_count_for_card_id)
        .collect()
}

pub(crate) fn ensure_card_monster_counts(player: &mut PlayerData) {
    let n = player.owned_cards.len();
    if player.card_monster_counts.len() > n {
        player.card_monster_counts.truncate(n);
    }
    while player.card_monster_counts.len() < n {
        let idx = player.card_monster_counts.len();
        let id = player.owned_cards[idx];
        player
            .card_monster_counts
            .push(default_monster_count_for_card_id(id));
    }
    for c in &mut player.card_monster_counts {
        *c = (*c).clamp(MIN_MONSTER_COUNT_PER_CARD_SLOT, MAX_MONSTER_COUNT_PER_CARD_SLOT);
    }
}

/// 本拠領地の `troops` / `body_monster_counts` をプレイヤーの所持魔獣列と一致させる
pub(crate) fn sync_home_territory_body_counts_from_player(
    territories: &mut [Territory],
    player: &PlayerData,
) {
    let home_id = player.home_territory_id.as_str();
    let Some(tidx) = get_territory_index(territories, home_id) else {
        return;
    };
    let n = player.owned_cards.len() as u32;
    territories[tidx].troops = n;
    territories[tidx].body_monster_counts = Some(player.card_monster_counts.clone());
}

/// この領地に援軍を送れるか（自領・クランメンバー・配下プレイヤーの領）。
pub(crate) fn can_receive_reinforcement(territories: &[Territory], deployable_owner_ids: &[String], territory_id: &str) -> bool {
    let Some(idx) = get_territory_index(territories, territory_id) else {
        return false;
    };
    let owner = match &territories[idx].owner_id {
        Some(id) => id.as_str(),
        None => return false,
    };
    owner == "player" || deployable_owner_ids.iter().any(|id| id.as_str() == owner)
}

pub(crate) fn territory_name<'a>(territories: &'a [Territory], id: &'a str) -> &'a str {
    territories.iter().find(|t| t.id.as_str() == id).map(|t| t.name.as_str()).unwrap_or(id)
}

pub(crate) fn push_log(log: &mut Vec<String>, line: String) {
    let ts = default_now_ms();
    log.push(format!("[ts:{}]{}", ts, line));
    if log.len() > MAX_LOG_LINES {
        log.drain(0..log.len() - MAX_LOG_LINES);
    }
}

/// 旧ログ（[ts:] プレフィックスなし）にタイムスタンプを付与するマイグレーション。
/// 既存行は起動時刻から逆算して等間隔に並べる。
pub fn migrate_log_timestamps(state: &mut GameState) {
    let now = default_now_ms();
    let total = state.log.len() as u64;
    if total == 0 { return; }
    let mut migrated = false;
    for (i, line) in state.log.iter_mut().enumerate() {
        if line.starts_with("[ts:") { continue; }
        let synthetic_ts = now.saturating_sub((total - i as u64) * 500);
        *line = format!("[ts:{}]{}", synthetic_ts, line);
        migrated = true;
    }
    if migrated {
        println!("[kingdom-server] 旧ログ {} 件にタイムスタンプを付与しました", total);
    }
}

/// グリッドサイズ（クライアントの GRID_COLS / GRID_ROWS と一致）
pub(crate) const GRID_COLS: u8 = 48;
pub(crate) const GRID_ROWS: u8 = 48;

fn terrain_name(level: u8) -> &'static str {
    match level {
        1 => "平原",
        2 => "丘陵",
        3 => "森",
        4 => "山地",
        5 => "山岳",
        6 => "川",
        7 => "深域",
        8 => "険境",
        9 => "魔境",
        _ => "平原",
    }
}

/// ランダムな地形レベル（1〜6）のグリッドを生成。1回スムージングしてまとまりを出し、川(6)を別途配置。
fn random_level_grid() -> Vec<Vec<u8>> {
    let mut rng = rand::thread_rng();
    let grid: Vec<Vec<u8>> = (0..GRID_ROWS as usize)
        .map(|_| (0..GRID_COLS as usize).map(|_| rng.gen_range(1..=9)).collect())
        .collect();
    // 隣接と平均してまとまりのある地形に（1〜5のみ）
    let mut next = grid.clone();
    for row in 0..GRID_ROWS as usize {
        for col in 0..GRID_COLS as usize {
            let mut sum = grid[row][col] as u32;
            let mut n = 1u32;
            if row > 0 {
                sum += grid[row - 1][col] as u32;
                n += 1;
            }
            if row < GRID_ROWS as usize - 1 {
                sum += grid[row + 1][col] as u32;
                n += 1;
            }
            if col > 0 {
                sum += grid[row][col - 1] as u32;
                n += 1;
            }
            if col < GRID_COLS as usize - 1 {
                sum += grid[row][col + 1] as u32;
                n += 1;
            }
            next[row][col] = (sum / n).clamp(1, 9) as u8;
        }
    }
    // 川(6)を約3%のマスにランダム配置（スムージングでは川が出ないため別途追加）
    for row in 0..GRID_ROWS as usize {
        for col in 0..GRID_COLS as usize {
            if rng.gen_ratio(3, 100) {
                next[row][col] = 6;
            }
        }
    }
    next
}

/// レベルに対応する魔獣ID（`card_id`）
fn level_to_card_id(level: u8) -> u32 {
    match level {
        1 => 10, // ゴブリン
        2 => 11, // コボルド
        3 => 12, // オーク
        4 => 13, // スケルトン
        5 => 14, // トロール
        _ => 15, // ドレイク
    }
}

/// 領地レベルに対する連戦数（KC準拠: Lv4以上で連戦）
pub(crate) fn wave_count_for_level(level: u8) -> u32 {
    match level {
        1..=5 => 1,
        6 => 2,
        7..=8 => 2,
        _ => 3,
    }
}

/// 全マスを領地として生成。id は c_{col}_{row}。本拠地 (24,24) のみプレイヤー所有。地形はランダム。
/// レベルに応じた中立地の敵を生成（魔獣マスタ定義を使用）
pub(crate) fn generate_neutral_enemies(level: u8) -> (u32, Vec<u32>, Vec<String>) {
    let (card_id, count, mc_per_body): (u32, u32, u32) = match level {
        1 => (10, 1, 50),       // Lv1×1, 50
        2 => (11, 1, 250),      // Lv3相当×1, 250
        3 => (12, 2, 250),      // Lv5×2, 各250
        4 => (13, 2, 500),      // Lv10×2, 各500
        5 => (14, 3, 1500),     // Lv15×3, 各1500
        6 => (14, 3, 3500),     // Lv20×3, 各3500（トロール代表）
        7 => (35, 2, 6000),     // Lv30×2, 各6000
        8 => (43, 3, 8500),     // Lv55×3, 各8500（タイタン代表）
        9 => (40, 3, 9000),     // Lv70×3, 各9000（ニーズヘッグ代表）
        _ => (40, 3, 9000),
    };
    let card = get_card(card_id).unwrap_or_else(|| get_card(10).unwrap());
    let monster_counts = vec![mc_per_body; count as usize];
    let suffixes = ["A", "B", "C"];
    let names: Vec<String> = (0..count as usize)
        .map(|i| format!("{}{}", card.name, suffixes.get(i).unwrap_or(&"")))
        .collect();
    (count, monster_counts, names)
}

/// 遺跡はバックグラウンドタスクで動的にスポーンする（突発イベント）。
fn default_territories() -> Vec<Territory> {
    let level_grid = random_level_grid();
    let mut out = Vec::with_capacity(GRID_COLS as usize * GRID_ROWS as usize);
    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLS {
            let id = format!("c_{}_{}", col, row);
            let level = level_grid[row as usize][col as usize];
            let name = terrain_name(level).to_string();
            
            if col == HOME_COL && row == HOME_ROW {
                // プレイヤー本拠地（体数・魔獣数は初期所持魔獣と一致）
                let owned = default_owned_cards();
                let home_mc = initial_card_monster_counts_for_owned(&owned);
                let ntroops = owned.len() as u32;
                out.push(Territory {
                    id,
                    name,
                    level,
                    owner_id: Some(DEFAULT_PLAYER_ID.to_string()),
                    troops: ntroops,
                    body_monster_counts: Some(home_mc),
                    body_names: None,
                    ruin: None,
                    is_base: true,
                    durability: 0,
                    max_durability: 0,
                    tower_level: 0,
                });
            } else {
                // 中立マス: レベルに応じた敵を配置。耐久なし＝戦闘勝利で即占領（PvP拠点のみ耐久を使う）
                let (troops, body_monster_counts, body_names) = generate_neutral_enemies(level);
                out.push(Territory {
                    id,
                    name,
                    level,
                    owner_id: None,
                    troops,
                    body_monster_counts: Some(body_monster_counts),
                    body_names: Some(body_names),
                    ruin: None,
                    is_base: false,
                    durability: 0,
                    max_durability: 0,
                    tower_level: 0,
                });
            }
        }
    }
    out
}

pub use crate::cards::CardStats;

/// クライアントから送る行動。JSON の action は小文字スネーク（クライアントと一致）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum Action {
    #[serde(rename = "end_turn")]
    EndTurn,
    /// 自領地に増援。owner_id == "player" の領地のみ。
    #[serde(rename = "deploy")]
    Deploy {
        territory_id: String,
        count: u32,
        /// 援軍の体ごとのモンスター数。未指定時は各1として扱う。
        #[serde(default)]
        monsters_per_body: Option<Vec<u32>>,
        /// 援軍の体ごとの表示名。
        #[serde(default)]
        body_names: Option<Vec<String>>,
    },
    /// 自領地から他領地へ攻撃。from は自領、to は隣接想定（現状は検証なし）。
    #[serde(rename = "attack")]
    Attack {
        from_territory_id: String,
        to_territory_id: String,
        count: u32,
        /// 攻撃側の体ごとのモンスター数（先頭から順に敵1体目・2体目…と戦闘）。未指定時は各体を1として扱う。
        #[serde(default)]
        monsters_per_body: Option<Vec<u32>>,
        /// 攻撃側の体ごとの表示名（戦闘ログ用）。未指定時は「味方ユニットN」。
        #[serde(default)]
        body_names: Option<Vec<String>>,
        /// 攻撃するユニットの表示名（ログの「〇〇が△△を攻撃」の〇〇）。未指定時は領地名。
        #[serde(default)]
        unit_name: Option<String>,
        /// 攻撃側の体ごとのSPEED。未指定時は各5として扱う。
        #[serde(default)]
        speed_per_body: Option<Vec<u32>>,
        /// 攻撃側の体ごとのスキルデータ。
        #[serde(default)]
        skills_per_body: Option<Vec<SkillData>>,
        /// 攻撃側の体ごとの全ステータス。
        #[serde(default)]
        stats_per_body: Option<Vec<CardStats>>,
        /// 編成に対応する所持魔獣スロットのインデックス（スタミナ・XP用。クライアントが付与）
        #[serde(default)]
        owned_card_indices: Option<Vec<usize>>,
    },
    /// 占領済み領地に前線基地を建設（KC準拠: 前線を拡大し施設スロット追加）
    #[serde(rename = "build_base")]
    BuildBase {
        territory_id: String,
    },
    /// 魔獣合成（KC準拠: 素材魔獣を消費してベース魔獣のスキルLvアップ or スキル移植）
    #[serde(rename = "synthesize_card")]
    SynthesizeCard {
        base_card_index: usize,
        material_card_indices: Vec<usize>,
    },
    /// 所持魔獣スロットの魔獣を増産（食料消費・魔獣マスタの上限まで）
    #[serde(rename = "produce_monsters")]
    ProduceMonsters {
        card_index: usize,
        amount: u32,
    },
    /// 同盟結成（KC準拠）
    #[serde(rename = "create_alliance")]
    CreateAlliance {
        name: String,
    },
    /// 同盟参加
    #[serde(rename = "join_alliance")]
    JoinAlliance {
        alliance_id: String,
    },
    /// 同盟脱退
    #[serde(rename = "leave_alliance")]
    LeaveAlliance,
    /// フリマ出品
    #[serde(rename = "list_on_flea_market")]
    ListOnFleaMarket {
        item: MarketItemType,
        price: u64,
    },
    /// フリマ購入
    #[serde(rename = "buy_from_flea_market")]
    BuyFromFleaMarket {
        listing_id: String,
    },
    /// フリマ出品取消
    #[serde(rename = "cancel_flea_market_listing")]
    CancelFleaMarketListing {
        listing_id: String,
    },
    /// 探索を開始（占領済み領地・同時派遣数は exploration_level まで）
    #[serde(rename = "start_exploration")]
    StartExploration {
        territory_id: String,
        #[serde(default)]
        card_indices: Vec<usize>,
    },
    /// 探索結果を回収
    #[serde(rename = "collect_exploration")]
    CollectExploration {
        mission_id: String,
    },
    /// 同盟へ資源寄付（同盟レベル・寄付累計が増加）
    #[serde(rename = "donate_alliance")]
    DonateAlliance {
        food: u64,
        wood: u64,
        stone: u64,
        iron: u64,
    },
}

/// 領地を ID で取得。インデックスを返す。
pub(crate) fn get_territory_index(territories: &[Territory], id: &str) -> Option<usize> {
    territories.iter().position(|t| t.id.as_str() == id)
}

pub(crate) fn is_home_territory(id: &str) -> bool {
    parse_territory_id(id).map(|(c, r)| c == HOME_COL && r == HOME_ROW).unwrap_or(false)
}

pub(crate) fn home_territory_id() -> String {
    format!("c_{}_{}", HOME_COL, HOME_ROW)
}

/// シングルプレイ: 旧セーブで `players` が空・または `owned_cards` だけトップレベルにある場合の補正用。
pub(crate) fn merge_legacy_into_working_players(
    legacy_owned_cards: &[u32],
    players: &mut HashMap<String, PlayerData>,
) {
    match players.entry(DEFAULT_PLAYER_ID.to_string()) {
        Entry::Vacant(v) => {
            let mut pd = PlayerData::new(DEFAULT_PLAYER_ID.to_string(), home_territory_id());
            if !legacy_owned_cards.is_empty() {
                pd.owned_cards = legacy_owned_cards.to_vec();
            }
            ensure_card_monster_counts(&mut pd);
            v.insert(pd);
        }
        Entry::Occupied(mut o) => {
            if o.get().owned_cards.is_empty() && !legacy_owned_cards.is_empty() {
                o.get_mut().owned_cards = legacy_owned_cards.to_vec();
            }
            ensure_card_monster_counts(o.get_mut());
        }
    }
}

/// 永続化ロード直後: `player` エントリとトップレベル `owned_cards` を揃える。
pub fn reconcile_singleplayer_after_load(state: &mut GameState) {
    merge_legacy_into_working_players(&state.owned_cards, &mut state.players);
    if let Some(p) = state.players.get(DEFAULT_PLAYER_ID) {
        if state.owned_cards.is_empty() && !p.owned_cards.is_empty() {
            state.owned_cards = p.owned_cards.clone();
        }
        state.card_monster_counts = p.card_monster_counts.clone();
        sync_home_territory_body_counts_from_player(&mut state.territories, p);
    }
}

/// c_{col}_{row} 形式の ID から (col, row) を取得。
pub(crate) fn parse_territory_id(id: &str) -> Option<(u8, u8)> {
    let id = id.as_bytes();
    if id.len() < 5 || &id[0..2] != b"c_" {
        return None;
    }
    let rest = std::str::from_utf8(&id[2..]).ok()?;
    let (col_str, row_str) = rest.split_once('_')?;
    let col: u8 = col_str.parse().ok()?;
    let row: u8 = row_str.parse().ok()?;
    if col < GRID_COLS && row < GRID_ROWS {
        Some((col, row))
    } else {
        None
    }
}

/// 攻撃時に「拠点を前線とみなす」オーナーID（自プレイヤー・援軍先・同盟メンバー）
pub(crate) fn attack_base_owner_ids(state: &GameState, acting_player_id: &str) -> Vec<String> {
    let mut ids: Vec<String> = vec![acting_player_id.to_string()];
    for oid in &state.deployable_owner_ids {
        if !ids.iter().any(|x| x == oid) {
            ids.push(oid.clone());
        }
    }
    for a in &state.alliances {
        if a.member_ids.iter().any(|m| m == acting_player_id) {
            for m in &a.member_ids {
                if !ids.iter().any(|x| x == m) {
                    ids.push(m.clone());
                }
            }
        }
    }
    ids
}

/// 4方向で隣接するマス同士か
pub(crate) fn territories_are_adjacent(a_id: &str, b_id: &str) -> bool {
    let (ac, ar) = match parse_territory_id(a_id) {
        Some(p) => p,
        None => return false,
    };
    let (bc, br) = match parse_territory_id(b_id) {
        Some(p) => p,
        None => return false,
    };
    let dc = (ac as i16 - bc as i16).abs();
    let dr = (ar as i16 - br as i16).abs();
    dc + dr == 1
}

/// 攻撃可能な目標か。**攻撃側陣営が所有する領地**（本拠・占領地・前線基地を問わない）のいずれかに 4 方向で隣接していること。
/// （`from` が隣接かは別途 `territories_are_adjacent` で検証。クライアント `isAttackable` と一致させる。）
pub(crate) fn is_attackable_target(
    territories: &[Territory],
    target_id: &str,
    base_owner_ids: &[String],
) -> bool {
    let (col, row) = match parse_territory_id(target_id) {
        Some(p) => p,
        None => return false,
    };
    let col = col as i16;
    let row = row as i16;
    let owned_positions: std::collections::HashSet<(u8, u8)> = territories
        .iter()
        .filter(|t| {
            t.owner_id
                .as_ref()
                .map(|o| base_owner_ids.iter().any(|id| id == o))
                .unwrap_or(false)
        })
        .filter_map(|t| parse_territory_id(&t.id))
        .collect();
    let neighbors = [
        (col - 1, row),
        (col + 1, row),
        (col, row - 1),
        (col, row + 1),
    ];
    for (c, r) in neighbors {
        if c < 0 || c >= GRID_COLS as i16 || r < 0 || r >= GRID_ROWS as i16 {
            continue;
        }
        let (cu, ru) = (c as u8, r as u8);
        if owned_positions.contains(&(cu, ru)) {
            return true;
        }
    }
    false
}

/// 領地レベルから資源生産量を算出（KC準拠: 高レベル領地ほど多く生産）
fn resource_rate_for_level(level: u8) -> (u64, u64, u64, u64) {
    match level {
        1 => (10, 10, 5, 3),
        2 => (15, 15, 8, 5),
        3 => (20, 20, 12, 8),
        4 => (25, 25, 18, 12),
        5 => (30, 30, 25, 18),
        _ => (35, 35, 30, 20),
    }
}

/// 時間ベース資源生産: 占領した領地の数・レベルに応じて資源が増加
pub fn tick_resources(state: &mut GameState) {
    let now = default_now_ms();

    let player_territories: Vec<u8> = state.territories.iter()
        .filter(|t| t.owner_id.as_deref() == Some(DEFAULT_PLAYER_ID))
        .map(|t| t.level)
        .collect();

    if let Some(player) = state.players.get_mut(DEFAULT_PLAYER_ID) {
        let elapsed_ms = now.saturating_sub(player.last_resource_tick);
        if elapsed_ms < 60_000 { return; }

        let minutes = elapsed_ms / 60_000;
        let (mut food_rate, mut wood_rate, mut stone_rate, mut iron_rate) = (5u64, 5u64, 3u64, 2u64);
        for &level in &player_territories {
            let (f, w, s, i) = resource_rate_for_level(level);
            food_rate += f;
            wood_rate += w;
            stone_rate += s;
            iron_rate += i;
        }

        let bonuses = crate::facilities::calculate_facility_bonuses(&player.facilities);
        let res_cap = 10_000u64.saturating_add(bonuses.storage_capacity as u64 * 150);

        player.resources.food = (player.resources.food + food_rate * minutes).min(res_cap);
        player.resources.wood = (player.resources.wood + wood_rate * minutes).min(res_cap);
        player.resources.stone = (player.resources.stone + stone_rate * minutes).min(res_cap);
        player.resources.iron = (player.resources.iron + iron_rate * minutes).min(res_cap);
        player.last_resource_tick = now;

        state.inventory = player.inventory.clone();
        state.resources = player.resources.clone();
    }
}

/// 純粋関数: 行動を適用した新状態を返す。戦闘はすべてここで処理し、ログに記録する。
/// `dev_auto_win`: true のとき攻撃側を10倍有利に（戦闘計算・ログは通常表示、ローカル開発用）。
pub fn apply_action(state: &GameState, action: &Action, dev_auto_win: bool) -> GameState {
    crate::model_actions::apply_action(state, action, dev_auto_win)
}

/// 期限切れの遺跡をクリーンアップ
/// 遺跡が期限切れになったら、遺跡を削除して元の中立マスに戻す
pub fn cleanup_expired_ruins(state: &mut GameState) -> bool {
    crate::model_ruins::cleanup_expired_ruins(state)
}

/// 遺跡をランダムな中立マスにスポーンさせる
/// 成功したらtrue、スポーン先がなければfalse
pub fn spawn_random_ruin(state: &mut GameState) -> bool {
    crate::model_ruins::spawn_random_ruin(state)
}

/// 現在の遺跡数をカウント
pub fn count_ruins(state: &GameState) -> usize {
    crate::model_ruins::count_ruins(state)
}

/// シーズン終了チェック: 期間を過ぎたらマップ・領地をリセットして新シーズン開始
pub fn check_season_end(state: &mut GameState) -> bool {
    let now = default_now_ms();
    let elapsed = now.saturating_sub(state.season.started_at);
    if elapsed < state.season.duration_ms {
        return false;
    }

    let old_season = state.season.season_number;
    state.season = SeasonInfo {
        season_number: old_season + 1,
        started_at: now,
        duration_ms: state.season.duration_ms,
    };

    state.territories = default_territories();

    for player in state.players.values_mut() {
        player.resources = Resources::default();
        player.explorations.clear();
    }

    state.alliances.clear();

    push_log(&mut state.log, format!(
        "シーズン{}が終了しました！シーズン{}が開始されます。",
        old_season, old_season + 1
    ));
    true
}
