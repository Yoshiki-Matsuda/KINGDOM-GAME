//! ゲーム状態・行動の共通データモデル（フロントと同一構造で JSON 化）
//!
//! 設計: サーバー権威・データ駆動。状態更新は純粋関数 `apply_action` のみ。
//! 最終形 PvPvE を想定し、owner_id でプレイヤー／中立を区別。

use std::collections::HashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::skills::{
    apply_attack_skills, apply_battle_start_skills, check_death_skills,
    apply_effect_to_character, CombatCharacter, SkillData,
};
use crate::items::InventoryItem;
use crate::ruins::{RuinInfo, generate_ruin};
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
fn parse_territory_coords(id: &str) -> Option<(i32, i32)> {
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
    /// プレイヤーデータを取得（存在しなければNone）
    pub fn get_player(&self, player_id: &str) -> Option<&PlayerData> {
        self.players.get(player_id)
    }
    
    /// プレイヤーデータを可変で取得
    pub fn get_player_mut(&mut self, player_id: &str) -> Option<&mut PlayerData> {
        self.players.get_mut(player_id)
    }
    
    /// プレイヤーのインベントリを取得（後方互換: playersにいなければ直接フィールドを返す）
    pub fn get_inventory(&self, player_id: &str) -> &Vec<InventoryItem> {
        self.players.get(player_id)
            .map(|p| &p.inventory)
            .unwrap_or(&self.inventory)
    }
    
    /// プレイヤーの施設を取得
    pub fn get_facilities(&self, player_id: &str) -> &Vec<BuiltFacility> {
        self.players.get(player_id)
            .map(|p| &p.facilities)
            .unwrap_or(&self.facilities)
    }
    
    /// プレイヤーの所持カードを取得
    pub fn get_owned_cards(&self, player_id: &str) -> &Vec<u32> {
        self.players.get(player_id)
            .map(|p| &p.owned_cards)
            .unwrap_or(&self.owned_cards)
    }
    
    /// プレイヤーが援軍を送れるowner_idリストを取得
    pub fn get_deployable_owner_ids(&self, player_id: &str) -> Vec<String> {
        let mut ids = vec![player_id.to_string()];
        if let Some(player) = self.players.get(player_id) {
            ids.extend(player.allied_player_ids.clone());
        } else {
            ids.extend(self.deployable_owner_ids.clone());
        }
        ids
    }
}

const MAX_LOG_LINES: usize = 200;

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
            log: vec!["ゲームを開始しました。".to_string()],
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
fn can_receive_reinforcement(territories: &[Territory], deployable_owner_ids: &[String], territory_id: &str) -> bool {
    let Some(idx) = get_territory_index(territories, territory_id) else {
        return false;
    };
    let owner = match &territories[idx].owner_id {
        Some(id) => id.as_str(),
        None => return false,
    };
    owner == "player" || deployable_owner_ids.iter().any(|id| id.as_str() == owner)
}

fn territory_name<'a>(territories: &'a [Territory], id: &'a str) -> &'a str {
    territories.iter().find(|t| t.id.as_str() == id).map(|t| t.name.as_str()).unwrap_or(id)
}

fn push_log(log: &mut Vec<String>, line: String) {
    log.push(line);
    if log.len() > MAX_LOG_LINES {
        log.drain(0..log.len() - MAX_LOG_LINES);
    }
}

