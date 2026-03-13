use crate::{
    model::{generate_neutral_enemies, GameState},
    ruins::generate_ruin,
};

pub(crate) fn cleanup_expired_ruins(state: &mut GameState) -> bool {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let mut changed = false;
    for territory in &mut state.territories {
        if let Some(ref ruin) = territory.ruin {
            if let Some(expires_at) = ruin.expires_at {
                if now_ms >= expires_at {
                    territory.ruin = None;
                    territory.owner_id = None;
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

pub(crate) fn spawn_random_ruin(state: &mut GameState) -> bool {
    use rand::seq::SliceRandom;

    let candidates: Vec<usize> = state
        .territories
        .iter()
        .enumerate()
        .filter(|(_, territory)| territory.owner_id.is_none() && territory.ruin.is_none())
        .map(|(index, _)| index)
        .collect();

    if candidates.is_empty() {
        return false;
    }

    let mut rng = rand::thread_rng();
    if let Some(&index) = candidates.choose(&mut rng) {
        let territory_id = state.territories[index].id.clone();
        let ruin = generate_ruin(&territory_id);

        let troops = ruin.enemies.len() as u32;
        let body_energies = ruin.enemy_energies.clone();
        let body_names = ruin.enemy_names.clone();

        state.territories[index].ruin = Some(ruin);
        state.territories[index].troops = troops;
        state.territories[index].body_energies = Some(body_energies);
        state.territories[index].body_names = Some(body_names);

        true
    } else {
        false
    }
}

pub(crate) fn count_ruins(state: &GameState) -> usize {
    state.territories.iter().filter(|territory| territory.ruin.is_some()).count()
}
