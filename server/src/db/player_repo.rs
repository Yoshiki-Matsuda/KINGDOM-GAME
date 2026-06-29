//! プレイヤーデータのDB読み書き

use std::collections::HashMap;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

use crate::model::{
    BuiltFacility, CardStatBonuses, FacilityPosition, MarchKind, MarchMission,
    PlayerData, Resources, StoredFormedUnit, default_now_ms,
};
use crate::items::InventoryItem;
use crate::skills::SkillData;
use crate::cards::CardStats;

/// DBからプレイヤーデータを読み込む
pub(crate) async fn load_player(
    pool: &SqlitePool,
    world_id: &str,
    player_id: &str,
) -> Option<PlayerData> {
    // players基本
    let row = sqlx::query(
        "SELECT * FROM players WHERE world_id = ? AND player_id = ?"
    )
    .bind(world_id)
    .bind(player_id)
    .fetch_optional(pool)
    .await
    .ok()??;

    let mut player = PlayerData {
        player_id: player_id.to_string(),
        home_territory_id: row.get::<String, _>("home_territory_id"),
        inventory: vec![],
        facilities: vec![],
        owned_cards: vec![],
        card_skill_levels: HashMap::new(),
        allied_player_ids: vec![],
        resources: Resources {
            food: row.get::<i64, _>("resource_food") as u64,
            wood: row.get::<i64, _>("resource_wood") as u64,
            stone: row.get::<i64, _>("resource_stone") as u64,
            iron: row.get::<i64, _>("resource_iron") as u64,
            gold: row.get::<i64, _>("resource_gold") as u64,
        },
        last_resource_tick: row.get::<i64, _>("last_resource_tick") as u64,
        last_stamina_tick: row.get::<i64, _>("last_stamina_tick") as u64,
        card_levels: vec![],
        card_exp: vec![],
        card_stamina: vec![],
        card_status_points: vec![],
        card_stat_bonuses: vec![],
        card_rest_until: vec![],
        card_awakened: vec![],
        card_enhanced: vec![],
        card_monster_counts: vec![],
        exploration_level: row.get::<u32, _>("exploration_level"),
        exploration_score: row.get::<i64, _>("exploration_score") as u64,
        unit_cost_cap: row.get::<f32, _>("unit_cost_cap"),
        dungeon_points: row.get::<i64, _>("dungeon_points") as u64,
        charge_points: row.get::<i64, _>("charge_points") as u64,
        marches: vec![],
        formed_units: vec![],
        ai_attack_cooldowns: vec![],
        ai_recover_until: row.get::<i64, _>("ai_recover_until") as u64,
        ai_last_attack_target: row.get::<Option<String>, _>("ai_last_attack_target"),
    };

    // カード
    let card_rows = sqlx::query(
        "SELECT * FROM player_cards WHERE world_id = ? AND player_id = ? ORDER BY slot_index"
    )
    .bind(world_id)
    .bind(player_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for row in &card_rows {
        player.owned_cards.push(row.get::<u32, _>("card_id"));
        player.card_monster_counts.push(row.get::<u32, _>("monster_count"));
        player.card_levels.push(row.get::<u32, _>("level"));
        player.card_exp.push(row.get::<i64, _>("exp") as u64);
        player.card_stamina.push(row.get::<u32, _>("stamina"));
        player.card_status_points.push(row.get::<u32, _>("status_points"));
        player.card_stat_bonuses.push(CardStatBonuses {
            speed: row.get::<u32, _>("bonus_speed"),
            attack: row.get::<u32, _>("bonus_attack"),
            intelligence: row.get::<u32, _>("bonus_intelligence"),
            defense: row.get::<u32, _>("bonus_defense"),
            magic_defense: row.get::<u32, _>("bonus_magic_defense"),
        });
        player.card_rest_until.push(row.get::<i64, _>("rest_until") as u64);
        player.card_awakened.push(row.get::<bool, _>("awakened"));
        player.card_enhanced.push(row.get::<bool, _>("enhanced"));
    }

    // スキルレベル
    let skill_rows = sqlx::query(
        "SELECT * FROM player_card_skill_levels WHERE world_id = ? AND player_id = ?"
    )
    .bind(world_id)
    .bind(player_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for row in &skill_rows {
        let idx = row.get::<u32, _>("slot_index") as usize;
        let s0 = row.get::<u8, _>("skill_0");
        let s1 = row.get::<u8, _>("skill_1");
        let s2 = row.get::<u8, _>("skill_2");
        player.card_skill_levels.insert(idx, [s0, s1, s2]);
    }

    // インベントリ
    let inv_rows = sqlx::query(
        "SELECT item_id, count FROM player_inventory WHERE world_id = ? AND player_id = ?"
    )
    .bind(world_id)
    .bind(player_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    player.inventory = inv_rows
        .iter()
        .map(|r| InventoryItem {
            item_id: r.get::<String, _>("item_id"),
            count: r.get::<u32, _>("count"),
        })
        .collect();

    // 施設
    let fac_rows = sqlx::query(
        "SELECT * FROM player_facilities WHERE world_id = ? AND player_id = ? ORDER BY idx"
    )
    .bind(world_id)
    .bind(player_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    player.facilities = fac_rows
        .iter()
        .map(|r| {
            let pos_col = r.get::<Option<u8>, _>("pos_col");
            let pos_row = r.get::<Option<u8>, _>("pos_row");
            let position = match (pos_col, pos_row) {
                (Some(c), Some(r)) => Some(FacilityPosition { col: c, row: r }),
                _ => None,
            };
            BuiltFacility {
                facility_id: r.get::<String, _>("facility_id"),
                level: r.get::<u8, _>("level"),
                build_complete_at: r.get::<Option<i64>, _>("build_complete_at").map(|v| v as u64),
                position,
            }
        })
        .collect();

    // 同盟関係
    let allied_rows = sqlx::query(
        "SELECT allied_id FROM player_allied_ids WHERE world_id = ? AND player_id = ?"
    )
    .bind(world_id)
    .bind(player_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    player.allied_player_ids = allied_rows
        .iter()
        .map(|r| r.get::<String, _>("allied_id"))
        .collect();

    // AIクールダウン
    let cd_rows = sqlx::query(
        "SELECT territory_id, expire_at FROM player_ai_cooldowns WHERE world_id = ? AND player_id = ?"
    )
    .bind(world_id)
    .bind(player_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    player.ai_attack_cooldowns = cd_rows
        .iter()
        .map(|r| (r.get::<String, _>("territory_id"), r.get::<u64, _>("expire_at")))
        .collect();

    // 遠征
    let march_rows = sqlx::query(
        "SELECT * FROM player_marches WHERE world_id = ? AND player_id = ?"
    )
    .bind(world_id)
    .bind(player_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // march_bodies をバッチ取得（N+1 回避）
    let march_ids: Vec<String> = march_rows.iter().map(|r| r.get::<String, _>("march_id")).collect();
    let all_body_rows = if !march_ids.is_empty() {
        let placeholders: Vec<&str> = march_ids.iter().map(|_| "?").collect();
        let sql = format!(
            "SELECT * FROM player_march_bodies WHERE world_id = ? AND player_id = ? AND march_id IN ({}) ORDER BY body_index",
            placeholders.join(",")
        );
        let mut q = sqlx::query(&sql).bind(world_id).bind(player_id);
        for mid in &march_ids {
            q = q.bind(mid);
        }
        q.fetch_all(pool).await.unwrap_or_default()
    } else {
        vec![]
    };

    // march_id → body rows のマップを構築
    let mut body_map: HashMap<String, Vec<&sqlx::sqlite::SqliteRow>> = HashMap::new();
    for brow in &all_body_rows {
        let mid: String = brow.get("march_id");
        body_map.entry(mid).or_default().push(brow);
    }

    for mrow in &march_rows {
        let march_id: String = mrow.get("march_id");
        let kind_str: String = mrow.get("kind");
        let kind = match kind_str.as_str() {
            "attack" => MarchKind::Attack,
            "deploy" => MarchKind::Deploy,
            "explore" => MarchKind::Explore,
            "return" => MarchKind::Return,
            _ => continue,
        };

        let body_rows = body_map.get(&march_id).map(|v| v.as_slice()).unwrap_or(&[]);

        let mut monsters_per_body = Vec::new();
        let mut body_names = Vec::new();
        let mut speed_per_body = Vec::new();
        let mut skills_per_body: Vec<SkillData> = Vec::new();
        let mut stats_per_body: Vec<CardStats> = Vec::new();
        let has_bodies = !body_rows.is_empty();

        for brow in body_rows {
            if let Some(mc) = brow.get::<Option<u32>, _>("monster_count") {
                monsters_per_body.push(mc);
            }
            if let Some(bn) = brow.get::<Option<String>, _>("body_name") {
                body_names.push(bn);
            }
            if let Some(sp) = brow.get::<Option<u32>, _>("speed") {
                speed_per_body.push(sp);
            }
            if let Some(sk_json) = brow.get::<Option<serde_json::Value>, _>("skills") {
                let sk: SkillData = serde_json::from_value(sk_json).unwrap_or_default();
                skills_per_body.push(sk);
            }
            if let Some(st_json) = brow.get::<Option<serde_json::Value>, _>("stats") {
                let st: CardStats = serde_json::from_value(st_json).unwrap_or(CardStats {
                    monster_count: 1, speed: 0, attack: 0, intelligence: 0,
                    defense: 0, magic_defense: 0, range: 1, cost: 1.0,
                    occupation_power: 1,
                });
                stats_per_body.push(st);
            }
        }

        let oci_json: Option<serde_json::Value> = mrow.get("owned_card_indices");
        let owned_card_indices: Option<Vec<usize>> = oci_json
            .and_then(|v| serde_json::from_value(v).ok());

        player.marches.push(MarchMission {
            march_id,
            kind,
            from_territory_id: mrow.get::<String, _>("from_territory_id"),
            to_territory_id: mrow.get::<String, _>("to_territory_id"),
            started_at: mrow.get::<i64, _>("started_at") as u64,
            arrives_at: mrow.get::<i64, _>("arrives_at") as u64,
            count: mrow.get::<u32, _>("count"),
            monsters_per_body: if has_bodies { Some(monsters_per_body) } else { None },
            body_names: if has_bodies { Some(body_names) } else { None },
            unit_name: mrow.get::<Option<String>, _>("unit_name"),
            speed_per_body: if has_bodies { Some(speed_per_body) } else { None },
            skills_per_body: if has_bodies { Some(skills_per_body) } else { None },
            stats_per_body: if has_bodies { Some(stats_per_body) } else { None },
            owned_card_indices,
            formed_unit_id: mrow.get::<Option<String>, _>("formed_unit_id"),
        });
    }

    // ユニット編成
    let unit_rows = sqlx::query(
        "SELECT * FROM player_formed_units WHERE world_id = ? AND player_id = ?"
    )
    .bind(world_id)
    .bind(player_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    player.formed_units = unit_rows
        .iter()
        .map(|r| StoredFormedUnit {
            id: r.get::<String, _>("unit_id"),
            name: r.get::<String, _>("unit_name"),
            indices: [
                r.get::<i32, _>("slot_0"),
                r.get::<i32, _>("slot_1"),
                r.get::<i32, _>("slot_2"),
            ],
        })
        .collect();

    Some(player)
}

/// 全プレイヤーをDBから読み込む
pub(crate) async fn load_all_players(
    pool: &SqlitePool,
    world_id: &str,
) -> HashMap<String, PlayerData> {
    let player_ids: Vec<String> = sqlx::query(
        "SELECT player_id FROM players WHERE world_id = ?"
    )
    .bind(world_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .iter()
    .map(|r| r.get::<String, _>("player_id"))
    .collect();

    let mut map = HashMap::new();
    for pid in player_ids {
        if let Some(p) = load_player(pool, world_id, &pid).await {
            map.insert(pid, p);
        }
    }
    map
}

/// プレイヤーデータをDBに保存（トランザクション内で呼び出す）
pub(crate) async fn save_player(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    world_id: &str,
    player: &PlayerData,
) -> Result<(), sqlx::Error> {
    let _now = default_now_ms();

    // players基本
    sqlx::query(
        "REPLACE INTO players (world_id, player_id, home_territory_id, \
         resource_food, resource_wood, resource_stone, resource_iron, resource_gold, \
         last_resource_tick, last_stamina_tick, exploration_level, exploration_score, \
         unit_cost_cap, dungeon_points, charge_points, ai_recover_until, ai_last_attack_target) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(world_id)
    .bind(&player.player_id)
    .bind(&player.home_territory_id)
    .bind(player.resources.food as i64)
    .bind(player.resources.wood as i64)
    .bind(player.resources.stone as i64)
    .bind(player.resources.iron as i64)
    .bind(player.resources.gold as i64)
    .bind(player.last_resource_tick as i64)
    .bind(player.last_stamina_tick as i64)
    .bind(player.exploration_level)
    .bind(player.exploration_score as i64)
    .bind(player.unit_cost_cap)
    .bind(player.dungeon_points as i64)
    .bind(player.charge_points as i64)
    .bind(player.ai_recover_until as i64)
    .bind(&player.ai_last_attack_target)
    .execute(&mut **tx)
    .await?;

    // 既存の子データを削除
    sqlx::query("DELETE FROM player_cards WHERE world_id = ? AND player_id = ?")
        .bind(world_id).bind(&player.player_id).execute(&mut **tx).await?;
    sqlx::query("DELETE FROM player_card_skill_levels WHERE world_id = ? AND player_id = ?")
        .bind(world_id).bind(&player.player_id).execute(&mut **tx).await?;
    sqlx::query("DELETE FROM player_inventory WHERE world_id = ? AND player_id = ?")
        .bind(world_id).bind(&player.player_id).execute(&mut **tx).await?;
    sqlx::query("DELETE FROM player_facilities WHERE world_id = ? AND player_id = ?")
        .bind(world_id).bind(&player.player_id).execute(&mut **tx).await?;
    sqlx::query("DELETE FROM player_allied_ids WHERE world_id = ? AND player_id = ?")
        .bind(world_id).bind(&player.player_id).execute(&mut **tx).await?;
    sqlx::query("DELETE FROM player_ai_cooldowns WHERE world_id = ? AND player_id = ?")
        .bind(world_id).bind(&player.player_id).execute(&mut **tx).await?;
    sqlx::query("DELETE FROM player_marches WHERE world_id = ? AND player_id = ?")
        .bind(world_id).bind(&player.player_id).execute(&mut **tx).await?;
    sqlx::query("DELETE FROM player_formed_units WHERE world_id = ? AND player_id = ?")
        .bind(world_id).bind(&player.player_id).execute(&mut **tx).await?;

    // カード
    for (i, card_id) in player.owned_cards.iter().enumerate() {
        let mc = player.card_monster_counts.get(i).copied().unwrap_or(1);
        let lv = player.card_levels.get(i).copied().unwrap_or(1);
        let exp = player.card_exp.get(i).copied().unwrap_or(0);
        let sta = player.card_stamina.get(i).copied().unwrap_or(100);
        let sp = player.card_status_points.get(i).copied().unwrap_or(0);
        let bonus = player.card_stat_bonuses.get(i).cloned().unwrap_or_default();
        let rest = player.card_rest_until.get(i).copied().unwrap_or(0);
        let aw = player.card_awakened.get(i).copied().unwrap_or(false);
        let en = player.card_enhanced.get(i).copied().unwrap_or(false);

        sqlx::query(
            "INSERT INTO player_cards (world_id, player_id, slot_index, card_id, \
             monster_count, level, exp, stamina, status_points, \
             bonus_speed, bonus_attack, bonus_intelligence, bonus_defense, bonus_magic_defense, \
             rest_until, awakened, enhanced) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"
        )
        .bind(world_id).bind(&player.player_id).bind(i as u32).bind(*card_id)
        .bind(mc).bind(lv).bind(exp as i64).bind(sta).bind(sp)
        .bind(bonus.speed).bind(bonus.attack).bind(bonus.intelligence)
        .bind(bonus.defense).bind(bonus.magic_defense)
        .bind(rest as i64).bind(aw).bind(en)
        .execute(&mut **tx)
        .await?;
    }

    // スキルレベル
    for (&slot, levels) in &player.card_skill_levels {
        sqlx::query(
            "INSERT INTO player_card_skill_levels (world_id, player_id, slot_index, skill_0, skill_1, skill_2) VALUES (?,?,?,?,?,?)"
        )
        .bind(world_id).bind(&player.player_id).bind(slot as u32)
        .bind(levels[0]).bind(levels[1]).bind(levels[2])
        .execute(&mut **tx)
        .await?;
    }

    // インベントリ
    for item in &player.inventory {
        sqlx::query(
            "INSERT INTO player_inventory (world_id, player_id, item_id, count) VALUES (?,?,?,?)"
        )
        .bind(world_id).bind(&player.player_id).bind(&item.item_id).bind(item.count)
        .execute(&mut **tx)
        .await?;
    }

    // 施設
    for (i, fac) in player.facilities.iter().enumerate() {
        let (pos_col, pos_row) = match fac.position {
            Some(p) => (Some(p.col), Some(p.row)),
            None => (None, None),
        };
        sqlx::query(
            "INSERT INTO player_facilities (world_id, player_id, idx, facility_id, level, build_complete_at, pos_col, pos_row) VALUES (?,?,?,?,?,?,?,?)"
        )
        .bind(world_id).bind(&player.player_id).bind(i as u32)
        .bind(&fac.facility_id).bind(fac.level).bind(fac.build_complete_at.map(|v| v as i64))
        .bind(pos_col).bind(pos_row)
        .execute(&mut **tx)
        .await?;
    }

    // 同盟関係
    for aid in &player.allied_player_ids {
        sqlx::query(
            "INSERT INTO player_allied_ids (world_id, player_id, allied_id) VALUES (?,?,?)"
        )
        .bind(world_id).bind(&player.player_id).bind(aid)
        .execute(&mut **tx)
        .await?;
    }

    // AIクールダウン
    for (tid, expire) in &player.ai_attack_cooldowns {
        sqlx::query(
            "INSERT INTO player_ai_cooldowns (world_id, player_id, territory_id, expire_at) VALUES (?,?,?,?)"
        )
        .bind(world_id).bind(&player.player_id).bind(tid).bind(*expire as i64)
        .execute(&mut **tx)
        .await?;
    }

    // 遠征
    for march in &player.marches {
        let kind_str = match march.kind {
            MarchKind::Attack => "attack",
            MarchKind::Deploy => "deploy",
            MarchKind::Explore => "explore",
            MarchKind::Return => "return",
        };
        let oci_json = march.owned_card_indices.as_ref()
            .map(|v| serde_json::to_value(v).unwrap_or(serde_json::Value::Null));

        sqlx::query(
            "INSERT INTO player_marches (world_id, player_id, march_id, kind, from_territory_id, \
             to_territory_id, started_at, arrives_at, count, unit_name, owned_card_indices, formed_unit_id) \
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?)"
        )
        .bind(world_id).bind(&player.player_id).bind(&march.march_id)
        .bind(kind_str).bind(&march.from_territory_id).bind(&march.to_territory_id)
        .bind(march.started_at as i64).bind(march.arrives_at as i64).bind(march.count)
        .bind(&march.unit_name).bind(&oci_json).bind(&march.formed_unit_id)
        .execute(&mut **tx)
        .await?;

        // body詳細
        let max_bodies = march.monsters_per_body.as_ref().map(|v| v.len())
            .unwrap_or(0)
            .max(march.body_names.as_ref().map(|v| v.len()).unwrap_or(0))
            .max(march.speed_per_body.as_ref().map(|v| v.len()).unwrap_or(0))
            .max(march.skills_per_body.as_ref().map(|v| v.len()).unwrap_or(0))
            .max(march.stats_per_body.as_ref().map(|v| v.len()).unwrap_or(0));

        for bi in 0..max_bodies {
            let mc = march.monsters_per_body.as_ref().and_then(|v| v.get(bi).copied());
            let bn = march.body_names.as_ref().and_then(|v| v.get(bi).cloned());
            let sp = march.speed_per_body.as_ref().and_then(|v| v.get(bi).copied());
            let sk = march.skills_per_body.as_ref().and_then(|v| v.get(bi))
                .map(|s| serde_json::to_value(s).unwrap_or(serde_json::Value::Null));
            let st = march.stats_per_body.as_ref().and_then(|v| v.get(bi))
                .map(|s| serde_json::to_value(s).unwrap_or(serde_json::Value::Null));

            sqlx::query(
                "INSERT INTO player_march_bodies (world_id, player_id, march_id, body_index, \
                 monster_count, body_name, speed, skills, stats) VALUES (?,?,?,?,?,?,?,?,?)"
            )
            .bind(world_id).bind(&player.player_id).bind(&march.march_id)
            .bind(bi as u32).bind(mc).bind(&bn).bind(sp).bind(&sk).bind(&st)
            .execute(&mut **tx)
            .await?;
        }
    }

    // ユニット編成
    for unit in &player.formed_units {
        sqlx::query(
            "INSERT INTO player_formed_units (world_id, player_id, unit_id, unit_name, slot_0, slot_1, slot_2) VALUES (?,?,?,?,?,?,?)"
        )
        .bind(world_id).bind(&player.player_id).bind(&unit.id)
        .bind(&unit.name).bind(unit.indices[0]).bind(unit.indices[1]).bind(unit.indices[2])
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

/// 全プレイヤーを保存
pub(crate) async fn save_all_players(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    world_id: &str,
    players: &HashMap<String, PlayerData>,
) -> Result<(), sqlx::Error> {
    // 既存プレイヤーを削除（子テーブルはCASCADEで消える）
    sqlx::query("DELETE FROM players WHERE world_id = ?")
        .bind(world_id)
        .execute(&mut **tx)
        .await?;

    for player in players.values() {
        save_player(tx, world_id, player).await?;
    }
    Ok(())
}
