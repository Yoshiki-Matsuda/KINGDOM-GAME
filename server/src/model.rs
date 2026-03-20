//! ゲーム状態・行動の共通データモデル（フロントと同一構造で JSON 化）
//!
//! 設計: サーバー権威・データ駆動。状態更新は純粋関数 `apply_action` のみ。
//! 最終形 PvPvE を想定し、owner_id でプレイヤー／中立を区別。

use std::collections::HashMap;
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
    /// 所持カード（カードID）
    #[serde(default)]
    pub owned_cards: Vec<u32>,
    /// 援軍を送れる他プレイヤーのID（クラン・配下など）
    #[serde(default)]
    pub allied_player_ids: Vec<String>,
}

impl PlayerData {
    pub fn new(player_id: String, home_territory_id: String) -> Self {
        Self {
            player_id,
            home_territory_id,
            inventory: default_dev_inventory(),
            facilities: vec![],
            owned_cards: default_owned_cards(),
            allied_player_ids: vec![],
        }
    }
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
    /// 配置エナジー（体数）
    pub troops: u32,
    /// 体ごとのエナジー。len() == troops のときのみ有効。未設定時は戦闘で各体を1として扱う。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_energies: Option<Vec<u32>>,
    /// 体ごとの表示名（戦闘ログ用）。len() == troops のときのみ有効。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_names: Option<Vec<String>>,
    /// 遺跡情報（存在する場合）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ruin: Option<RuinInfo>,
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
    /// プレイヤーの所持カード（シングルプレイ用）
    #[serde(default)]
    pub owned_cards: Vec<u32>,
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
) -> GameState {
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
    }
}

pub(crate) const MAX_LOG_LINES: usize = 200;

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
            // 後方互換性のため直接フィールドにもコピー
            deployable_owner_ids: vec![],
            inventory: default_player.inventory,
            owned_cards: default_player.owned_cards,
            facilities: default_player.facilities,
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

/// 初期所持カード（北欧神話キャラ）
fn default_owned_cards() -> Vec<u32> {
    // カードID 0〜9: 初期カード（各1枚）
    vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
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
    log.push(line);
    if log.len() > MAX_LOG_LINES {
        log.drain(0..log.len() - MAX_LOG_LINES);
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
        _ => "平原",
    }
}

/// ランダムな地形レベル（1〜6）のグリッドを生成。1回スムージングしてまとまりを出し、川(6)を別途配置。
fn random_level_grid() -> Vec<Vec<u8>> {
    let mut rng = rand::thread_rng();
    let grid: Vec<Vec<u8>> = (0..GRID_ROWS as usize)
        .map(|_| (0..GRID_COLS as usize).map(|_| rng.gen_range(1..=5)).collect())
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
            next[row][col] = (sum / n).clamp(1, 5) as u8;
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

/// レベルに対応するカードID
fn level_to_card_id(level: u8) -> u32 {
    match level {
        1 => 10, // スライム
        2 => 11, // ゴブリン
        3 => 12, // オーク
        4 => 13, // 骸骨戦士
        5 => 14, // オーガ
        _ => 15, // ワイバーン
    }
}

/// 全マスを領地として生成。id は c_{col}_{row}。本拠地 (24,24) のみプレイヤー所有。地形はランダム。
/// レベルに応じた中立地の敵を生成（カード定義を使用）
pub(crate) fn generate_neutral_enemies(level: u8) -> (u32, Vec<u32>, Vec<String>) {
    let card_id = level_to_card_id(level);
    let card = get_card(card_id).unwrap_or_else(|| get_card(10).unwrap()); // fallback to slime
    
    let count = 3u32;
    let energies = vec![card.stats.energy; count as usize];
    let names = vec![
        format!("{}A", card.name),
        format!("{}B", card.name),
        format!("{}C", card.name),
    ];
    (count, energies, names)
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
                // プレイヤー本拠地
                out.push(Territory {
                    id,
                    name,
                    level,
                    owner_id: Some(DEFAULT_PLAYER_ID.to_string()),
                    troops: 10,
                    body_energies: Some(vec![10u32; 10]),
                    body_names: None,
                    ruin: None,
                });
            } else {
                // 中立マス: レベルに応じた敵を配置
                let (troops, body_energies, body_names) = generate_neutral_enemies(level);
                out.push(Territory {
                    id,
                    name,
                    level,
                    owner_id: None,
                    troops,
                    body_energies: Some(body_energies),
                    body_names: Some(body_names),
                    ruin: None,
                });
            }
        }
    }
    out
}

/// カードの全ステータス
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CardStats {
    pub energy: u32,
    pub speed: u32,
    pub attack: u32,
    pub magic: u32,
    pub defense: u32,
    pub magic_defense: u32,
}

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
        /// 援軍の体ごとのエナジー。未指定時は各1として扱う。
        #[serde(default)]
        energy_per_body: Option<Vec<u32>>,
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
        /// 攻撃側の体ごとのエナジー（先頭から順に敵1体目・2体目…と戦闘）。未指定時は各体を1として扱う。
        #[serde(default)]
        energy_per_body: Option<Vec<u32>>,
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

/// 攻撃可能な目標か。本拠地または自領に隣接しているマスのみ（4方向）。
pub(crate) fn is_attackable_target(territories: &[Territory], target_id: &str) -> bool {
    let (col, row) = match parse_territory_id(target_id) {
        Some(p) => p,
        None => return false,
    };
    let col = col as i16;
    let row = row as i16;
    let player_positions: std::collections::HashSet<(u8, u8)> = territories
        .iter()
        .filter(|t| t.owner_id.as_deref() == Some("player"))
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
        if (cu == HOME_COL && ru == HOME_ROW) || player_positions.contains(&(cu, ru)) {
            return true;
        }
    }
    false
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
