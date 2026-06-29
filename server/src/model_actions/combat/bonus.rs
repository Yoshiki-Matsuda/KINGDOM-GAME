use super::*;

pub(crate) fn assign_positions(chars: &mut [CombatCharacter]) {
    let len = chars.len();
    if len == 0 {
        return;
    }
    for c in chars.iter_mut() {
        c.position = crate::skills::Position::Front;
    }
    chars[len - 1].position = crate::skills::Position::Leader;
    if len >= 3 {
        chars[len - 2].position = crate::skills::Position::Back;
    }
}

pub(crate) fn position_label(position: crate::skills::Position) -> &'static str {
    match position {
        crate::skills::Position::Front => "FRONT",
        crate::skills::Position::Back => "BACK",
        crate::skills::Position::Leader => "LEADER",
    }
}

pub(crate) fn push_enemy_roster_log(log: &mut Vec<GameEvent>, enemy_chars: &[CombatCharacter]) {
    let enemies: Vec<serde_json::Value> = enemy_chars.iter().map(|ch| {
        serde_json::json!({
            "name": ch.name,
            "position": position_label(ch.position),
            "monster_count": ch.current_monster_count,
            "attack": ch.attack,
            "defense": ch.defense,
            "intelligence": ch.intelligence,
            "magic_defense": ch.magic_defense,
            "speed": ch.current_speed,
        })
    }).collect();
    push_enemy_roster_event(log, enemies);
}

/// ローカル開発用: 攻撃側のダメージ倍率のみ引き上げ（魔獣数は所持値のまま・上限9999準拠）
pub(crate) fn apply_dev_auto_win_boost(our_chars: &mut [CombatCharacter], _enemy_chars: &[CombatCharacter]) {
    for character in our_chars.iter_mut() {
        if !character.is_alive {
            continue;
        }
        character.damage_multiplier *= 10.0;
    }
}

/// 同族ボーナス: 同一種族2体で攻防+5%、3体以上で+10%
pub(crate) fn apply_race_bonus(chars: &mut [CombatCharacter], log: &mut Vec<GameEvent>) {
    use std::collections::HashMap;
    let mut race_counts: HashMap<crate::cards::Race, usize> = HashMap::new();
    for c in chars.iter() {
        if let Some(r) = c.race {
            *race_counts.entry(r).or_insert(0) += 1;
        }
    }
    let mut bonus_races: Vec<(crate::cards::Race, f32)> = Vec::new();
    for (race, count) in &race_counts {
        if *count >= 3 {
            bonus_races.push((*race, 0.10));
        } else if *count >= 2 {
            bonus_races.push((*race, 0.05));
        }
    }
    if bonus_races.is_empty() {
        return;
    }
    for c in chars.iter_mut() {
        if let Some(r) = c.race {
            if let Some((_, frac)) = bonus_races.iter().find(|(br, _)| *br == r) {
                let atk_bonus = (c.attack as f32 * frac).ceil() as u32;
                let def_bonus = (c.defense as f32 * frac).ceil() as u32;
                c.attack += atk_bonus;
                c.defense += def_bonus;
            }
        }
    }
    for (race, frac) in &bonus_races {
        let race_name = match race {
            crate::cards::Race::Beast => "獣族",
            crate::cards::Race::Demihuman => "亜人族",
            crate::cards::Race::Demon => "魔族",
            crate::cards::Race::Dragon => "龍族",
            crate::cards::Race::Giant => "巨人族",
            crate::cards::Race::Spirit => "精霊族",
            crate::cards::Race::Undead => "不死族",
        };
        let pct = (*frac * 100.0) as u32;
        push_system_event(log, &format!("◆ {}の同族ボーナス発動（攻撃・防御+{}%）", race_name, pct));
    }
}

