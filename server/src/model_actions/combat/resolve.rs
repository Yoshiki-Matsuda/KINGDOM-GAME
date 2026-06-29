use super::*;

/// 攻撃ダメージ適用後の結果処理（処刑・全体攻撃・シールド・撃破・反射・反撃）。
///
/// `teams[0]` は行動側チーム（actor_idx の所属）、`teams[1]` は相手チーム。
/// 魅了・混乱による同士討ちでは `target_team == 0` になる。
/// 死亡スキル（復活等）の判定はプレイヤー側キャラが倒れたときのみ行う（KC準拠）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_attack_outcome(
    teams: &mut [&mut Vec<CombatCharacter>; 2],
    actor_idx: usize,
    target_team: usize,
    target_idx: usize,
    actor_is_player: bool,
    target_team_is_player: bool,
    atk_name: &str,
    def_name: &str,
    net_damage: f32,
    attack_mods: &crate::skills::AttackModifiers,
    log: &mut Vec<GameEvent>,
) {
    // 処刑（残存率が閾値以下なら即死、死亡スキル判定なし）
    let hp_ratio = teams[target_team][target_idx].current_monster_count
        / teams[target_team][target_idx].base_monster_count as f32;
    if attack_mods.execute_threshold > 0.0 && hp_ratio <= attack_mods.execute_threshold {
        teams[target_team][target_idx].is_alive = false;
        teams[target_team][target_idx].current_monster_count = 0.0;
        push_system_event(log, &format!("処刑発動！{}を即死させた！", def_name));
        return;
    }

    // 全体攻撃（固定 + 割合）
    if attack_mods.aoe_damage > 0.0 || attack_mods.aoe_percent_damage > 0.0 {
        let label = if target_team_is_player {
            if actor_is_player { "味方全員" } else { "プレイヤー側全員" }
        } else {
            "敵全員"
        };
        for member in teams[target_team].iter_mut() {
            if !member.is_alive {
                continue;
            }
            let splash = attack_mods.aoe_damage
                + member.current_monster_count * attack_mods.aoe_percent_damage;
            if splash <= 0.0 {
                continue;
            }
            member.current_monster_count -= splash;
            if member.current_monster_count <= 0.0 {
                member.is_alive = false;
                if !(target_team_is_player && check_death_skills(member, log)) {
                    push_system_event(log, &format!("{}が全体攻撃で撃破されました。", member.name));
                }
            }
        }
        push_skill_effect_event(log, &format!(
                "全体攻撃で{}にダメージ！（固定{:.0}{}）",
                label,
                attack_mods.aoe_damage,
                if attack_mods.aoe_percent_damage > 0.0 {
                    format!(" + 各HP{:.1}%", attack_mods.aoe_percent_damage * 100.0)
                } else {
                    String::new()
                }
            ));
    }

    // シールド吸収後に本体ダメージ
    let damage_after_shield = teams[target_team][target_idx].absorb_damage_with_shield(net_damage);
    if damage_after_shield > 0.0 {
        teams[target_team][target_idx].current_monster_count -= damage_after_shield;
    }

    if attack_mods.absorb_rate > 0.0 && damage_after_shield > 0.0 {
        let before = teams[0][actor_idx].current_monster_count;
        let absorb = damage_after_shield * attack_mods.absorb_rate;
        teams[0][actor_idx].current_monster_count += absorb;
        let after = teams[0][actor_idx].current_monster_count;
        push_absorb_event(log, atk_name, absorb, before, after);
    }

    if teams[target_team][target_idx].current_monster_count <= 0.0 {
        teams[target_team][target_idx].is_alive = false;
        let death_logged =
            target_team_is_player && check_death_skills(&mut teams[target_team][target_idx], log);
        if !death_logged {
            push_defeat_event(log, atk_name, def_name, !actor_is_player);
        }

        if attack_mods.monster_steal > 0.0 {
            teams[0][actor_idx].current_monster_count += attack_mods.monster_steal;
            push_system_event(log, &format!("{}が {:.0} 魔獣数を奪取！", atk_name, attack_mods.monster_steal));
        }
        if attack_mods.extra_attacks > 0 {
            teams[0][actor_idx].extra_attacks += attack_mods.extra_attacks;
            push_system_event(log, &format!("{}が追加攻撃権を得た！", atk_name));
        }
    } else if damage_after_shield <= 0.0 {
        push_skill_effect_event(log, &format!("{}のシールドがダメージを吸収！", def_name));
    } else {
        // 反射
        let reflect_rate = teams[target_team][target_idx].get_reflect_rate();
        if reflect_rate > 0.0 {
            let reflect_damage = net_damage * reflect_rate;
            teams[0][actor_idx].current_monster_count -= reflect_damage;
            push_skill_effect_event(log, &format!("{}の反射で {:.0} ダメージ！", def_name, reflect_damage));
            if teams[0][actor_idx].current_monster_count <= 0.0 {
                teams[0][actor_idx].is_alive = false;
                if !(actor_is_player && check_death_skills(&mut teams[0][actor_idx], log)) {
                    push_system_event(log, &format!("{}が反射で撃破されました。", atk_name));
                }
            }
        }
        // 反撃
        let counter_rate = teams[target_team][target_idx].get_counter_rate();
        if counter_rate > 0.0
            && rand::random::<f32>() < counter_rate
            && teams[0][actor_idx].is_alive
        {
            let c_atk = teams[target_team][target_idx].attack as f32
                * teams[target_team][target_idx].attack_buff_multiplier();
            let c_def = teams[0][actor_idx].defense as f32
                * teams[0][actor_idx].defense_buff_multiplier();
            let c_mc = teams[target_team][target_idx].current_monster_count;
            let c_ratio = (c_atk / c_def.max(1.0)).clamp(0.3, 1.1);
            let counter_raw = c_ratio
                * c_mc
                * 0.5
                * teams[target_team][target_idx].outgoing_damage_multiplier();
            let c_min = kc_minimum_damage(c_mc) * 0.5;
            let c_max = c_mc * 0.55;
            let (cl, ch) = if c_min <= c_max {
                (c_min, c_max)
            } else {
                (c_max, c_max)
            };
            let counter_dmg = counter_raw.clamp(cl, ch);
            teams[0][actor_idx].current_monster_count -= counter_dmg;
            push_skill_effect_event(log, &format!("{}の反撃！ {:.0} ダメージ！", def_name, counter_dmg));
            if teams[0][actor_idx].current_monster_count <= 0.0 {
                teams[0][actor_idx].is_alive = false;
                if !(actor_is_player && check_death_skills(&mut teams[0][actor_idx], log)) {
                    push_system_event(log, &format!("{}が反撃で撃破されました。", atk_name));
                }
            }
        }
    }
}

