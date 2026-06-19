use super::*;
use crate::cards::{card_has_illustration, enemy_name_has_illustration};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

/// Lv1-3 ベース地形の加重（平原25% / 丘陵35% / 森40%）。Lv4=川・Lv5=高山・Lv6=山地は後段フェーズで付与
const TERRAIN_BASE_WEIGHTS: [u32; 3] = [25, 35, 40];
/// 川タイルの領地レベル
pub(crate) const TERRAIN_LEVEL_RIVER: u8 = 4;
/// 高山（山岳クラスタ外周・雪峰）
pub(crate) const TERRAIN_LEVEL_ALPINE: u8 = 5;
/// 山地（山岳クラスタ中心）
pub(crate) const TERRAIN_LEVEL_MOUNTAIN: u8 = 6;
/// 険境タイルの領地レベル
pub(crate) const TERRAIN_LEVEL_PERIL: u8 = 7;
/// 魔境タイルの領地レベル
pub(crate) const TERRAIN_LEVEL_DEMON: u8 = 8;
/// 深域タイルの領地レベル
pub(crate) const TERRAIN_LEVEL_DEEP: u8 = 9;
/// Lv3/Lv4 から山岳シードを置く確率（2.0%）
const MOUNTAIN_SEED_CHANCE: (u32, u32) = (20, 1000);
/// 山岳シードから隣接マスへ広がる確率（45%）
const MOUNTAIN_SPREAD_CHANCE: (u32, u32) = (45, 100);
/// 川セグメント（3マス）配置試行確率（2.5% / マス）
const RIVER_SEGMENT_CHANCE: (u32, u32) = (25, 1000);
/// Lv7-9 パッチ確率（万分率、Lv9深域が最レア）
const DEEP_TERRAIN_CHANCE_9: (u32, u32) = (12, 10000);
const DEEP_TERRAIN_CHANCE_8: (u32, u32) = (35, 10000);
const DEEP_TERRAIN_CHANCE_7: (u32, u32) = (70, 10000);

/// 地形生成シードを解決（明示指定 > 環境変数 > ランダム）
pub fn resolve_terrain_seed(explicit: Option<u64>) -> u64 {
    if let Some(seed) = explicit {
        return seed;
    }
    crate::config::optional_terrain_seed_from_env()
        .unwrap_or_else(|| rand::thread_rng().gen())
}

fn pick_base_level(rng: &mut StdRng) -> u8 {
    let total: u32 = TERRAIN_BASE_WEIGHTS.iter().sum();
    let roll = rng.gen_range(0..total);
    let mut acc = 0u32;
    for (i, &weight) in TERRAIN_BASE_WEIGHTS.iter().enumerate() {
        acc += weight;
        if roll < acc {
            return (i + 1) as u8;
        }
    }
    3
}

fn fill_base_terrain(cols: usize, rows: usize, rng: &mut StdRng) -> Vec<Vec<u8>> {
    (0..rows)
        .map(|_| (0..cols).map(|_| pick_base_level(rng)).collect())
        .collect()
}

fn cluster_base_terrain(grid: &[Vec<u8>], cols: usize, rows: usize, rng: &mut StdRng) -> Vec<Vec<u8>> {
    let mut next = grid.to_vec();
    for row in 0..rows {
        for col in 0..cols {
            let level = grid[row][col];
            let mut same = 0u8;
            for (nr, nc) in neighbors4(row, col, rows, cols) {
                if grid[nr][nc] == level {
                    same += 1;
                }
            }
            // 隣接に同レベルが2枚以上あれば、そのまま維持。なければ25%で隣のレベルへ寄せる
            if same >= 2 || !rng.gen_ratio(25, 100) {
                continue;
            }
            let neighbors = neighbors4(row, col, rows, cols);
            if let Some(&(nr, nc)) = neighbors.choose(rng) {
                next[row][col] = grid[nr][nc];
            }
        }
    }
    next
}

fn spread_mountains(grid: &mut [Vec<u8>], cols: usize, rows: usize, rng: &mut StdRng) {
    for row in 0..rows {
        for col in 0..cols {
            let level = grid[row][col];
            if (level == 3 || level == 4) && rng.gen_ratio(MOUNTAIN_SEED_CHANCE.0, MOUNTAIN_SEED_CHANCE.1) {
                grid[row][col] = TERRAIN_LEVEL_MOUNTAIN;
            }
        }
    }
    let mut to_spread = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            if grid[row][col] == TERRAIN_LEVEL_MOUNTAIN {
                for (nr, nc) in neighbors4(row, col, rows, cols) {
                    let neighbor = grid[nr][nc];
                    if neighbor != TERRAIN_LEVEL_MOUNTAIN
                        && neighbor != TERRAIN_LEVEL_ALPINE
                        && neighbor != TERRAIN_LEVEL_RIVER
                    {
                        to_spread.push((nr, nc));
                    }
                }
            }
        }
    }
    for (row, col) in to_spread {
        if grid[row][col] != TERRAIN_LEVEL_MOUNTAIN
            && grid[row][col] != TERRAIN_LEVEL_ALPINE
            && grid[row][col] != TERRAIN_LEVEL_RIVER
            && rng.gen_ratio(MOUNTAIN_SPREAD_CHANCE.0, MOUNTAIN_SPREAD_CHANCE.1)
        {
            grid[row][col] = TERRAIN_LEVEL_ALPINE;
        }
    }
    // Mountain clustering: existing mountains can grow into adjacent tiles
    let mountains: Vec<(usize, usize)> = (0..rows)
        .flat_map(|r| (0..cols).map(move |c| (r, c)))
        .filter(|&(r, c)| grid[r][c] == TERRAIN_LEVEL_MOUNTAIN)
        .collect();
    for (row, col) in mountains {
        if rng.gen_ratio(40, 100) {
            let nbrs = neighbors4(row, col, rows, cols);
            let candidates: Vec<(usize, usize)> = nbrs
                .into_iter()
                .filter(|&(nr, nc)| {
                    grid[nr][nc] != TERRAIN_LEVEL_MOUNTAIN
                        && grid[nr][nc] != TERRAIN_LEVEL_ALPINE
                        && grid[nr][nc] != TERRAIN_LEVEL_RIVER
                })
                .collect();
            if let Some(&(nr, nc)) = candidates.choose(rng) {
                grid[nr][nc] = TERRAIN_LEVEL_MOUNTAIN;
            }
        }
    }
}