/// グリッドサイズ（クライアントの GRID_COLS / GRID_ROWS と一致）
const GRID_COLS: u8 = 48;
const GRID_ROWS: u8 = 48;

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
fn generate_neutral_enemies(level: u8) -> (u32, Vec<u32>, Vec<String>) {
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
fn get_territory_index(territories: &[Territory], id: &str) -> Option<usize> {
    territories.iter().position(|t| t.id.as_str() == id)
}

fn is_home_territory(id: &str) -> bool {
    parse_territory_id(id).map(|(c, r)| c == HOME_COL && r == HOME_ROW).unwrap_or(false)
}

fn home_territory_id() -> String {
    format!("c_{}_{}", HOME_COL, HOME_ROW)
}

/// c_{col}_{row} 形式の ID から (col, row) を取得。
fn parse_territory_id(id: &str) -> Option<(u8, u8)> {
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
fn is_attackable_target(territories: &[Territory], target_id: &str) -> bool {
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
    let mut log = state.log.clone();
    match action {
        Action::EndTurn => {
            push_log(&mut log, format!("--- ターン {} 終了 ---", state.turn));
            GameState {
                turn: state.turn + 1,
                phase: state.phase.clone(),
                territories: state.territories.clone(),
                log,
                players: state.players.clone(),
                deployable_owner_ids: state.deployable_owner_ids.clone(),
                inventory: state.inventory.clone(),
                facilities: state.facilities.clone(),
                owned_cards: state.owned_cards.clone(),
            }
        }
        Action::Deploy { territory_id, count, energy_per_body, body_names: deploy_body_names } => {
            if is_home_territory(territory_id) {
                return state.clone();
            }
            let mut territories = state.territories.clone();
            let Some(idx) = get_territory_index(&territories, territory_id) else {
                return state.clone();
            };
            if !can_receive_reinforcement(&territories, &state.deployable_owner_ids, territory_id) {
                return state.clone();
            }
            if *count == 0 || *count > 100 {
                return state.clone();
            }
            let name = territory_name(&territories, territory_id).to_string();
            territories[idx].troops += count;

            // 援軍のエナジーを追加（クライアントから送られた値を使用、未指定時は1）
            let reinforcement_energies: Vec<u32> = energy_per_body
                .clone()
                .filter(|v| v.len() == *count as usize)
                .unwrap_or_else(|| vec![1u32; *count as usize]);
            if let Some(ref mut v) = territories[idx].body_energies {
                v.extend(reinforcement_energies.iter());
            } else {
                // 既存の体数分は1で埋め、援軍のエナジーを追加
                let existing = territories[idx].troops.saturating_sub(*count) as usize;
                let mut new_energies = vec![1u32; existing];
                new_energies.extend(reinforcement_energies.iter());
                territories[idx].body_energies = Some(new_energies);
            }

            // 援軍の表示名を追加
            let reinforcement_names: Vec<String> = deploy_body_names
                .clone()
                .filter(|v| v.len() == *count as usize)
                .unwrap_or_else(|| (1..=*count as usize).map(|i| format!("援軍{}", i)).collect());
            if let Some(ref mut v) = territories[idx].body_names {
                v.extend(reinforcement_names.iter().cloned());
            } else {
                let existing = territories[idx].troops.saturating_sub(*count) as usize;
                let mut new_names: Vec<String> = (1..=existing).map(|i| format!("守備{}", i)).collect();
                new_names.extend(reinforcement_names.iter().cloned());
                territories[idx].body_names = Some(new_names);
            }

            // ログに合計エナジーを表示
            let total_energy: u32 = reinforcement_energies.iter().sum();
            push_log(
                &mut log,
                format!("ターン{}: {}にエナジー{}（合計{}）を増援した。", state.turn, name, count, total_energy),
            );
            GameState {
                turn: state.turn,
                phase: state.phase.clone(),
                territories,
                log,
                players: state.players.clone(),
                deployable_owner_ids: state.deployable_owner_ids.clone(),
                inventory: state.inventory.clone(),
                facilities: state.facilities.clone(),
                owned_cards: state.owned_cards.clone(),
            }
        }
        Action::Attack {
            from_territory_id,
            to_territory_id,
            count,
            energy_per_body,
            body_names: our_body_names,
            unit_name: attack_unit_name,
            speed_per_body,
            skills_per_body,
            stats_per_body,
        } => {
            let mut territories = state.territories.clone();
            let from_idx = match get_territory_index(&territories, from_territory_id) {
                Some(i) => i,
                None => return state.clone(),
            };
            let to_idx = match get_territory_index(&territories, to_territory_id) {
                Some(i) => i,
                None => return state.clone(),
            };
            let home_id = home_territory_id();
            let home_idx = match get_territory_index(&territories, &home_id) {
                Some(i) => i,
                None => return state.clone(),
            };
            if from_idx == to_idx {
                return state.clone();
            }
            if is_home_territory(to_territory_id) {
                return state.clone();
            }
            if territories[from_idx].owner_id.as_deref() != Some("player") {
                return state.clone();
            }
            if territories[home_idx].troops < *count || *count == 0 {
                return state.clone();
            }
            if !is_attackable_target(&territories, to_territory_id) {
                return state.clone();
            }

            let from_name = territory_name(&territories, from_territory_id).to_string();
            let to_name = territory_name(&territories, to_territory_id).to_string();
            let to_troops = territories[to_idx].troops;

            // 施設ボーナスを計算
            let facility_bonuses = crate::facilities::calculate_facility_bonuses(&state.facilities);

            // 味方: 攻撃に出す体ごとのエナジー・SPEED・スキル
            let our_energies: Vec<u32> = energy_per_body
                .clone()
                .filter(|v| v.len() == *count as usize)
                .unwrap_or_else(|| vec![1u32; *count as usize]);
            let our_speeds: Vec<u32> = speed_per_body
                .clone()
                .filter(|v| v.len() == *count as usize)
                .unwrap_or_else(|| vec![5u32; *count as usize]);
            let our_names: Vec<String> = our_body_names
                .clone()
                .filter(|v| v.len() == *count as usize)
                .unwrap_or_else(|| (1..=*count as usize).map(|i| format!("味方ユニット{}", i)).collect());
            let our_skills: Vec<SkillData> = skills_per_body
                .clone()
                .filter(|v| v.len() == *count as usize)
                .unwrap_or_else(|| vec![SkillData::default(); *count as usize]);

            // stats_per_bodyから各ステータスを取得（存在しない場合はデフォルト値）
            let our_stats: Vec<CardStats> = stats_per_body
                .clone()
                .filter(|v| v.len() == *count as usize)
                .unwrap_or_else(|| vec![CardStats::default(); *count as usize]);

            // 味方キャラクターをCombatCharacterに変換（施設ボーナス適用）
            let mut our_chars: Vec<CombatCharacter> = our_names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let stats = our_stats.get(i).cloned().unwrap_or_default();
                    let base_energy = if stats.energy > 0 { stats.energy } else { *our_energies.get(i).unwrap_or(&1) };
                    let base_speed = if stats.speed > 0 { stats.speed } else { *our_speeds.get(i).unwrap_or(&5) };
                    let attack = if stats.attack > 0 { stats.attack } else { 5 };
                    let magic = if stats.magic > 0 { stats.magic } else { 5 };
                    let defense = if stats.defense > 0 { stats.defense } else { 3 };
                    let magic_defense = if stats.magic_defense > 0 { stats.magic_defense } else { 3 };
                    let skills = our_skills.get(i).cloned().unwrap_or_default();
                    
                    // 施設ボーナスを適用
                    let boosted_energy = crate::facilities::apply_energy_bonus(base_energy, &facility_bonuses);
                    let boosted_speed = base_speed + facility_bonuses.speed_bonus;
                    
                    CombatCharacter::with_stats(
                        i,
                        name.clone(),
                        boosted_energy,
                        boosted_speed,
                        attack,
                        magic,
                        defense,
                        magic_defense,
                        skills,
                    )
                })
                .collect();

            // 敵: 領地の体ごとのエナジーと表示名（スキルなし）
            let enemy_energies: Vec<u32> = territories[to_idx]
                .body_energies
                .clone()
                .filter(|v| v.len() == to_troops as usize)
                .unwrap_or_else(|| vec![1u32; to_troops as usize]);
            let enemy_names: Vec<String> = territories[to_idx]
                .body_names
                .clone()
                .filter(|v| v.len() == to_troops as usize)
                .unwrap_or_else(|| (1..=to_troops as usize).map(|i| format!("敵ユニット{}", i)).collect());

            let mut enemy_chars: Vec<CombatCharacter> = enemy_names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let energy = *enemy_energies.get(i).unwrap_or(&1);
                    // 名前からサフィックス（A/B/C）を除去してカード名を取得
                    let card_name = name.trim_end_matches(|c| c == 'A' || c == 'B' || c == 'C');
                    if let Some(card) = crate::cards::get_card_by_name(card_name) {
                        // カード定義があればフルステータスを使用
                        CombatCharacter::with_stats(
                            i + 100,
                            name.clone(),
                            energy,
                            card.stats.speed,
                            card.stats.attack,
                            card.stats.magic,
                            card.stats.defense,
                            card.stats.magic_defense,
                            SkillData::default(),
                        )
                    } else {
                        // カード定義がなければデフォルト
                        CombatCharacter::new(i + 100, name.clone(), energy, 5, SkillData::default())
                    }
                })
                .collect();

            // 開発モード: 味方各体のエナジーを敵最大+1にして確実に勝てるようにする
            if dev_auto_win {
                let max_enemy = enemy_chars.iter().map(|c| c.base_energy).max().unwrap_or(1);
                for c in our_chars.iter_mut() {
                    c.current_energy = (max_enemy + 1) as f32;
                }
            }

            // 戦闘開始の概要（座標を含む）
            let attacker_label = attack_unit_name.as_deref().unwrap_or(from_name.as_str());
            let coords_str = parse_territory_coords(to_territory_id)
                .map(|(col, row)| format!("<{},{}>", col, row))
                .unwrap_or_default();
            push_log(
                &mut log,
                format!("【{}{}侵攻戦】{}が{}へ侵攻開始", to_name, coords_str, attacker_label, to_name),
            );

            // === スキルフェーズ: 戦闘開始時パッシブスキル発動 ===
            push_log(&mut log, "--- スキル発動フェーズ ---".to_string());
            apply_battle_start_skills(&mut our_chars, &mut log);

            // === 戦闘フェーズ ===
            push_log(&mut log, "--- 戦闘フェーズ ---".to_string());

            let mut our_idx = 0usize;
            let mut enemy_idx = 0usize;

            while our_idx < our_chars.len() && enemy_idx < enemy_chars.len() {
                // 生存チェック（インデックスでアクセスして借用を避ける）
                if !our_chars[our_idx].is_alive {
                    our_idx += 1;
                    continue;
                }
                if !enemy_chars[enemy_idx].is_alive {
                    enemy_idx += 1;
                    continue;
                }

                // 行動不能チェック（凍結・気絶）
                if our_chars[our_idx].is_disabled() {
                    push_log(&mut log, format!("{}は行動不能！", our_chars[our_idx].name));
                    our_chars[our_idx].process_turn_effects(&mut log);
                    our_idx += 1;
                    continue;
                }

                // アクティブスキル発動（沈黙チェック）
                let attack_mods = if our_chars[our_idx].is_silenced() {
                    push_log(&mut log, format!("{}は沈黙中でスキル使用不可！", our_chars[our_idx].name));
                    crate::skills::AttackModifiers::new()
                } else {
                    apply_attack_skills(&mut our_chars[our_idx], &mut log)
                };

                // 回避判定
                let evasion_rate = enemy_chars[enemy_idx].get_evasion_rate();
                if evasion_rate > 0.0 && rand::random::<f32>() < evasion_rate {
                    push_log(&mut log, format!("{}の攻撃を{}が回避！", our_chars[our_idx].name, enemy_chars[enemy_idx].name));
                    our_idx += 1;
                    continue;
                }

                // 無敵判定
                if enemy_chars[enemy_idx].consume_invincible() {
                    push_log(&mut log, format!("{}は無敵で攻撃を無効化！", enemy_chars[enemy_idx].name));
                    our_idx += 1;
                    continue;
                }

                // ダメージ計算（エナジーベース + スキル補正）
                let base_damage = our_chars[our_idx].current_energy * our_chars[our_idx].damage_multiplier;
                let mut total_damage = (base_damage * attack_mods.damage_multiplier + attack_mods.damage_add).max(0.0);
                
                // 固定ダメージ追加
                total_damage += attack_mods.true_damage;
                
                // 割合ダメージ追加
                if attack_mods.percent_damage > 0.0 {
                    let percent_dmg = enemy_chars[enemy_idx].current_energy * attack_mods.percent_damage;
                    total_damage += percent_dmg;
                    push_log(&mut log, format!("割合ダメージ+{:.0}", percent_dmg));
                }
                
                // 脆弱・マークによる追加ダメージ
                let vulnerability = enemy_chars[enemy_idx].get_vulnerability();
                let mark_damage = enemy_chars[enemy_idx].get_mark_damage();
                total_damage = total_damage * (1.0 + vulnerability) + mark_damage;

                let enemy_defense = if attack_mods.ignore_defense {
                    0.0
                } else {
                    enemy_chars[enemy_idx].current_energy * (1.0 - enemy_chars[enemy_idx].damage_reduction)
                };

                let our_name = our_chars[our_idx].name.clone();
                let enemy_name = enemy_chars[enemy_idx].name.clone();

                push_log(
                    &mut log,
                    format!(
                        "{}が{}に攻撃！（攻撃力{:.0} vs 防御力{:.0}）",
                        our_name, enemy_name, total_damage, enemy_defense
                    ),
                );

                // 処刑判定（HPが閾値以下なら即死）
                let hp_ratio = enemy_chars[enemy_idx].current_energy / enemy_chars[enemy_idx].base_energy as f32;
                if attack_mods.execute_threshold > 0.0 && hp_ratio <= attack_mods.execute_threshold {
                    enemy_chars[enemy_idx].is_alive = false;
                    enemy_chars[enemy_idx].current_energy = 0.0;
                    push_log(&mut log, format!("処刑発動！{}を即死させた！", enemy_name));
                    enemy_idx += 1;
                    our_chars[our_idx].process_turn_effects(&mut log);
                    continue;
                }

                // AOE（全体攻撃）ダメージがある場合
                if attack_mods.aoe_damage > 0.0 {
                    push_log(&mut log, format!("全体攻撃で敵全員に{:.0}ダメージ！", attack_mods.aoe_damage));
                    for ec in enemy_chars.iter_mut() {
                        if ec.is_alive {
                            ec.current_energy -= attack_mods.aoe_damage;
                            if ec.current_energy <= 0.0 {
                                ec.is_alive = false;
                                push_log(&mut log, format!("{}が全体攻撃で撃破されました。", ec.name));
                            }
                        }
                    }
                }

                // 状態異常付与
                for status_effect in &attack_mods.status_effects {
                    apply_effect_to_character(status_effect, &mut enemy_chars[enemy_idx], &mut log);
                }

                // 自身への効果
                for self_effect in &attack_mods.self_effects {
                    apply_effect_to_character(self_effect, &mut our_chars[our_idx], &mut log);
                }

                // 味方への効果
                for ally_effect in &attack_mods.ally_effects {
                    for ally in our_chars.iter_mut() {
                        if ally.is_alive {
                            apply_effect_to_character(ally_effect, ally, &mut log);
                        }
                    }
                }

                // 回復効果（HP最低の味方へ）
                for heal_effect in &attack_mods.heal_effects {
                    if let Some(lowest) = our_chars.iter_mut().filter(|c| c.is_alive).min_by(|a, b| {
                        a.current_energy.partial_cmp(&b.current_energy).unwrap_or(std::cmp::Ordering::Equal)
                    }) {
                        apply_effect_to_character(heal_effect, lowest, &mut log);
                    }
                }

                // 通常戦闘判定
                if total_damage > enemy_defense {
                    // シールドでダメージ吸収
                    let damage_after_shield = enemy_chars[enemy_idx].absorb_damage_with_shield(total_damage - enemy_defense);
                    
                    if damage_after_shield > 0.0 || total_damage > enemy_defense {
                        enemy_chars[enemy_idx].is_alive = false;
                        push_log(&mut log, format!("{}が{}を撃破しました。", our_name, enemy_name));

                        // エナジー奪取
                        if attack_mods.energy_steal > 0.0 {
                            our_chars[our_idx].current_energy += attack_mods.energy_steal;
                            push_log(&mut log, format!("{}が{:.0}エナジーを奪取！", our_name, attack_mods.energy_steal));
                        }

                        // 吸収効果
                        if attack_mods.absorb_rate > 0.0 {
                            let absorb = enemy_defense * attack_mods.absorb_rate;
                            our_chars[our_idx].current_energy += absorb;
                            push_log(&mut log, format!("{}が{:.0}エナジーを吸収！", our_name, absorb));
                        }

                        // 追加攻撃
                        if attack_mods.extra_attacks > 0 {
                            our_chars[our_idx].extra_attacks += attack_mods.extra_attacks;
                            push_log(&mut log, format!("{}が追加攻撃権を得た！", our_name));
                        }

                        enemy_idx += 1;
                    } else {
                        push_log(&mut log, format!("{}のシールドがダメージを吸収！", enemy_name));
                    }
                } else if total_damage < enemy_defense {
                    // 反射ダメージ
                    let reflect_rate = enemy_chars[enemy_idx].get_reflect_rate();
                    if reflect_rate > 0.0 {
                        let reflect_damage = total_damage * reflect_rate;
                        our_chars[our_idx].current_energy -= reflect_damage;
                        push_log(&mut log, format!("{}の反射で{:.0}ダメージ！", enemy_name, reflect_damage));
                    }

                    our_chars[our_idx].is_alive = false;
                    // 復活スキルチェック
                    if !check_death_skills(&mut our_chars[our_idx], &mut log) {
                        push_log(&mut log, format!("{}が{}に撃破されました。", our_name, enemy_name));
                    }

                    // 反撃判定
                    let counter_rate = enemy_chars[enemy_idx].get_counter_rate();
                    if counter_rate > 0.0 && rand::random::<f32>() < counter_rate {
                        push_log(&mut log, format!("{}の反撃！", enemy_name));
                    }

                    our_idx += 1;
                } else {
                    // 相打ち
                    our_chars[our_idx].is_alive = false;
                    enemy_chars[enemy_idx].is_alive = false;
                    let our_revived = check_death_skills(&mut our_chars[our_idx], &mut log);
                    if !our_revived {
                        push_log(&mut log, format!("相打ち。{}と{}が撃破されました。", our_name, enemy_name));
                    } else {
                        push_log(&mut log, format!("{}が撃破されました。", enemy_name));
                    }
                    our_idx += 1;
                    enemy_idx += 1;
                }

                // ターン終了処理（状態異常ダメージなど）
                our_chars[our_idx.saturating_sub(1)].process_turn_effects(&mut log);

                // 追加攻撃の処理
                if our_idx > 0 && our_chars[our_idx - 1].extra_attacks > 0 && our_chars[our_idx - 1].is_alive {
                    our_chars[our_idx - 1].extra_attacks -= 1;
                    our_idx -= 1;
                }
            }

            // 生存者カウント
            let surviving_allies: Vec<&CombatCharacter> = our_chars.iter().filter(|c| c.is_alive).collect();
            let surviving_enemies: Vec<&CombatCharacter> = enemy_chars.iter().filter(|c| c.is_alive).collect();

            let conquered = surviving_enemies.is_empty();
            let mut new_inventory = state.inventory.clone();
            
            let mut new_owned_cards = state.owned_cards.clone();
            
            if conquered {
                territories[to_idx].owner_id = Some("player".to_string());
                let occupying: Vec<u32> = if surviving_allies.is_empty() {
                    vec![1u32]
                } else {
                    surviving_allies.iter().map(|c| c.effective_energy()).collect()
                };
                territories[to_idx].troops = occupying.len() as u32;
                territories[to_idx].body_energies = Some(occupying);
                territories[to_idx].body_names = None;
                push_log(&mut log, format!("{}を占領しました！", to_name));

                // ドロップ計算（施設ボーナス適用）
                let is_ruin = territories[to_idx].ruin.is_some();
                let enemy_type_refs: Vec<&str> = enemy_names.iter().map(|s| s.as_str()).collect();
                let drops = crate::items::calculate_drops(&enemy_type_refs, facility_bonuses.drop_rate);
                
                if !drops.is_empty() {
                    push_log(&mut log, "--- 戦利品 ---".to_string());
                    for drop in &drops {
                        push_log(&mut log, format!("{}x{} を入手！", drop.item_id, drop.count));
                    }
                    crate::items::add_items_to_inventory(&mut new_inventory, drops);
                }
                
                // カードドロップ判定（倒した敵ごと）
                let dropped_cards = calculate_card_drops(&enemy_names, facility_bonuses.drop_rate as f32);
                if !dropped_cards.is_empty() {
                    push_log(&mut log, "--- カード入手 ---".to_string());
                    for card_id in &dropped_cards {
                        if let Some(card) = crate::cards::get_card(*card_id) {
                            push_log(&mut log, format!("カード「{}」を入手！", card.name));
                            new_owned_cards.push(*card_id);
                        }
                    }
                }
                
                // 遺跡は占領後に消滅
                if is_ruin {
                    territories[to_idx].ruin = None;
                    push_log(&mut log, "遺跡を攻略しました！".to_string());
                }
            } else {
                let remaining_energies: Vec<u32> = surviving_enemies.iter().map(|c| c.effective_energy()).collect();
                let remaining_names: Vec<String> = surviving_enemies.iter().map(|c| c.name.clone()).collect();
                territories[to_idx].troops = remaining_energies.len() as u32;
                territories[to_idx].body_energies = Some(remaining_energies);
                territories[to_idx].body_names = if remaining_names.is_empty() { None } else { Some(remaining_names) };
                push_log(&mut log, format!("攻撃失敗。{}の防衛に成功。", to_name));
            }

            // playersも更新（マルチプレイ対応）
            let mut new_players = state.players.clone();
            if let Some(player) = new_players.get_mut(DEFAULT_PLAYER_ID) {
                player.inventory = new_inventory.clone();
                player.owned_cards = new_owned_cards.clone();
            }
            
            GameState {
                turn: state.turn,
                phase: state.phase.clone(),
                territories,
                log,
                players: new_players,
                deployable_owner_ids: state.deployable_owner_ids.clone(),
                inventory: new_inventory,
                facilities: state.facilities.clone(),
                owned_cards: new_owned_cards,
            }
        }
    }
}

