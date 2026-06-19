use std::collections::HashMap;

use rand::seq::SliceRandom;

use crate::model::{
    apply_home_safe_zone_levels, can_place_home_with_safe_zone, default_dev_inventory,
    default_owned_cards, generate_territories, initial_card_monster_counts_for_owned,
    sync_home_territory_body_counts_from_player, AiFaction, AiPersonality, GameState, PlayerData,
    Resources, SeasonInfo, Territory, WorldConfig,
};
use crate::config;

const BASE_FACTION_COUNT: u32 = 5;
const MIN_FACTION_COUNT: u32 = 2;

static FACTION_NAMES: &[&str] = &[
    "北方連合",
    "東方王国",
    "西方諸侯",
    "南方帝国",
    "中央軍団",
    "辺境伯領",
    "蒼天騎士団",
    "紅蓮氏族",
];

static FACTION_COLORS: &[u32] = &[
    0xE74C3C, 0x3498DB, 0x2ECC71, 0xF39C12, 0x9B59B6, 0x1ABC9C, 0xE67E22, 0x34495E,
];

pub fn ai_faction_count(cols: u16, rows: u16) -> u32 {
    let area = cols as u32 * rows as u32;
    let raw = (BASE_FACTION_COUNT as f64 * area as f64 / config::default_world_area() as f64).round() as u32;
    let capped = raw.max(MIN_FACTION_COUNT);
    config::ai_faction_max_cap()
        .map(|max| capped.min(max))
        .unwrap_or(capped)
}

