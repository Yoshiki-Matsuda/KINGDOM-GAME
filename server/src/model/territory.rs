use super::*;

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

/// 攻撃時に「拠点を前線とみなす」オーナーID（自プレイヤー・援軍先・同盟メンバー）
pub(crate) fn attack_base_owner_ids(state: &GameState, acting_player_id: &str) -> Vec<String> {
    let mut ids: Vec<String> = vec![acting_player_id.to_string()];
    if let Some(player) = state.players.get(acting_player_id) {
        for oid in &player.allied_player_ids {
            if !ids.iter().any(|x| x == oid) {
                ids.push(oid.clone());
            }
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