/// 敵名からカードドロップを計算
fn calculate_card_drops(enemy_names: &[String], drop_rate_bonus: f32) -> Vec<u32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut dropped = Vec::new();
    
    for name in enemy_names {
        // 名前からサフィックス（A/B/C）を除去してカード名を取得
        let card_name = name.trim_end_matches(|c| c == 'A' || c == 'B' || c == 'C');
        if let Some(card) = crate::cards::get_card_by_name(card_name) {
            let base_chance = crate::cards::get_card_drop_chance(card.rarity);
            let actual_chance = base_chance * (1.0 + drop_rate_bonus);
            if rng.gen::<f32>() < actual_chance {
                dropped.push(card.id);
            }
        }
    }
    
    dropped
}

/// 期限切れの遺跡をクリーンアップ
/// 遺跡が期限切れになったら、遺跡を削除して元の中立マスに戻す
pub fn cleanup_expired_ruins(state: &mut GameState) -> bool {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let mut changed = false;
    for territory in &mut state.territories {
        if let Some(ref ruin) = territory.ruin {
            if let Some(expires_at) = ruin.expires_at {
                if now_ms >= expires_at {
                    // 遺跡が期限切れ → 削除して中立マスに戻す
                    territory.ruin = None;
                    territory.owner_id = None;
                    // レベルに応じた敵を配置
                    let (troops, body_energies, body_names) = generate_neutral_enemies(territory.level);
                    territory.troops = troops;
                    territory.body_energies = Some(body_energies);
                    territory.body_names = Some(body_names);
                    changed = true;
                }
            }
        }
    }
    changed
}