/// 戦闘ターン中の1キャラ分の行動を処理する。
/// 対象選択（魅了・混乱の同士討ち含む）→攻撃スキル→回避/無敵→ダメージ→反射・反撃→追撃効果。
pub(crate) fn perform_actor_turn(
    attacker_team: &mut Vec<CombatCharacter>,
    opponent_team: &mut Vec<CombatCharacter>,
    actor_idx: usize,
    actor_is_player: bool,
    log: &mut Vec<GameEvent>,
) {
    if !attacker_team[actor_idx].is_alive {
        return;
    }

    if attacker_team[actor_idx].is_disabled() {
        push_system_event(log, &format!("{}は行動不能！", attacker_team[actor_idx].name));
        return;
    }

    // 魅了・混乱時は自陣営を狙う
    let target_own_team = {
        let ch = &attacker_team[actor_idx];
        if ch.is_charmed() {
            push_skill_effect_event(log, &format!("{}は魅了され味方を攻撃する！", ch.name));
            true
        } else {
            let p = ch.confused_own_team_chance();
            if p > 0.0 && rand::random::<f32>() < p {
                push_skill_effect_event(log, &format!("{}は混乱して味方を狙う！", ch.name));
                true
            } else {
                false
            }
        }
    };
    let target_team_is_player = if target_own_team { actor_is_player } else { !actor_is_player };

    let target_idx = if target_own_team {
        match find_target_excluding(
            attacker_team[actor_idx].range,
            attacker_team.as_slice(),
            Some(actor_idx),
        ) {
            Some(i) => i,
            None => return,
        }
    } else {
        match find_target(attacker_team[actor_idx].range, opponent_team.as_slice()) {
            Some(i) => i,
            None => return,
        }
    };

    let attack_mods = if attacker_team[actor_idx].is_silenced() {
        push_skill_effect_event(log, &format!("{}は沈黙中でスキル使用不可！", attacker_team[actor_idx].name));
        crate::skills::AttackModifiers::new()
    } else {
        apply_attack_skills(&mut attacker_team[actor_idx], actor_is_player, log)
    };

    // ダメージを伴わないスキル発動はバフ・デバフ付与のみで行動終了
    if attack_mods.skill_activated && !attack_mods_has_skill_damage(&attack_mods) {
        if target_own_team {
            apply_on_attack_skill_followup_one_team(&attack_mods, attacker_team, actor_idx, target_idx, log);
        } else {
            apply_on_attack_skill_followup(&attack_mods, attacker_team, opponent_team, actor_idx, target_idx, log);
        }
        return;
    }

    let atk_name = attacker_team[actor_idx].name.clone();
    let def_name = if target_own_team {
        attacker_team[target_idx].name.clone()
    } else {
        opponent_team[target_idx].name.clone()
    };

    let evasion_rate = if target_own_team {
        attacker_team[target_idx].get_evasion_rate()
    } else {
        opponent_team[target_idx].get_evasion_rate()
    };
    if evasion_rate > 0.0 && rand::random::<f32>() < evasion_rate {
        push_system_event(log, &format!("{}の攻撃を{}が回避！", atk_name, def_name));
        return;
    }

    let blocked_by_invincible = if target_own_team {
        attacker_team[target_idx].consume_invincible()
    } else {
        opponent_team[target_idx].consume_invincible()
    };
    if blocked_by_invincible {
        push_system_event(log, &format!("{}は無敵で攻撃を無効化！", def_name));
        return;
    }

    let net_damage = {
        let a = &attacker_team[actor_idx];
        let d = if target_own_team {
            &attacker_team[target_idx]
        } else {
            &opponent_team[target_idx]
        };
        compute_net_attack_damage(a, d, &attack_mods, log)
    };
    push_attack_event(
        log,
        &atk_name,
        &def_name,
        net_damage,
        crate::skills::side_label(actor_is_player),
    );

    let target_team = if target_own_team { 0 } else { 1 };
    resolve_attack_outcome(
        &mut [&mut *attacker_team, &mut *opponent_team],
        actor_idx,
        target_team,
        target_idx,
        actor_is_player,
        target_team_is_player,
        &atk_name,
        &def_name,
        net_damage,
        &attack_mods,
        log,
    );

    if target_own_team {
        apply_on_attack_skill_followup_one_team(&attack_mods, attacker_team, actor_idx, target_idx, log);
    } else {
        apply_on_attack_skill_followup(&attack_mods, attacker_team, opponent_team, actor_idx, target_idx, log);
    }
}