/// 種族研究所Lv ボーナス: 該当種族の魔獣に攻防+1%/Lv（最大+10%）
/// KC仕様: 獣族生態研究所・亜人族行動研究所・不死族解析研究所・精霊族調査研究所・
///         巨人族監視研究所・魔族防衛研究所・龍族探索研究所 等
pub(crate) fn apply_race_lab_bonus(
    chars: &mut [CombatCharacter],
    facilities: &[crate::model::BuiltFacility],
    log: &mut Vec<GameEvent>,
) {
    use crate::cards::Race;
    let now = default_now_ms();
    let mut lab_levels: std::collections::HashMap<Race, u32> = std::collections::HashMap::new();
    for f in facilities {
        if let Some(complete_at) = f.build_complete_at {
            if complete_at > now {
                continue;
            }
        }
        let race = match f.facility_id.as_str() {
            "beast_lab" => Some(Race::Beast),
            "demihuman_lab" => Some(Race::Demihuman),
            "demon_lab" => Some(Race::Demon),
            "dragon_lab" => Some(Race::Dragon),
            "giant_lab" => Some(Race::Giant),
            "spirit_lab" => Some(Race::Spirit),
            "undead_lab" => Some(Race::Undead),
            _ => None,
        };
        if let Some(r) = race {
            let lv = f.level as u32;
            let entry = lab_levels.entry(r).or_insert(0);
            if lv > *entry {
                *entry = lv;
            }
        }
    }
    if lab_levels.is_empty() {
        return;
    }
    for c in chars.iter_mut() {
        if let Some(r) = c.race {
            if let Some(lv) = lab_levels.get(&r).copied() {
                let pct = (lv.min(10) as f32) / 100.0;
                let atk_bonus = (c.attack as f32 * pct).ceil() as u32;
                let def_bonus = (c.defense as f32 * pct).ceil() as u32;
                c.attack += atk_bonus;
                c.defense += def_bonus;
            }
        }
    }
    for (race, lv) in &lab_levels {
        let race_name = match race {
            Race::Beast => "獣族",
            Race::Demihuman => "亜人族",
            Race::Demon => "魔族",
            Race::Dragon => "龍族",
            Race::Giant => "巨人族",
            Race::Spirit => "精霊族",
            Race::Undead => "不死族",
        };
        let pct = lv.min(&10);
        push_system_event(log, &format!("◆ {}研究所Lv{}ボーナス（攻撃・防御+{}%）", race_name, lv, pct));
    }
}

/// 戦神の祠・守護神の祠の攻防ボーナスを戦闘キャラに適用
/// KC仕様: 拠点内の魔獣に攻/防+2%/Lv（戦神→攻撃、守護神→防御）
pub(crate) fn apply_shrine_bonus(
    chars: &mut [CombatCharacter],
    bonuses: &crate::facilities::FacilityBonuses,
    log: &mut Vec<GameEvent>,
) {
    if bonuses.attack_bonus == 0 && bonuses.defense_bonus == 0 {
        return;
    }
    let atk_pct = bonuses.attack_bonus as f32 / 100.0;
    let def_pct = bonuses.defense_bonus as f32 / 100.0;
    for c in chars.iter_mut() {
        if atk_pct > 0.0 {
            let b = (c.attack as f32 * atk_pct).ceil() as u32;
            c.attack += b;
        }
        if def_pct > 0.0 {
            let b = (c.defense as f32 * def_pct).ceil() as u32;
            c.defense += b;
        }
    }
    if bonuses.attack_bonus > 0 {
        push_system_event(log, &format!("◆ 戦神の祠ボーナス（攻撃+{}%）", bonuses.attack_bonus));
    }
    if bonuses.defense_bonus > 0 {
        push_system_event(log, &format!("◆ 守護神の祠ボーナス（防御+{}%）", bonuses.defense_bonus));
    }
}

/// KC準拠: 種族ティア（巨人・魔・龍 > 亜人・不死・精霊 > 獣）
pub(crate) fn race_combat_tier(r: crate::cards::Race) -> u8 {
    use crate::cards::Race::*;
    match r {
        Giant | Demon | Dragon => 2,
        Demihuman | Undead | Spirit => 1,
        Beast => 0,
    }
}

/// 攻撃側が上位ティアほど与ダメージ増、下位なら減（1段階あたり±12%）
pub(crate) fn race_matchup_damage_multiplier(att: Option<crate::cards::Race>, def: Option<crate::cards::Race>) -> f32 {
    match (att, def) {
        (Some(a), Some(d)) => {
            let diff = race_combat_tier(a) as i32 - race_combat_tier(d) as i32;
            1.0 + (diff as f32) * 0.12
        }
        _ => 1.0,
    }
}