/// 遺跡をランダムな中立マスにスポーンさせる
/// 成功したらtrue、スポーン先がなければfalse
pub fn spawn_random_ruin(state: &mut GameState) -> bool {
    use rand::seq::SliceRandom;
    
    // 遺跡がなく、プレイヤー所有でない中立マスを候補に
    let candidates: Vec<usize> = state.territories
        .iter()
        .enumerate()
        .filter(|(_, t)| t.owner_id.is_none() && t.ruin.is_none())
        .map(|(i, _)| i)
        .collect();
    
    if candidates.is_empty() {
        return false;
    }
    
    let mut rng = rand::thread_rng();
    if let Some(&idx) = candidates.choose(&mut rng) {
        let territory_id = state.territories[idx].id.clone();
        let ruin = generate_ruin(&territory_id);
        
        // 遺跡の敵情報をマスに反映
        let troops = ruin.enemies.len() as u32;
        let body_energies = ruin.enemy_energies.clone();
        let body_names = ruin.enemy_names.clone();
        
        state.territories[idx].ruin = Some(ruin);
        state.territories[idx].troops = troops;
        state.territories[idx].body_energies = Some(body_energies);
        state.territories[idx].body_names = Some(body_names);
        
        true
    } else {
        false
    }
}

/// 現在の遺跡数をカウント
pub fn count_ruins(state: &GameState) -> usize {
    state.territories.iter().filter(|t| t.ruin.is_some()).count()
}