fn neighbors4(row: usize, col: usize, rows: usize, cols: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(4);
    if row > 0 {
        out.push((row - 1, col));
    }
    if row + 1 < rows {
        out.push((row + 1, col));
    }
    if col > 0 {
        out.push((row, col - 1));
    }
    if col + 1 < cols {
        out.push((row, col + 1));
    }
    out
}

fn river_segment_cells(
    start_col: usize,
    start_row: usize,
    dc: i32,
    dr: i32,
) -> [(usize, usize); 3] {
    [
        (start_col, start_row),
        ((start_col as i32 + dc) as usize, (start_row as i32 + dr) as usize),
        ((start_col as i32 + dc * 2) as usize, (start_row as i32 + dr * 2) as usize),
    ]
}

fn can_place_river_segment(
    grid: &[Vec<u8>],
    cells: &[(usize, usize); 3],
    rows: usize,
    cols: usize,
) -> bool {
    for &(c, r) in cells {
        if grid[r][c] == TERRAIN_LEVEL_MOUNTAIN
            || grid[r][c] == TERRAIN_LEVEL_ALPINE
            || grid[r][c] == TERRAIN_LEVEL_RIVER
        {
            return false;
        }
        // 既存の川に隣接するセグメントは置かない（3マス単位の独立した川のみ）
        for (nr, nc) in neighbors4(r, c, rows, cols) {
            if grid[nr][nc] == TERRAIN_LEVEL_RIVER {
                return false;
            }
        }
    }
    true
}

fn place_river_segments(grid: &mut [Vec<u8>], cols: usize, rows: usize, rng: &mut StdRng) {
    let mut order: Vec<(usize, usize)> = Vec::with_capacity(cols * rows);
    for row in 0..rows {
        for col in 0..cols {
            order.push((col, row));
        }
    }
    order.shuffle(rng);

    for (start_col, start_row) in order {
        if !rng.gen_ratio(RIVER_SEGMENT_CHANCE.0, RIVER_SEGMENT_CHANCE.1) {
            continue;
        }
        let horizontal = rng.gen_bool(0.5);
        let mut options: Vec<(i32, i32)> = Vec::new();
        if horizontal {
            if start_col + 2 < cols {
                options.push((1, 0));
            }
            if start_col >= 2 {
                options.push((-1, 0));
            }
        } else {
            if start_row + 2 < rows {
                options.push((0, 1));
            }
            if start_row >= 2 {
                options.push((0, -1));
            }
        }
        if options.is_empty() {
            continue;
        }
        let (dc, dr) = options[rng.gen_range(0..options.len())];
        let cells = river_segment_cells(start_col, start_row, dc, dr);
        if !can_place_river_segment(grid, &cells, rows, cols) {
            continue;
        }
        for &(c, r) in &cells {
            grid[r][c] = TERRAIN_LEVEL_RIVER;
        }
    }
}

fn scatter_deep_terrain(grid: &mut [Vec<u8>], cols: usize, rows: usize, rng: &mut StdRng) {
    for row in 0..rows {
        for col in 0..cols {
            let level = grid[row][col];
            if level == TERRAIN_LEVEL_RIVER
                || level == TERRAIN_LEVEL_MOUNTAIN
                || level == TERRAIN_LEVEL_ALPINE
                || level >= TERRAIN_LEVEL_PERIL
            {
                continue;
            }
            if rng.gen_ratio(DEEP_TERRAIN_CHANCE_9.0, DEEP_TERRAIN_CHANCE_9.1) {
                grid[row][col] = TERRAIN_LEVEL_DEEP;
            } else if rng.gen_ratio(DEEP_TERRAIN_CHANCE_8.0, DEEP_TERRAIN_CHANCE_8.1) {
                grid[row][col] = TERRAIN_LEVEL_DEMON;
            } else if rng.gen_ratio(DEEP_TERRAIN_CHANCE_7.0, DEEP_TERRAIN_CHANCE_7.1) {
                grid[row][col] = TERRAIN_LEVEL_PERIL;
            }
        }
    }
}

