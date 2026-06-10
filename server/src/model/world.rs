use super::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

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

/// 領地レベルに対する連戦数（KC準拠: Lv4以上で連戦）
pub(crate) fn wave_count_for_level(level: u8) -> u32 {
    match level {
        1..=5 => 1,
        6 => 2,
        7..=8 => 2,
        _ => 3,
    }
}

/// KC準拠: 領地Lvごとに選べる敵カード候補（同一ユニット内で同種魔獣は不可のため、複数体時は別IDを使う）
fn neutral_card_pool_for_level(level: u8) -> &'static [u32] {
    match level {
        1 => &[10],              // ゴブリン
        2 => &[11],              // コボルド
        3 => &[12, 11],          // オーク・コボルド
        4 => &[13, 12],          // スケルトン・オーク
        5 => &[14, 13, 12],      // トロール・スケルトン・オーク
        6 => &[14, 37, 15],      // トロール・ミノタウロス・ドレイク（Lv20帯×3種）
        7 => &[35, 36],          // デスナイト・ヒュドラ
        8 => &[43, 41, 42],      // タイタン・ヴァンパイアロード・リッチ
        9 => &[40, 41, 42],      // ニーズヘッグ・ヴァンパイアロード・リッチ
        _ => &[40, 41, 42],
    }
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
        1 => (1, 50),
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

/// 領地Lvとシードキーから中立敵を決定的に生成（未触の中立マスは state に保存しない）
pub(crate) fn generate_neutral_enemies_for_territory(level: u8, seed_key: &str) -> (u32, Vec<u32>, Vec<String>) {
    let (count, mc_per_body) = neutral_enemy_stats(level);
    let mut rng = StdRng::seed_from_u64(hash_seed(seed_key));
    let card_ids =
        pick_distinct_neutral_cards(neutral_card_pool_for_level(level), count as usize, &mut rng);
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

/// 戦闘時の守備編成を解決。プレイヤー駐留・遺跡・攻撃後の残存のみ state を参照する。
pub(crate) fn resolve_territory_defenders(territory: &Territory) -> (u32, Vec<u32>, Vec<String>) {
    if territory.owner_id.is_some() || territory.ruin.is_some() || territory.body_names.is_some() {
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
        return (troops, counts, names);
    }
    generate_neutral_enemies_for_territory(territory.level, &territory.id)
}

fn enemy_name_species_key(name: &str) -> &str {
    name.trim_end_matches(|c: char| c == 'A' || c == 'B' || c == 'C')
}

/// 旧形式（トロールA/B/C など同一種の複製）か、同種重複の中立敵編成か
fn neutral_enemy_names_need_refresh(names: &[String]) -> bool {
    if names.is_empty() {
        return false;
    }
    let mut species = HashSet::new();
    for name in names {
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
    fn accepts_distinct_neutral_names() {
        let names = vec![
            "トロール".to_string(),
            "ミノタウロス".to_string(),
            "ドレイク".to_string(),
        ];
        assert!(!neutral_enemy_names_need_refresh(&names));
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
        let (troops, _, names) = resolve_territory_defenders(&territory);
        assert!(troops > 0);
        assert!(!names.is_empty());
    }
}

/// 遺跡はバックグラウンドタスクで動的にスポーンする（突発イベント）。
pub(super) fn default_territories() -> Vec<Territory> {
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
                // 中立マス: 敵編成は Lv + マスID から戦闘時に生成（state には level のみ保持）
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
    out
}
