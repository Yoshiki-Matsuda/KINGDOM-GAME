//! ワールド（ゲーム状態全体）のDB読み書き

use sqlx::sqlite::SqlitePool;
use sqlx::Row;

use crate::model::{
    AiFaction, AiPersonality, Alliance, GameEvent, GameState, MarketItemType, MarketListing,
    SeasonInfo, Territory, WorldConfig,
};
use crate::ruins::RuinInfo;
use super::player_repo;

/// ワールドIDを決定（PVPは固定、PVEはプレイヤーID）
pub(crate) fn pvp_world_id() -> &'static str {
    "pvp_shared"
}

pub(crate) fn pve_world_id(player_id: &str) -> String {
    format!("pve_{player_id}")
}

/// DBからGameStateを再構築
pub(crate) async fn load_world(pool: &SqlitePool, world_id: &str) -> Option<GameState> {
    let row = sqlx::query("SELECT * FROM worlds WHERE id = ?")
        .bind(world_id)
        .fetch_optional(pool)
        .await
        .ok()??;

    let world = WorldConfig {
        cols: row.get::<u16, _>("cols"),
        rows: row.get::<u16, _>("rows"),
        home_col: row.get::<u16, _>("home_col"),
        home_row: row.get::<u16, _>("home_row"),
        terrain_seed: row.get::<i64, _>("terrain_seed") as u64,
    };

    let world_owner_id: Option<String> = row.get("world_owner_id");

    let season = SeasonInfo {
        season_number: row.get::<u32, _>("season_number"),
        started_at: row.get::<i64, _>("season_started_at") as u64,
        duration_ms: row.get::<i64, _>("season_duration_ms") as u64,
    };

    // AI勢力
    let faction_rows = sqlx::query(
        "SELECT * FROM world_ai_factions WHERE world_id = ?"
    )
    .bind(world_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let ai_factions: Vec<AiFaction> = faction_rows
        .iter()
        .map(|r| {
            let personality_str: String = r.get("personality");
            let personality = match personality_str.as_str() {
                "aggressive" => AiPersonality::Aggressive,
                "defensive" => AiPersonality::Defensive,
                _ => AiPersonality::Balanced,
            };
            AiFaction {
                faction_id: r.get::<String, _>("faction_id"),
                name: r.get::<String, _>("name"),
                personality,
                home_territory_id: r.get::<String, _>("home_territory_id"),
                color: r.get::<u32, _>("color"),
            }
        })
        .collect();

    // 領地
    let territory_rows = sqlx::query(
        "SELECT * FROM territories WHERE world_id = ? ORDER BY id"
    )
    .bind(world_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let territories: Vec<Territory> = territory_rows
        .iter()
        .map(|r| {
            let ruin_json: Option<serde_json::Value> = r.get("ruin");
            let ruin: Option<RuinInfo> = ruin_json.and_then(|v| serde_json::from_value(v).ok());

            let body_mc: Option<serde_json::Value> = r.get("body_monster_counts");
            let body_monster_counts: Option<Vec<u32>> = body_mc.and_then(|v| serde_json::from_value(v).ok());

            let body_names_json: Option<serde_json::Value> = r.get("body_names");
            let body_names: Option<Vec<String>> = body_names_json.and_then(|v| serde_json::from_value(v).ok());

            Territory {
                id: r.get::<String, _>("id"),
                name: r.get::<String, _>("name"),
                level: r.get::<u8, _>("level"),
                owner_id: r.get::<Option<String>, _>("owner_id"),
                troops: r.get::<u32, _>("troops"),
                body_monster_counts,
                body_names,
                ruin,
                is_base: r.get::<bool, _>("is_base"),
                durability: r.get::<u32, _>("durability"),
                max_durability: r.get::<u32, _>("max_durability"),
                tower_level: r.get::<u8, _>("tower_level"),
            }
        })
        .collect();

    // プレイヤー
    let players = player_repo::load_all_players(pool, world_id).await;

    // 同盟
    let alliance_rows = sqlx::query(
        "SELECT * FROM alliances WHERE world_id = ?"
    )
    .bind(world_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut alliances = Vec::new();
    for arow in &alliance_rows {
        let alliance_id: String = arow.get("alliance_id");

        let member_rows = sqlx::query(
            "SELECT player_id FROM alliance_members WHERE world_id = ? AND alliance_id = ?"
        )
        .bind(world_id)
        .bind(&alliance_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let member_ids: Vec<String> = member_rows.iter().map(|r| r.get::<String, _>("player_id")).collect();

        let child_rows = sqlx::query(
            "SELECT child_alliance_id FROM alliance_children WHERE world_id = ? AND alliance_id = ?"
        )
        .bind(world_id)
        .bind(&alliance_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let child_alliance_ids: Vec<String> = child_rows.iter()
            .map(|r| r.get::<String, _>("child_alliance_id"))
            .collect();

        alliances.push(Alliance {
            id: alliance_id,
            name: arow.get::<String, _>("name"),
            leader_id: arow.get::<String, _>("leader_id"),
            member_ids,
            territory_points: arow.get::<i64, _>("territory_points") as u64,
            level: arow.get::<u32, _>("level"),
            donated_total: arow.get::<i64, _>("donated_total") as u64,
            parent_alliance_id: arow.get::<Option<String>, _>("parent_alliance_id"),
            child_alliance_ids,
        });
    }

    // マーケット
    let listing_rows = sqlx::query(
        "SELECT * FROM market_listings WHERE world_id = ?"
    )
    .bind(world_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let market_listings: Vec<MarketListing> = listing_rows
        .iter()
        .map(|r| {
            let item_type_str: String = r.get("item_type");
            let item = match item_type_str.as_str() {
                "card" => MarketItemType::Card {
                    card_id: r.get::<Option<u32>, _>("item_card_id").unwrap_or(0),
                },
                "item" => MarketItemType::Item {
                    item_id: r.get::<Option<String>, _>("item_item_id").unwrap_or_default(),
                    count: r.get::<Option<u32>, _>("item_count").unwrap_or(0),
                },
                "resource" => MarketItemType::Resource {
                    resource_type: r.get::<Option<String>, _>("item_resource_type").unwrap_or_default(),
                    amount: r.get::<Option<i64>, _>("item_amount").map(|v| v as u64).unwrap_or(0),
                },
                _ => MarketItemType::Resource {
                    resource_type: String::new(),
                    amount: 0,
                },
            };
            MarketListing {
                listing_id: r.get::<String, _>("listing_id"),
                seller_id: r.get::<String, _>("seller_id"),
                item,
                price: r.get::<i64, _>("price") as u64,
                listed_at: r.get::<i64, _>("listed_at") as u64,
            }
        })
        .collect();

    // ログ
    let log_rows = sqlx::query(
        "SELECT id, timestamp, actor_id, event_type, data, message FROM event_logs WHERE world_id = ? ORDER BY id"
    )
    .bind(world_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let log: Vec<GameEvent> = log_rows.iter().map(|r| {
        let data_str = r.get::<String, _>("data");
        GameEvent {
            id: r.get::<i64, _>("id") as u64,
            timestamp: r.get::<i64, _>("timestamp") as u64,
            actor_id: r.get::<Option<String>, _>("actor_id"),
            event_type: r.get::<String, _>("event_type"),
            data: serde_json::from_str(&data_str).unwrap_or(serde_json::json!({})),
            message: r.get::<String, _>("message"),
        }
    }).collect();

    Some(GameState {
        world,
        world_owner_id,
        ai_factions,
        territories,
        log,
        players,
        alliances,
        season,
        market_listings,
        visible_marches: vec![],
    })
}

/// GameStateをDBに保存（トランザクション）
pub(crate) async fn save_world(
    pool: &SqlitePool,
    world_id: &str,
    mode: &str,
    state: &GameState,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // worlds
    sqlx::query(
        "REPLACE INTO worlds (id, mode, world_owner_id, cols, rows, home_col, home_row, \
         terrain_seed, season_number, season_started_at, season_duration_ms) \
         VALUES (?,?,?,?,?,?,?,?,?,?,?)"
    )
    .bind(world_id)
    .bind(mode)
    .bind(&state.world_owner_id)
    .bind(state.world.cols)
    .bind(state.world.rows)
    .bind(state.world.home_col)
    .bind(state.world.home_row)
    .bind(state.world.terrain_seed as i64)
    .bind(state.season.season_number)
    .bind(state.season.started_at as i64)
    .bind(state.season.duration_ms as i64)
    .execute(&mut *tx)
    .await?;

    // AI勢力
    sqlx::query("DELETE FROM world_ai_factions WHERE world_id = ?")
        .bind(world_id)
        .execute(&mut *tx)
        .await?;

    for f in &state.ai_factions {
        let personality_str = match f.personality {
            AiPersonality::Aggressive => "aggressive",
            AiPersonality::Balanced => "balanced",
            AiPersonality::Defensive => "defensive",
        };
        sqlx::query(
            "INSERT INTO world_ai_factions (world_id, faction_id, name, personality, home_territory_id, color) VALUES (?,?,?,?,?,?)"
        )
        .bind(world_id).bind(&f.faction_id).bind(&f.name)
        .bind(personality_str).bind(&f.home_territory_id).bind(f.color)
        .execute(&mut *tx)
        .await?;
    }

    // 領地
    sqlx::query("DELETE FROM territories WHERE world_id = ?")
        .bind(world_id)
        .execute(&mut *tx)
        .await?;

    for t in &state.territories {
        let ruin_json = t.ruin.as_ref()
            .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null));
        let body_mc_json = t.body_monster_counts.as_ref()
            .map(|v| serde_json::to_value(v).unwrap_or(serde_json::Value::Null));
        let body_names_json = t.body_names.as_ref()
            .map(|v| serde_json::to_value(v).unwrap_or(serde_json::Value::Null));

        sqlx::query(
            "INSERT INTO territories (world_id, id, name, level, owner_id, troops, \
             body_monster_counts, body_names, ruin, is_base, durability, max_durability, tower_level) \
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)"
        )
        .bind(world_id).bind(&t.id).bind(&t.name).bind(t.level)
        .bind(&t.owner_id).bind(t.troops)
        .bind(&body_mc_json).bind(&body_names_json).bind(&ruin_json)
        .bind(t.is_base).bind(t.durability).bind(t.max_durability).bind(t.tower_level)
        .execute(&mut *tx)
        .await?;
    }

    // プレイヤー
    player_repo::save_all_players(&mut tx, world_id, &state.players).await?;

    // 同盟
    sqlx::query("DELETE FROM alliances WHERE world_id = ?")
        .bind(world_id)
        .execute(&mut *tx)
        .await?;

    for a in &state.alliances {
        sqlx::query(
            "INSERT INTO alliances (world_id, alliance_id, name, leader_id, territory_points, level, donated_total, parent_alliance_id) VALUES (?,?,?,?,?,?,?,?)"
        )
        .bind(world_id).bind(&a.id).bind(&a.name).bind(&a.leader_id)
        .bind(a.territory_points as i64).bind(a.level).bind(a.donated_total as i64)
        .bind(&a.parent_alliance_id)
        .execute(&mut *tx)
        .await?;

        for mid in &a.member_ids {
            sqlx::query(
                "INSERT INTO alliance_members (world_id, alliance_id, player_id) VALUES (?,?,?)"
            )
            .bind(world_id).bind(&a.id).bind(mid)
            .execute(&mut *tx)
            .await?;
        }

        for cid in &a.child_alliance_ids {
            sqlx::query(
                "INSERT INTO alliance_children (world_id, alliance_id, child_alliance_id) VALUES (?,?,?)"
            )
            .bind(world_id).bind(&a.id).bind(cid)
            .execute(&mut *tx)
            .await?;
        }
    }

    // マーケット
    sqlx::query("DELETE FROM market_listings WHERE world_id = ?")
        .bind(world_id)
        .execute(&mut *tx)
        .await?;

    for l in &state.market_listings {
        let (item_type, card_id, item_id, item_count, res_type, amount) = match &l.item {
            MarketItemType::Card { card_id } => ("card", Some(*card_id), None, None, None, None),
            MarketItemType::Item { item_id, count } => ("item", None, Some(item_id.as_str()), Some(*count), None, None),
            MarketItemType::Resource { resource_type, amount } => ("resource", None, None, None, Some(resource_type.as_str()), Some(*amount as i64)),
        };
        sqlx::query(
            "INSERT INTO market_listings (world_id, listing_id, seller_id, item_type, \
             item_card_id, item_item_id, item_count, item_resource_type, item_amount, price, listed_at) \
             VALUES (?,?,?,?,?,?,?,?,?,?,?)"
        )
        .bind(world_id).bind(&l.listing_id).bind(&l.seller_id)
        .bind(item_type).bind(card_id).bind(item_id).bind(item_count)
        .bind(res_type).bind(amount)
        .bind(l.price as i64).bind(l.listed_at as i64)
        .execute(&mut *tx)
        .await?;
    }

    // ログ
    sqlx::query("DELETE FROM event_logs WHERE world_id = ?")
        .bind(world_id)
        .execute(&mut *tx)
        .await?;

    for ev in &state.log {
        let data_str = serde_json::to_string(&ev.data).unwrap_or_else(|_| "{}".to_string());
        sqlx::query(
            "INSERT INTO event_logs (world_id, id, timestamp, actor_id, event_type, data, message) VALUES (?,?,?,?,?,?,?)"
        )
        .bind(world_id)
        .bind(ev.id as i64)
        .bind(ev.timestamp as i64)
        .bind(&ev.actor_id)
        .bind(&ev.event_type)
        .bind(&data_str)
        .bind(&ev.message)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// ワールドを削除（ワイプ用）
pub(crate) async fn delete_world(pool: &SqlitePool, world_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM worlds WHERE id = ?")
        .bind(world_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// PVEワールド一覧を取得
pub(crate) async fn list_pve_world_ids(pool: &SqlitePool) -> Vec<String> {
    sqlx::query("SELECT id FROM worlds WHERE mode = 'pve'")
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|r| {
            let id: String = r.get("id");
            // "pve_" プレフィックスを除去してプレイヤーIDを返す
            id.strip_prefix("pve_").unwrap_or(&id).to_string()
        })
        .collect()
}

/// ワールドが存在するか
#[allow(dead_code)]
pub(crate) async fn world_exists(pool: &SqlitePool, world_id: &str) -> bool {
    let row = sqlx::query("SELECT COUNT(*) as cnt FROM worlds WHERE id = ?")
        .bind(world_id)
        .fetch_one(pool)
        .await;
    match row {
        Ok(r) => r.get::<i64, _>("cnt") > 0,
        Err(_) => false,
    }
}