/// シード付きの地形レベルグリッドを生成（Lv1-9、5フェーズパイプライン）
pub(crate) fn random_level_grid(cols: u16, rows: u16, seed: u64) -> Vec<Vec<u8>> {
    let cols = cols as usize;
    let rows = rows as usize;
    let mut rng = StdRng::seed_from_u64(seed);
    let base = fill_base_terrain(cols, rows, &mut rng);
    let mut grid = cluster_base_terrain(&base, cols, rows, &mut rng);
    spread_mountains(&mut grid, cols, rows, &mut rng);
    place_river_segments(&mut grid, cols, rows, &mut rng);
    scatter_deep_terrain(&mut grid, cols, rows, &mut rng);
    grid
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

/// KC準拠: 領地Lvごとに選べる敵カード候補（イラストありの収集魔獣のみ）
fn neutral_card_pool_for_level(level: u8) -> &'static [u32] {
    match level {
        1 => &[50, 60, 70, 80, 90, 100, 110],
        2 => &[51, 61, 71, 81, 91, 101, 111],
        3 => &[52, 62, 63, 64, 72, 82, 102, 112],
        4 => &[53, 54, 55, 65, 73, 92, 103, 113],
        5 => &[56, 66, 74, 83, 93, 104, 114],
        6 => &[56, 66, 74, 83],
        7 => &[56, 74, 83, 93],
        8 => &[83, 93, 104, 114],
        9 => &[83, 93, 104, 114],
        _ => &[83, 93, 104, 114],
    }
}

fn illustrated_neutral_pool(level: u8) -> Vec<u32> {
    neutral_card_pool_for_level(level)
        .iter()
        .copied()
        .filter(|&id| card_has_illustration(id))
        .collect()
}

fn hash_seed(key: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

fn pick_distinct_neutral_cards(pool: &[u32], count: usize, rng: &mut impl Rng) -> Vec<u32> {
    let mut ids: Vec<u32> = pool.to_vec();
    ids.sort_unstable();
    ids.dedup();
    ids.shuffle(rng);
    ids.truncate(count.min(ids.len()));
    ids
}

fn neutral_enemy_stats(level: u8) -> (u32, u32) {
    match level {
        // Lv1: 初期所持2体編成（コスト上限4.0）でぎりぎり勝てる程度
        1 => (1, 18),
        2 => (1, 250),
        3 => (2, 250),
        4 => (2, 500),
        5 => (3, 1500),
        6 => (3, 3500),
        7 => (2, 6000),
        8 => (3, 8500),
        9 => (3, 9000),
        _ => (3, 9000),
    }
}

/// 領地Lvごとの占領報酬レンジ（4資源とも同一 min〜max で独立ランダム）
fn conquest_resource_range(level: u8) -> (u64, u64) {
    let mul = level as u64;
    let min = 10u64.saturating_mul(mul).saturating_add(5);
    let max = 50u64.saturating_mul(mul).saturating_add(30);
    (min, max)
}

/// 領地占領時の即時資源ボーナス（食料・木材・石材・鉄）。時間生産とは別。
pub(crate) fn conquest_resource_bonus(level: u8) -> (u64, u64, u64, u64) {
    let (min, max) = conquest_resource_range(level);
    let mut rng = rand::thread_rng();
    let mut roll = || rng.gen_range(min..=max);
    (roll(), roll(), roll(), roll())
}

/// 探索到着時の資源ボーナス（占領ロールの40%・必ず成功）
pub(crate) fn exploration_resource_bonus(level: u8) -> (u64, u64, u64, u64) {
    let (f, w, s, i) = conquest_resource_bonus(level);
    (f * 2 / 5, w * 2 / 5, s * 2 / 5, i * 2 / 5)
}

/// 領地Lvとシードキーから中立敵を決定的に生成（未触の中立マスは state に保存しない）
pub(crate) fn generate_neutral_enemies_for_territory(level: u8, seed_key: &str) -> (u32, Vec<u32>, Vec<String>) {
    let (count, mc_per_body) = neutral_enemy_stats(level);
    let mut rng = StdRng::seed_from_u64(hash_seed(seed_key));
    let pool = illustrated_neutral_pool(level);
    let card_ids = if pool.is_empty() {
        vec![4] // ゴーレム（イラストあり）フォールバック
    } else {
        pick_distinct_neutral_cards(&pool, count as usize, &mut rng)
    };
    let troops = card_ids.len() as u32;
    let monster_counts = vec![mc_per_body; card_ids.len()];
    let names: Vec<String> = card_ids
        .iter()
        .map(|&id| {
            get_card(id)
                .map(|c| c.name.to_string())
                .unwrap_or_else(|| format!("敵#{id}"))
        })
        .collect();
    (troops, monster_counts, names)
}

/// 連戦などシード不要な場面向け（従来互換）
pub(crate) fn generate_neutral_enemies(level: u8) -> (u32, Vec<u32>, Vec<String>) {
    generate_neutral_enemies_for_territory(level, &format!("ephemeral:{level}"))
}

/// 戦闘編成の最大体数（FRONT / BACK / LEADER）
pub(crate) const BATTLE_FORMATION_SLOTS: usize = 3;

fn defenders_from_owned_slots(player: &PlayerData, slots: &[usize]) -> (u32, Vec<u32>, Vec<String>) {
    let mut names = Vec::new();
    let mut counts = Vec::new();
    for &i in slots {
        let card_id = player.owned_cards.get(i).copied().unwrap_or(0);
        let name = get_card(card_id)
            .map(|c| c.name.to_string())
            .unwrap_or_else(|| format!("魔獣#{card_id}"));
        let mc = player.card_monster_counts.get(i).copied().unwrap_or(1).max(1);
        names.push(name);
        counts.push(mc);
    }
    (names.len() as u32, counts, names)
}

fn formed_unit_available_at_home(player: &PlayerData, unit: &StoredFormedUnit, now: u64) -> bool {
    if !is_kc_unit_ready_to_deploy(&unit.indices) {
        return false;
    }
    if march_busy_formed_unit_ids(player, now).contains(&unit.id) {
        return false;
    }
    let locked = march_locked_card_slots(player, now);
    formation_owned_slots_in_slot_order(&unit.indices)
        .iter()
        .all(|&slot| slot < player.owned_cards.len() && !locked.contains(&slot))
}

fn score_defense_unit_power(player: &PlayerData, unit: &StoredFormedUnit) -> u64 {
    formation_owned_slots_in_slot_order(&unit.indices)
        .iter()
        .map(|&i| player.card_monster_counts.get(i).copied().unwrap_or(1) as u64)
        .sum()
}

fn pick_home_defense_unit<'a>(player: &'a PlayerData, now: u64) -> Option<&'a StoredFormedUnit> {
    player
        .formed_units
        .iter()
        .filter(|u| formed_unit_available_at_home(player, u, now))
        .max_by_key(|u| score_defense_unit_power(player, u))
}