fn manhattan(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn pick_next_faction_home(
    world: &WorldConfig,
    territories: &[Territory],
    player_home: (i32, i32),
    placed: &[(u16, u16)],
    ai_id: &str,
) -> Option<(u16, u16)> {
    let cols = world.cols as i32;
    let rows = world.rows as i32;
    let min_sep = ((cols.min(rows) / 4).max(6)) as i32;

    let mut candidates: Vec<(u16, u16)> = Vec::new();
    for row in 2..(rows - 2) {
        for col in 2..(cols - 2) {
            if col == player_home.0 && row == player_home.1 {
                continue;
            }
            if manhattan((col, row), player_home) < min_sep {
                continue;
            }
            if placed.iter().any(|&(pc, pr)| {
                manhattan((pc as i32, pr as i32), (col, row)) < min_sep / 2
            }) {
                continue;
            }
            let id = format!("c_{}_{}", col, row);
            if can_place_home_with_safe_zone(territories, &id, ai_id, world) {
                candidates.push((col as u16, row as u16));
            }
        }
    }
    candidates.shuffle(&mut rand::thread_rng());
    candidates.into_iter().next()
}

fn personality_for_index(i: usize) -> AiPersonality {
    match i % 3 {
        0 => AiPersonality::Aggressive,
        1 => AiPersonality::Balanced,
        _ => AiPersonality::Defensive,
    }
}

fn setup_ai_player(faction_id: &str, home_territory_id: String) -> PlayerData {
    let ai_id = format!("ai_{faction_id}");
    let mut player = PlayerData::new(ai_id, home_territory_id);
    player.owned_cards = default_owned_cards();
    let n = player.owned_cards.len();
    player.card_monster_counts = initial_card_monster_counts_for_owned(&player.owned_cards);
    player.card_stamina = vec![config::max_card_stamina(); n];
    player.card_levels = vec![5; n];
    player.card_exp = vec![0; n];
    player.card_status_points = vec![0; n];
    player.card_stat_bonuses = vec![crate::model::CardStatBonuses::default(); n];
    player.card_rest_until = vec![0; n];
    player.card_awakened = vec![false; n];
    player.card_enhanced = vec![false; n];
    player.resources = Resources {
        food: 2000,
        wood: 2000,
        stone: 1500,
        iron: 1000,
        gold: 500,
    };
    player.inventory = default_dev_inventory();
    player.exploration_level = 3;
    crate::ai_actions::initialize_ai_formed_units(&mut player);
    player
}

fn occupy_ai_home(
    territories: &mut [Territory],
    col: u16,
    row: u16,
    ai_id: &str,
    player: &PlayerData,
) {
    let id = format!("c_{}_{}", col, row);
    let Some(idx) = territories.iter().position(|t| t.id == id) else {
        return;
    };
    territories[idx].owner_id = Some(ai_id.to_string());
    territories[idx].is_base = true;
    territories[idx].troops = player.owned_cards.len() as u32;
    territories[idx].body_monster_counts = Some(player.card_monster_counts.clone());
    territories[idx].body_names = None;
}

pub fn new_pve(player_id: &str, mut world: WorldConfig) -> GameState {
    if world.terrain_seed == 0 {
        world.terrain_seed = crate::model::resolve_terrain_seed(None);
    }
    let home_territory_id = format!("c_{}_{}", world.home_col, world.home_row);
    let player = PlayerData::new(player_id.to_string(), home_territory_id.clone());
    let mut territories = generate_territories(&world, player_id, Some(player_id));
    let mut players = HashMap::new();
    players.insert(player_id.to_string(), player);
    if let Some(p) = players.get(player_id) {
        sync_home_territory_body_counts_from_player(&mut territories, p);
    }

    let player_home = (world.home_col as i32, world.home_row as i32);
    let faction_count = ai_faction_count(world.cols, world.rows);
    let mut placed_homes: Vec<(u16, u16)> = Vec::new();
    let mut ai_factions = Vec::new();

    for i in 0..faction_count as usize {
        let faction_id = format!("faction_{i}");
        let ai_id = format!("ai_{faction_id}");
        let Some((col, row)) =
            pick_next_faction_home(&world, &territories, player_home, &placed_homes, &ai_id)
        else {
            break;
        };
        let home_id = format!("c_{}_{}", col, row);
        let name = FACTION_NAMES[i % FACTION_NAMES.len()].to_string();
        let color = FACTION_COLORS[i % FACTION_COLORS.len()];
        let ai_player = setup_ai_player(&faction_id, home_id.clone());
        occupy_ai_home(&mut territories, col, row, &ai_id, &ai_player);
        sync_home_territory_body_counts_from_player(&mut territories, &ai_player);
        apply_home_safe_zone_levels(&mut territories, col, row, &world);

        players.insert(ai_id.clone(), ai_player);
        ai_factions.push(AiFaction {
            faction_id,
            name,
            personality: personality_for_index(i),
            home_territory_id: home_id,
            color,
        });
        placed_homes.push((col, row));
    }

    let ai_count = ai_factions.len();
    GameState {
        world,
        world_owner_id: Some(player_id.to_string()),
        ai_factions,
        territories,
        log: vec![crate::model::GameEvent {
            id: 1,
            timestamp: crate::model::default_now_ms(),
            actor_id: None,
            event_type: "system".to_string(),
            data: serde_json::json!({}),
            message: format!("PVEワールドを生成しました。AI勢力: {ai_count}体"),
        }],
        players,
        alliances: vec![],
        season: SeasonInfo::default(),
        market_listings: vec![],
        visible_marches: vec![],
    }
}

pub fn is_ai_player_id(player_id: &str) -> bool {
    player_id.starts_with("ai_")
}

pub fn is_human_player_id(player_id: &str) -> bool {
    !is_ai_player_id(player_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pve_ai_neighbors_stay_neutral() {
        let world = WorldConfig {
            cols: 24,
            rows: 24,
            home_col: 12,
            home_row: 12,
            terrain_seed: 7,
        };
        let state = new_pve("human", world);
        for faction in &state.ai_factions {
            let (hc, hr) = {
                let id = &faction.home_territory_id;
                let col: u16 = id
                    .strip_prefix("c_")
                    .and_then(|s| s.split('_').next())
                    .and_then(|s| s.parse().ok())
                    .unwrap();
                let row: u16 = id
                    .split('_')
                    .nth(2)
                    .and_then(|s| s.parse().ok())
                    .unwrap();
                (col as i32, row as i32)
            };
            for (dc, dr) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nc = hc + dc;
                let nr = hr + dr;
                if nc < 0 || nr < 0 || nc >= world.cols as i32 || nr >= world.rows as i32 {
                    continue;
                }
                let nid = format!("c_{}_{}", nc, nr);
                let territory = state.territories.iter().find(|t| t.id == nid).unwrap();
                assert!(
                    territory.owner_id.is_none(),
                    "neighbor {nid} of AI home should stay neutral"
                );
            }
        }
    }
}
