use super::*;
use std::collections::HashSet;

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

fn pick_distinct_neutral_cards(pool: &[u32], count: usize) -> Vec<u32> {
    let mut ids: Vec<u32> = pool.to_vec();
    ids.sort_unstable();
    ids.dedup();
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    ids.shuffle(&mut rng);
    ids.truncate(count.min(ids.len()));
    ids
}

/// 全マスを領地として生成。id は c_{col}_{row}。本拠地 (24,24) のみプレイヤー所有。地形はランダム。
/// レベルに応じた中立地の敵を生成（魔獣マスタ定義を使用。1ユニット最大3種・同種不可）
pub(crate) fn generate_neutral_enemies(level: u8) -> (u32, Vec<u32>, Vec<String>) {
    let (count, mc_per_body): (u32, u32) = match level {
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
    };
    let card_ids = pick_distinct_neutral_cards(neutral_card_pool_for_level(level), count as usize);
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

/// 保存済み state の中立領地で、旧 ABC 複製編成を現行ルール（最大3種・同種不可）に差し替える
pub fn migrate_legacy_neutral_enemies(state: &mut GameState) -> bool {
    let mut fixed = 0usize;
    for territory in state.territories.iter_mut() {
        if territory.owner_id.is_some() || territory.is_base {
            continue;
        }
        let Some(names) = territory.body_names.clone() else {
            continue;
        };
        if !neutral_enemy_names_need_refresh(&names) {
            continue;
        }
        let (troops, body_monster_counts, body_names) = generate_neutral_enemies(territory.level);
        territory.troops = troops;
        territory.body_monster_counts = Some(body_monster_counts);
        territory.body_names = Some(body_names);
        fixed += 1;
    }
    if fixed > 0 {
        println!(
            "[kingdom-server] 中立敵の旧編成（同一種ABC）を {} 領地で更新しました",
            fixed
        );
    }
    fixed > 0
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

    /// 開発用: `server/data/state.json` があれば起動マイグレーションと同じ処理を適用して保存する
    #[test]
    fn migrate_dev_state_json_if_present() {
        let path = std::path::Path::new("data/state.json");
        if !path.exists() {
            return;
        }
        let raw = std::fs::read_to_string(path).expect("read state.json");
        let mut state: GameState = serde_json::from_str(&raw).expect("parse state.json");
        let changed = migrate_legacy_neutral_enemies(&mut state);
        if !changed {
            return;
        }
        let out = serde_json::to_string_pretty(&state).expect("serialize state");
        std::fs::write(path, out).expect("write state.json");
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