fn fallback_defense_slots(player: &PlayerData, now: u64) -> Vec<usize> {
    let locked = march_locked_card_slots(player, now);
    let mut available: Vec<usize> = (0..player.owned_cards.len())
        .filter(|i| !locked.contains(i))
        .collect();
    available.sort_by_key(|&i| {
        std::cmp::Reverse(player.card_monster_counts.get(i).copied().unwrap_or(1))
    });
    available.truncate(BATTLE_FORMATION_SLOTS);
    available
}

fn resolve_owned_player_defenders(
    player: &PlayerData,
    territory: &Territory,
    now: u64,
) -> (u32, Vec<u32>, Vec<String>) {
    // 占領地の駐留部隊（援軍で body_names が付与されている）
    if let (Some(names), Some(counts)) = (&territory.body_names, &territory.body_monster_counts) {
        let n = names.len();
        if n > 0 && n == counts.len() && territory.troops as usize == n {
            return (territory.troops, counts.clone(), names.clone());
        }
    }

    // 本拠など: 保存済みユニット編成から守備ユニットを選ぶ（プレイヤー・AI共通）
    if let Some(unit) = pick_home_defense_unit(player, now) {
        let slots = formation_owned_slots_in_slot_order(&unit.indices);
        return defenders_from_owned_slots(player, &slots);
    }

    let slots = fallback_defense_slots(player, now);
    defenders_from_owned_slots(player, &slots)
}

fn resolve_stored_garrison_defenders(territory: &Territory) -> (u32, Vec<u32>, Vec<String>) {
    let troops = territory.troops;
    let counts = territory
        .body_monster_counts
        .clone()
        .filter(|values| values.len() == troops as usize)
        .unwrap_or_else(|| vec![1u32; troops.max(1) as usize]);
    let names = territory
        .body_names
        .clone()
        .filter(|values| values.len() == troops as usize)
        .unwrap_or_else(|| {
            (1..=troops.max(1))
                .map(|i| format!("敵ユニット{i}"))
                .collect()
        });
    (troops, counts, names)
}

/// 戦闘時の守備編成を解決。駐留部隊があればそれを、なければ formed_units の守備ユニットを使う。
pub(crate) fn resolve_territory_defenders(
    territory: &Territory,
    players: &HashMap<String, PlayerData>,
) -> (u32, Vec<u32>, Vec<String>) {
    let now = default_now_ms();
    if let Some(owner_id) = territory.owner_id.as_ref() {
        if let Some(player) = players.get(owner_id) {
            return resolve_owned_player_defenders(player, territory, now);
        }
    }
    if territory.ruin.is_some() || territory.body_names.is_some() || territory.owner_id.is_some() {
        return resolve_stored_garrison_defenders(territory);
    }
    generate_neutral_enemies_for_territory(territory.level, &territory.id)
}

fn enemy_name_species_key(name: &str) -> &str {
    name.trim_end_matches(|c: char| c == 'A' || c == 'B' || c == 'C')
}

/// 旧形式（トロールA/B/C など同一種の複製）か、同種重複、またはイラストなし敵
fn neutral_enemy_names_need_refresh(names: &[String]) -> bool {
    if names.is_empty() {
        return false;
    }
    let mut species = HashSet::new();
    for name in names {
        if !enemy_name_has_illustration(name) {
            return true;
        }
        let base = enemy_name_species_key(name);
        if !species.insert(base) {
            return true;
        }
        if name.len() == base.len() + 1 {
            let suffix = name.chars().last().unwrap_or(' ');
            if suffix == 'A' || suffix == 'B' || suffix == 'C' {
                return true;
            }
        }
    }
    false
}

