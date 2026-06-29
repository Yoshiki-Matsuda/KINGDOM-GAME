//! 構造化ゲームイベント（旧テキストログの後継）

use serde::{Deserialize, Serialize};

/// ログ上限（旧 MAX_LOG_LINES 相当）
pub(crate) const MAX_EVENTS: usize = 2000;

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// ゲームイベント（構造化ログ）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub id: u64,
    pub timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    pub event_type: String,
    pub data: serde_json::Value,
    pub message: String,
}

static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn push(log: &mut Vec<GameEvent>, ev: GameEvent) {
    log.push(ev);
    if log.len() > MAX_EVENTS {
        log.drain(0..log.len() - MAX_EVENTS);
    }
}

fn push_new(log: &mut Vec<GameEvent>, actor_id: Option<&str>, event_type: &str, data: serde_json::Value, message: String) {
    push(log, GameEvent {
        id: next_id(),
        timestamp: now_ms(),
        actor_id: actor_id.map(|s| s.to_string()),
        event_type: event_type.to_string(),
        data,
        message,
    });
}

// ---------------------------------------------------------------------------
// 汎用
// ---------------------------------------------------------------------------

/// 旧 push_log 相当。段階的移行用の汎用システムイベント。
pub fn push_system_event(log: &mut Vec<GameEvent>, message: &str) {
    push_new(log, None, "system", serde_json::json!({}), message.to_string());
}

/// 旧 push_actor_log 相当。段階的移行用の汎用アクターイベント。
pub fn push_actor_system_event(log: &mut Vec<GameEvent>, actor_id: &str, message: &str) {
    push_new(log, Some(actor_id), "system", serde_json::json!({}), message.to_string());
}

// ---------------------------------------------------------------------------
// 戦闘
// ---------------------------------------------------------------------------

pub fn push_battle_start_event(
    log: &mut Vec<GameEvent>,
    actor_id: &str,
    territory_name: &str,
    defender_label: &str,
    coords: &str,
    attacker_label: &str,
) {
    let msg = format!(
        "【{territory_name}{coords}侵攻戦】{attacker_label}が{territory_name}（{defender_label}）へ侵攻開始"
    );
    push_new(log, Some(actor_id), "battle_start", serde_json::json!({
        "territory_name": territory_name,
        "defender_label": defender_label,
        "attacker_label": attacker_label,
        "coords": coords,
    }), msg);
}

pub fn push_battle_end_event(
    log: &mut Vec<GameEvent>,
    actor_id: &str,
    result: &str,
    territory_name: &str,
) {
    let msg = match result {
        "victory" => format!("{territory_name}を占領しました！"),
        "practice_victory" => format!("{territory_name}の演習戦に勝利した！"),
        "practice_defeat" => format!("演習戦に敗北した。{territory_name}の防衛に成功。"),
        "defeat" => format!("攻撃失敗。{territory_name}の防衛に成功。"),
        "partial" => format!("敵を撃破したが{territory_name}は耐久が残り、占領には至らなかった。"),
        "timeout" => "8ターン経過。防衛側の勝利。".to_string(),
        _ => result.to_string(),
    };
    push_new(log, Some(actor_id), "battle_end", serde_json::json!({
        "result": result,
        "territory_name": territory_name,
    }), msg);
}

pub fn push_attack_event(
    log: &mut Vec<GameEvent>,
    attacker: &str,
    target: &str,
    damage: f32,
    side: &str,
) {
    let msg = format!("[{side}] {attacker}が{target}に攻撃！（{damage:.0} ダメージ）");
    push_new(log, None, "attack", serde_json::json!({
        "attacker": attacker,
        "target": target,
        "damage": damage,
        "side": side,
    }), msg);
}

pub fn push_defeat_event(
    log: &mut Vec<GameEvent>,
    attacker: &str,
    target: &str,
    actor_is_player: bool,
) {
    let msg = if !actor_is_player {
        format!("{target}が撃破されました。")
    } else {
        format!("{attacker}が{target}を撃破しました。")
    };
    push_new(log, None, "defeat", serde_json::json!({
        "attacker": attacker,
        "target": target,
    }), msg);
}

pub fn push_absorb_event(
    log: &mut Vec<GameEvent>,
    attacker: &str,
    absorb: f32,
    before: f32,
    after: f32,
) {
    let msg = format!("{attacker}が {absorb:.0} 魔獣数を吸収！（{before:.0} → {after:.0}）");
    push_new(log, None, "absorb", serde_json::json!({
        "attacker": attacker,
        "absorb": absorb,
        "before": before,
        "after": after,
    }), msg);
}

pub fn push_enemy_roster_event(
    log: &mut Vec<GameEvent>,
    enemies: Vec<serde_json::Value>,
) {
    push_new(log, None, "enemy_roster", serde_json::json!({ "enemies": enemies }), "--- 敵編成 ---".to_string());
}