/// 未触の中立マスから敵編成の永続データを除去（Lv + マスID から都度生成する）
pub fn migrate_neutral_enemy_storage(state: &mut GameState) -> bool {
    let mut stripped = 0usize;
    let mut fixed_legacy = 0usize;
    for territory in state.territories.iter_mut() {
        if territory.owner_id.is_some() || territory.is_base {
            continue;
        }
        if territory.ruin.is_some() {
            continue;
        }
        if let Some(names) = territory.body_names.clone() {
            if neutral_enemy_names_need_refresh(&names) {
                let (troops, body_monster_counts, body_names) =
                    generate_neutral_enemies_for_territory(territory.level, &territory.id);
                territory.troops = troops;
                territory.body_monster_counts = Some(body_monster_counts);
                territory.body_names = Some(body_names);
                fixed_legacy += 1;
            }
            continue;
        }
        if territory.troops != 0
            || territory.body_monster_counts.is_some()
            || territory.body_names.is_some()
        {
            territory.troops = 0;
            territory.body_monster_counts = None;
            territory.body_names = None;
            stripped += 1;
        }
    }
    if stripped > 0 {
        println!(
            "[kingdom-server] 未触中立マス {stripped} 件の敵編成を state から除去しました"
        );
    }
    if fixed_legacy > 0 {
        println!(
            "[kingdom-server] 攻撃済み中立マスの旧編成（同一種ABC）を {fixed_legacy} 件更新しました"
        );
    }
    stripped > 0 || fixed_legacy > 0
}

/// 後方互換エイリアス
pub fn migrate_legacy_neutral_enemies(state: &mut GameState) -> bool {
    migrate_neutral_enemy_storage(state)
}

pub(crate) const HOME_SAFE_ZONE_RADIUS: i32 = 2;

fn home_manhattan_distance(home_col: i32, home_row: i32, col: i32, row: i32) -> i32 {
    (col - home_col).abs() + (row - home_row).abs()
}

/// 本拠から距離 1..=HOME_SAFE_ZONE_RADIUS のマス ID（本拠自身は含めない）
pub(crate) fn home_safe_zone_territory_ids(
    home_col: u16,
    home_row: u16,
    world: &WorldConfig,
) -> Vec<String> {
    let hc = home_col as i32;
    let hr = home_row as i32;
    let mut ids = Vec::new();
    for row in 0..world.rows as i32 {
        for col in 0..world.cols as i32 {
            let dist = home_manhattan_distance(hc, hr, col, row);
            if (1..=HOME_SAFE_ZONE_RADIUS).contains(&dist) {
                ids.push(format!("c_{}_{}", col, row));
            }
        }
    }
    ids
}

fn safe_zone_level_for_tile(home_id: &str, tile_id: &str) -> u8 {
    let mut rng = StdRng::seed_from_u64(hash_seed(&format!("{home_id}:{tile_id}")));
    if rng.gen_bool(0.5) {
        1
    } else {
        2
    }
}

/// 本拠を置けるか: 本拠マスが中立かつ、距離0〜2に他プレイヤー領地がない
pub(crate) fn can_place_home_with_safe_zone(
    territories: &[Territory],
    home_territory_id: &str,
    for_owner_id: &str,
    world: &WorldConfig,
) -> bool {
    let (home_col, home_row) = match parse_territory_id(home_territory_id) {
        Some(coords) => coords,
        None => return false,
    };
    if home_col < 0
        || home_row < 0
        || home_col >= world.cols as i32
        || home_row >= world.rows as i32
    {
        return false;
    }
    let Some(home_idx) = get_territory_index(territories, home_territory_id) else {
        return false;
    };
    let home = &territories[home_idx];
    if home.ruin.is_some() {
        return false;
    }
    if let Some(owner) = &home.owner_id {
        if owner != for_owner_id {
            return false;
        }
    }
    for territory in territories {
        let Some((col, row)) = parse_territory_id(&territory.id) else {
            continue;
        };
        let dist = home_manhattan_distance(home_col, home_row, col, row);
        if dist > HOME_SAFE_ZONE_RADIUS {
            continue;
        }
        if let Some(owner) = &territory.owner_id {
            if owner != for_owner_id {
                return false;
            }
        }
    }
    true
}

/// 安全圏の中立マスをレベル1or2に上書き（他プレイヤー・遺跡はスキップ）
pub(crate) fn apply_home_safe_zone_levels(
    territories: &mut [Territory],
    home_col: u16,
    home_row: u16,
    world: &WorldConfig,
) {
    let home_id = format!("c_{}_{}", home_col, home_row);
    for tile_id in home_safe_zone_territory_ids(home_col, home_row, world) {
        let Some(idx) = get_territory_index(territories, &tile_id) else {
            continue;
        };
        if territories[idx].owner_id.is_some() || territories[idx].ruin.is_some() {
            continue;
        }
        let level = safe_zone_level_for_tile(&home_id, &tile_id);
        let territory = &mut territories[idx];
        territory.level = level;
        territory.name = terrain_name(level).to_string();
        territory.troops = 0;
        territory.body_monster_counts = None;
        territory.body_names = None;
    }
}

/// 遺跡はバックグラウンドタスクで動的にスポーンする（突発イベント）。
pub fn generate_territories(
    world: &WorldConfig,
    home_owner_id: &str,
    home_owner_id_override: Option<&str>,
) -> Vec<Territory> {
    let seed = if world.terrain_seed != 0 {
        world.terrain_seed
    } else {
        resolve_terrain_seed(None)
    };
    let level_grid = random_level_grid(world.cols, world.rows, seed);
    let owner = home_owner_id_override.unwrap_or(home_owner_id);
    let mut out = Vec::with_capacity(world.cols as usize * world.rows as usize);
    for row in 0..world.rows {
        for col in 0..world.cols {
            let id = format!("c_{}_{}", col, row);
            let is_home = col == world.home_col && row == world.home_row;
            let level = if is_home {
                1u8
            } else {
                level_grid[row as usize][col as usize]
            };
            let name = terrain_name(level).to_string();

            if is_home {
                let owned = default_owned_cards();
                let home_mc = initial_card_monster_counts_for_owned(&owned);
                let ntroops = owned.len() as u32;
                out.push(Territory {
                    id,
                    name,
                    level,
                    owner_id: Some(owner.to_string()),
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
                out.push(Territory {
                    id,
                    name,
                    level,
                    owner_id: None,
                    troops: 0,
                    body_monster_counts: None,
                    body_names: None,
                    ruin: None,
                    is_base: false,
                    durability: 0,
                    max_durability: 0,
                    tower_level: 0,
                });
            }
        }
    }
    apply_home_safe_zone_levels(&mut out, world.home_col, world.home_row, world);
    out
}

#[cfg(test)]
pub fn migrate_legacy_terrain(state: &mut GameState) -> bool {
    if state.world.terrain_seed != 0 {
        return false;
    }
    state.world.terrain_seed = resolve_terrain_seed(None);
    let seed = state.world.terrain_seed;
    let grid = random_level_grid(state.world.cols, state.world.rows, seed);
    let world = state.world;

    for territory in &mut state.territories {
        let Some((col, row)) = parse_territory_id(&territory.id) else {
            continue;
        };
        if col < 0
            || row < 0
            || col >= world.cols as i32
            || row >= world.rows as i32
        {
            continue;
        }
        let level = if territory.is_base {
            1u8
        } else {
            grid[row as usize][col as usize]
        };
        territory.level = level;
        territory.name = terrain_name(level).to_string();
        if territory.owner_id.is_none() && territory.ruin.is_none() {
            territory.troops = 0;
            territory.body_monster_counts = None;
            territory.body_names = None;
        }
    }

    apply_home_safe_zone_levels(
        &mut state.territories,
        world.home_col,
        world.home_row,
        &world,
    );
    for faction in state.ai_factions.clone() {
        if let Some((col, row)) = parse_territory_id(&faction.home_territory_id) {
            if col >= 0 && row >= 0 {
                apply_home_safe_zone_levels(
                    &mut state.territories,
                    col as u16,
                    row as u16,
                    &world,
                );
            }
        }
    }

    println!(
        "[kingdom-server] 旧地形を新ジェネレータで再生成しました (terrain_seed={seed})"
    );
    true
}

fn terrain_name(level: u8) -> &'static str {
    match level {
        1 => "平原",
        2 => "丘陵",
        3 => "森",
        4 => "川",
        5 => "山岳",
        6 => "山地",
        7 => "険境",
        8 => "魔境",
        9 => "深域",
        _ => "平原",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_legacy_abc_neutral_names() {
        let legacy = vec![
            "トロールA".to_string(),
            "トロールB".to_string(),
            "トロールC".to_string(),
        ];
        assert!(neutral_enemy_names_need_refresh(&legacy));
    }

    #[test]
    fn accepts_distinct_illustrated_neutral_names() {
        let names = vec![
            "バット".to_string(),
            "ゴブリンアーチャー".to_string(),
            "レッサーデーモン".to_string(),
        ];
        assert!(!neutral_enemy_names_need_refresh(&names));
    }

    #[test]
    fn refreshes_unillustrated_neutral_names() {
        let names = vec!["コボルド".to_string()];
        assert!(neutral_enemy_names_need_refresh(&names));
    }

    #[test]
    fn neutral_enemies_are_deterministic_per_territory() {
        let a = generate_neutral_enemies_for_territory(5, "c_2_3");
        let b = generate_neutral_enemies_for_territory(5, "c_2_3");
        assert_eq!(a, b);
        let c = generate_neutral_enemies_for_territory(5, "c_2_4");
        assert_ne!(a.2, c.2);
    }

    #[test]
    fn resolve_owned_home_defenders_use_formed_unit() {
        let mut player = PlayerData::new("ai_test".into(), "c_1_1".into());
        player.owned_cards = default_owned_cards();
        player.card_monster_counts = vec![100; 10];
        player.formed_units = vec![StoredFormedUnit {
            id: "unit-1".into(),
            name: "守備隊".into(),
            indices: [0, 1, 2],
        }];
        let mut players = HashMap::new();
        players.insert("ai_test".into(), player);
        let territory = Territory {
            id: "c_2_2".into(),
            name: "丘陵".into(),
            level: 2,
            owner_id: Some("ai_test".into()),
            troops: 10,
            body_monster_counts: Some(vec![100; 10]),
            body_names: None,
            ruin: None,
            is_base: true,
            durability: 0,
            max_durability: 0,
            tower_level: 0,
        };
        let (troops, _, names) = resolve_territory_defenders(&territory, &players);
        assert_eq!(troops, 3);
        assert_eq!(names.len(), 3);
        assert!(!names[0].starts_with("敵ユニット"));
        let odin = get_card(0).map(|c| c.name).unwrap_or("");
        assert!(names.contains(&odin.to_string()));
    }

    #[test]
    fn resolve_procedural_neutral_without_storage() {
        let territory = Territory {
            id: "c_3_4".to_string(),
            name: "森".to_string(),
            level: 3,
            owner_id: None,
            troops: 0,
            body_monster_counts: None,
            body_names: None,
            ruin: None,
            is_base: false,
            durability: 0,
            max_durability: 0,
            tower_level: 0,
        };
        let (troops, _, names) = resolve_territory_defenders(&territory, &HashMap::new());
        assert!(troops > 0);
        assert!(!names.is_empty());
    }

    fn test_world() -> WorldConfig {
        WorldConfig {
            cols: 12,
            rows: 12,
            home_col: 6,
            home_row: 6,
            terrain_seed: 42,
        }
    }

    fn count_level(grid: &[Vec<u8>], level: u8) -> usize {
        grid.iter()
            .flat_map(|row| row.iter())
            .filter(|&&cell| cell == level)
            .count()
    }

    fn horizontal_river_run(grid: &[Vec<u8>], row: usize, col: usize) -> usize {
        if grid[row][col] != TERRAIN_LEVEL_RIVER {
            return 0;
        }
        let mut start = col;
        while start > 0 && grid[row][start - 1] == TERRAIN_LEVEL_RIVER {
            start -= 1;
        }
        let mut end = col;
        let cols = grid[row].len();
        while end + 1 < cols && grid[row][end + 1] == TERRAIN_LEVEL_RIVER {
            end += 1;
        }
        end - start + 1
    }

    fn vertical_river_run(grid: &[Vec<u8>], row: usize, col: usize) -> usize {
        if grid[row][col] != TERRAIN_LEVEL_RIVER {
            return 0;
        }
        let mut start = row;
        while start > 0 && grid[start - 1][col] == TERRAIN_LEVEL_RIVER {
            start -= 1;
        }
        let mut end = row;
        let rows = grid.len();
        while end + 1 < rows && grid[end + 1][col] == TERRAIN_LEVEL_RIVER {
            end += 1;
        }
        end - start + 1
    }

    fn assert_river_segments_valid(grid: &[Vec<u8>]) {
        let rows = grid.len();
        let cols = grid[0].len();
        let mut visited = vec![vec![false; cols]; rows];
        for row in 0..rows {
            for col in 0..cols {
                if grid[row][col] != TERRAIN_LEVEL_RIVER || visited[row][col] {
                    continue;
                }
                let mut component = Vec::new();
                let mut stack = vec![(row, col)];
                while let Some((r, c)) = stack.pop() {
                    if visited[r][c] || grid[r][c] != TERRAIN_LEVEL_RIVER {
                        continue;
                    }
                    visited[r][c] = true;
                    component.push((r, c));
                    for (nr, nc) in neighbors4(r, c, rows, cols) {
                        if grid[nr][nc] == TERRAIN_LEVEL_RIVER && !visited[nr][nc] {
                            stack.push((nr, nc));
                        }
                    }
                }
                assert_eq!(
                    component.len(),
                    3,
                    "river component size {} at ({col},{row})",
                    component.len()
                );
                let h = horizontal_river_run(grid, row, col);
                let v = vertical_river_run(grid, row, col);
                assert!(
                    h == 3 || v == 3,
                    "river component at ({col},{row}) not straight-3 (h={h}, v={v})"
                );
            }
        }
    }

    #[test]
    fn same_seed_produces_identical_grid() {
        let a = random_level_grid(48, 48, 12345);
        let b = random_level_grid(48, 48, 12345);
        assert_eq!(a, b);
    }

    #[test]
    fn different_seed_produces_different_grid() {
        let a = random_level_grid(48, 48, 1);
        let b = random_level_grid(48, 48, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn river_segments_are_three_connected() {
        for seed in [1u64, 42, 999, 12345] {
            let grid = random_level_grid(48, 48, seed);
            assert_river_segments_valid(&grid);
        }
    }

    #[test]
    fn higher_levels_are_rarer() {
        let grid = random_level_grid(48, 48, 42);
        let lv1 = count_level(&grid, 1);
        let lv6 = count_level(&grid, TERRAIN_LEVEL_MOUNTAIN);
        let lv7 = count_level(&grid, TERRAIN_LEVEL_PERIL);
        let lv9 = count_level(&grid, TERRAIN_LEVEL_DEEP);
        assert!(lv1 > lv6, "lv1={lv1} should exceed lv6={lv6}");
        assert!(lv6 > lv7, "lv6={lv6} should exceed lv7={lv7}");
        assert!(lv7 > lv9, "lv7={lv7} should exceed lv9 deep={lv9}");
    }

    #[test]
    fn mountains_cluster_more_than_random() {
        let grid = random_level_grid(48, 48, 42);
        let mut lv6_with_adjacent = 0usize;
        let mut lv6_total = 0usize;
        let rows = grid.len();
        let cols = grid[0].len();
        for row in 0..rows {
            for col in 0..cols {
                if grid[row][col] != TERRAIN_LEVEL_MOUNTAIN {
                    continue;
                }
                lv6_total += 1;
                let has_adjacent_lv6 = neighbors4(row, col, rows, cols)
                    .iter()
                    .any(|&(r, c)| grid[r][c] == TERRAIN_LEVEL_MOUNTAIN);
                if has_adjacent_lv6 {
                    lv6_with_adjacent += 1;
                }
            }
        }
        if lv6_total == 0 {
            return;
        }
        let rate = lv6_with_adjacent as f64 / lv6_total as f64;
        assert!(
            rate > 0.15,
            "lv6 mountain adjacency rate {rate} should show clustering (total={lv6_total})"
        );
    }

    #[test]
    fn migrate_legacy_terrain_regenerates_when_seed_missing() {
        let mut state = GameState::default();
        state.world.terrain_seed = 0;
        let before: Vec<u8> = state.territories.iter().map(|t| t.level).collect();
        assert!(migrate_legacy_terrain(&mut state));
        assert_ne!(state.world.terrain_seed, 0);
        let after: Vec<u8> = state.territories.iter().map(|t| t.level).collect();
        assert_ne!(before, after, "terrain levels should change after migration");
    }

    #[test]
    fn migrate_legacy_terrain_skips_when_seed_present() {
        let mut state = GameState::default();
        state.world.terrain_seed = 99;
        let level = state.territories[0].level;
        assert!(!migrate_legacy_terrain(&mut state));
        assert_eq!(state.territories[0].level, level);
    }

    #[test]
    fn terrain_distribution_is_sane_for_new_generator() {
        let grid = random_level_grid(48, 48, 42);
        let mut counts = [0usize; 10];
        for row in &grid {
            for &cell in row {
                counts[cell as usize] += 1;
            }
        }
        let total = 48 * 48;
        let low = counts[1] + counts[2] + counts[3];
        let lv1 = counts[1];
        let lv4 = counts[TERRAIN_LEVEL_RIVER as usize];
        let lv5 = counts[TERRAIN_LEVEL_ALPINE as usize];
        let lv6 = counts[TERRAIN_LEVEL_MOUNTAIN as usize];
        let lv7 = counts[TERRAIN_LEVEL_PERIL as usize];
        let lv8 = counts[TERRAIN_LEVEL_DEMON as usize];
        let lv9 = counts[TERRAIN_LEVEL_DEEP as usize];
        assert!(lv5 > 0, "lv5 alpine={lv5} should appear around mountain clusters");
        assert!(lv6 > 0, "lv6 mountain={lv6} should appear from forest seeds");
        assert!(lv5 > lv6, "lv5 alpine ring={lv5} should exceed lv6 cores={lv6}");
        assert!(
            low * 100 < total * 95,
            "lv1-3={low} ({:.0}%) should stay below 95%",
            low as f64 / total as f64 * 100.0
        );
        assert!(lv4 > lv6, "lv4 rivers={lv4} should exceed lv6 mountains={lv6}");
        assert!(lv1 > lv6, "lv1={lv1} lv6={lv6}");
        assert!(lv7 > lv8, "lv7={lv7} should exceed lv8={lv8}");
        assert!(lv8 > lv9, "lv8={lv8} should exceed lv9 deep={lv9}");
        assert!(lv9 >= 2, "lv9 deep={lv9} should appear on 48x48");
        assert!(lv9 < 40, "lv9 deep={lv9} should stay rare");
        assert!(lv7 < 120, "lv7={lv7} should stay uncommon");
        assert!(lv4 >= 60, "lv4={lv4} should have enough river tiles");
        assert!(lv4 < 220, "lv4={lv4} from 3-tile segments");
        assert_river_segments_valid(&grid);
    }

    #[test]
    fn terrain_generation_is_fast_enough() {
        let start = std::time::Instant::now();
        for seed in 0..100u64 {
            let _ = random_level_grid(48, 48, seed);
        }
        assert!(
            start.elapsed().as_millis() < 600,
            "100 grids took {:?}",
            start.elapsed()
        );
    }

    #[test]
    fn apply_home_safe_zone_sets_levels_one_or_two() {
        let world = test_world();
        let territories = generate_territories(&world, "player_a", None);
        let home_idx = get_territory_index(&territories, "c_6_6").unwrap();
        assert_eq!(territories[home_idx].level, 1);
        for tile_id in home_safe_zone_territory_ids(world.home_col, world.home_row, &world) {
            let idx = get_territory_index(&territories, &tile_id).unwrap();
            let t = &territories[idx];
            if t.owner_id.is_none() && t.ruin.is_none() {
                assert!(t.level == 1 || t.level == 2, "tile {tile_id} level {}", t.level);
            }
        }
    }

    #[test]
    fn can_place_home_rejects_neighbor_owned_by_other() {
        let world = test_world();
        let mut territories = generate_territories(&world, "player_a", None);
        let neighbor_id = "c_7_6";
        let idx = get_territory_index(&territories, neighbor_id).unwrap();
        territories[idx].owner_id = Some("player_b".to_string());
        assert!(!can_place_home_with_safe_zone(
            &territories,
            "c_8_6",
            "player_c",
            &world
        ));
    }

    #[test]
    fn safe_zone_levels_are_deterministic() {
        let home_id = "c_6_6";
        let tile_id = "c_7_6";
        assert_eq!(
            safe_zone_level_for_tile(home_id, tile_id),
            safe_zone_level_for_tile(home_id, tile_id)
        );
    }

    #[test]
    fn apply_home_safe_zone_skips_ruins() {
        let world = test_world();
        let mut territories = generate_territories(&world, "player_a", None);
        let ruin_id = "c_7_6";
        let idx = get_territory_index(&territories, ruin_id).unwrap();
        let before = territories[idx].level;
        territories[idx].ruin = Some(crate::ruins::generate_ruin(ruin_id));
        apply_home_safe_zone_levels(&mut territories, world.home_col, world.home_row, &world);
        assert_eq!(territories[idx].level, before);
    }

    #[test]
    fn generate_territories_applies_safe_zone_on_init() {
        let world = test_world();
        let territories = generate_territories(&world, "solo", None);
        let zone_id = "c_6_7";
        let idx = get_territory_index(&territories, zone_id).unwrap();
        assert!(territories[idx].level == 1 || territories[idx].level == 2);
    }
}