pub fn push_phase_event(
    log: &mut Vec<GameEvent>,
    phase: &str,
) {
    push_new(log, None, "phase", serde_json::json!({ "phase": phase }), format!("--- {phase} ---"));
}

// ---------------------------------------------------------------------------
// スキル
// ---------------------------------------------------------------------------

pub fn push_skill_event(
    log: &mut Vec<GameEvent>,
    side: &str,
    char_name: &str,
    skill_name: &str,
    is_unique: bool,
    prefix: &str,
) {
    let label = if is_unique { "固有スキル" } else { "" };
    let msg = format!("{prefix} {side} {char_name}の{label}「{skill_name}」が発動！");
    let et = if is_unique { "skill_unique" } else { "skill_active" };
    push_new(log, None, et, serde_json::json!({
        "character": char_name,
        "skill": skill_name,
        "side": side,
    }), msg);
}

pub fn push_skill_effect_event(log: &mut Vec<GameEvent>, message: &str) {
    push_new(log, None, "skill_effect", serde_json::json!({}), message.to_string());
}

// ---------------------------------------------------------------------------
// 戦利品・報酬
// ---------------------------------------------------------------------------

pub fn push_loot_gold_event(log: &mut Vec<GameEvent>, actor_id: &str, amount: u32) {
    push_new(log, Some(actor_id), "loot_gold", serde_json::json!({ "amount": amount }), format!("ゴールド+{amount} を入手！"));
}

pub fn push_loot_item_event(log: &mut Vec<GameEvent>, actor_id: &str, item_id: &str, item_name: &str, count: u32) {
    push_new(log, Some(actor_id), "loot_item", serde_json::json!({
        "item_id": item_id, "item_name": item_name, "count": count,
    }), format!("{item_name}x{count} を入手！"));
}

pub fn push_card_drop_event(log: &mut Vec<GameEvent>, actor_id: &str, card_name: &str) {
    push_new(log, Some(actor_id), "card_drop", serde_json::json!({ "card_name": card_name }), format!("魔獣「{card_name}」を入手！"));
}

pub fn push_conquest_event(log: &mut Vec<GameEvent>, actor_id: &str, territory_name: &str) {
    push_new(log, Some(actor_id), "conquest", serde_json::json!({ "territory_name": territory_name }), format!("{territory_name}を占領しました！"));
}

pub fn push_conquest_reward_event(log: &mut Vec<GameEvent>, actor_id: &str, food: u64, wood: u64, stone: u64, iron: u64) {
    push_new(log, Some(actor_id), "conquest_reward", serde_json::json!({
        "food": food, "wood": wood, "stone": stone, "iron": iron,
    }), format!("占領報酬: 食料+{food}・木+{wood}・石+{stone}・鉄+{iron}"));
}

pub fn push_ruin_clear_event(log: &mut Vec<GameEvent>, actor_id: &str) {
    push_new(log, Some(actor_id), "ruin_clear", serde_json::json!({}), "遺跡を攻略しました！".to_string());
}

// ---------------------------------------------------------------------------
// レベルアップ
// ---------------------------------------------------------------------------

pub fn push_level_up_event(log: &mut Vec<GameEvent>, name: &str, level: u32) {
    push_new(log, None, "level_up", serde_json::json!({ "name": name, "level": level }), format!("{name}がLv{level}に上がった！"));
}

// ---------------------------------------------------------------------------
// 探索
// ---------------------------------------------------------------------------

pub fn push_explore_dispatch_event(log: &mut Vec<GameEvent>, actor_id: &str, territory_name: &str, message: &str) {
    push_new(log, Some(actor_id), "explore_dispatch", serde_json::json!({ "territory_name": territory_name }), message.to_string());
}

pub fn push_explore_complete_event(log: &mut Vec<GameEvent>, actor_id: &str, territory_name: &str, food: u64, wood: u64, stone: u64, iron: u64) {
    push_new(log, Some(actor_id), "explore_complete", serde_json::json!({
        "territory_name": territory_name, "food": food, "wood": wood, "stone": stone, "iron": iron,
    }), format!("{territory_name}の探索が完了。食料+{food}・木+{wood}・石+{stone}・鉄+{iron}"));
}

pub fn push_explore_level_up_event(log: &mut Vec<GameEvent>, actor_id: &str, message: &str) {
    push_new(log, Some(actor_id), "explore_level_up", serde_json::json!({}), message.to_string());
}

// ---------------------------------------------------------------------------
// 同盟
// ---------------------------------------------------------------------------

pub fn push_alliance_event(log: &mut Vec<GameEvent>, message: &str) {
    push_new(log, None, "alliance", serde_json::json!({}), message.to_string());
}
