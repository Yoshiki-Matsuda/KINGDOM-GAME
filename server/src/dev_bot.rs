//! 開発用敵プレイヤーBOT — 既存の HTTP 認証 + WebSocket 行動のみを使う（専用APIなし）

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::cards::{get_card, get_card_skills};
use crate::config;
use crate::model::{
    attack_base_owner_ids, is_attackable_target, parse_territory_id, territories_are_adjacent,
    Action, GameState, Territory,
};
use crate::model_actions::STAMINA_ATTACK;

#[derive(Debug, Deserialize)]
struct AuthResponse {
    token: String,
}

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Clone)]
pub(crate) struct DevBotConfig {
    pub http_origin: String,
    pub ws_url: String,
    pub username: String,
    pub password: String,
    pub target_player: String,
    pub interval: Duration,
}

pub(crate) fn is_enabled() -> bool {
    config::dev_bot_enabled()
}

impl DevBotConfig {
    pub fn from_env(port: u16) -> Option<Self> {
        if !is_enabled() {
            return None;
        }
        let username = config::env_string(
            config::ENV_DEV_BOT_USERNAME,
            config::DEFAULT_DEV_BOT_USERNAME,
        );
        let password = std::env::var(config::ENV_DEV_BOT_PASSWORD)
            .or_else(|_| std::env::var(config::ENV_DEV_AUTH_PASSWORD))
            .unwrap_or_else(|_| config::DEFAULT_DEV_AUTH_PASSWORD.to_string());
        let target_player = config::env_string(
            config::ENV_DEV_BOT_TARGET,
            config::DEFAULT_DEV_BOT_TARGET,
        );
        let interval_secs = config::dev_bot_interval_secs();
        let http_origin = config::env_string(
            config::ENV_DEV_BOT_HTTP_ORIGIN,
            &format!("http://127.0.0.1:{port}"),
        );
        let ws_url = config::env_string(
            config::ENV_DEV_BOT_WS_URL,
            &format!("ws://127.0.0.1:{port}/ws"),
        );
        Some(Self {
            http_origin,
            ws_url,
            username,
            password,
            target_player,
            interval: Duration::from_secs(interval_secs),
        })
    }
}

pub(crate) async fn run(config: DevBotConfig) {
    println!(
        "[dev-bot] 起動: user={} target={} interval={}s",
        config.username,
        config.target_player,
        config.interval.as_secs()
    );
    loop {
        match run_once(&config).await {
            Ok(()) => println!("[dev-bot] ターン完了"),
            Err(e) => eprintln!("[dev-bot] エラー: {e}"),
        }
        tokio::time::sleep(config.interval).await;
    }
}

async fn run_once(config: &DevBotConfig) -> Result<(), String> {
    let token = login(config).await?;
    let (mut ws, _) = connect_async(&config.ws_url)
        .await
        .map_err(|e| format!("WebSocket接続失敗: {e}"))?;

    let auth = serde_json::json!({ "type": "auth", "token": token }).to_string();
    ws.send(Message::Text(auth))
        .await
        .map_err(|e| format!("認証送信失敗: {e}"))?;

    let state = recv_game_state(&mut ws, Duration::from_secs(15)).await?;
    let bot_id = config.username.clone();

    let Some(action) = plan_attack(&state, &bot_id, &config.target_player) else {
        println!("[dev-bot] 攻撃可能なマスがありません");
        let _ = ws.close(None).await;
        return Ok(());
    };

    let action_json = serde_json::to_string(&action).map_err(|e| e.to_string())?;
    println!("[dev-bot] 送信: {action_json}");
    ws.send(Message::Text(action_json))
        .await
        .map_err(|e| format!("行動送信失敗: {e}"))?;

    let _ = recv_game_state(&mut ws, Duration::from_secs(180)).await?;
    let _ = ws.close(None).await;
    Ok(())
}

async fn login(config: &DevBotConfig) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/auth/login", config.http_origin);
    let response = client
        .post(url)
        .json(&serde_json::json!({
            "username": config.username,
            "password": config.password,
        }))
        .send()
        .await
        .map_err(|e| format!("ログイン通信失敗: {e}"))?;
    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("ログイン失敗: {body}"));
    }
    let auth: AuthResponse = response
        .json()
        .await
        .map_err(|e| format!("ログイン応答の解析失敗: {e}"))?;
    Ok(auth.token)
}

async fn recv_game_state(ws: &mut WsStream, timeout: Duration) -> Result<GameState, String> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return Err("状態受信タイムアウト".to_string());
        }
        let msg = tokio::time::timeout(remaining, ws.next())
            .await
            .map_err(|_| "状態受信タイムアウト".to_string())?
            .ok_or_else(|| "WebSocketが切断されました".to_string())?
            .map_err(|e| format!("WebSocket受信エラー: {e}"))?;
        let Message::Text(text) = msg else {
            continue;
        };
        if let Ok(state) = serde_json::from_str::<GameState>(&text) {
            if !state.territories.is_empty() {
                return Ok(state);
            }
        }
    }
}

fn manhattan_distance(a_id: &str, b_id: &str) -> Option<u16> {
    let (ac, ar) = parse_territory_id(a_id)?;
    let (bc, br) = parse_territory_id(b_id)?;
    Some(
        (ac as i16 - bc as i16).unsigned_abs() as u16
            + (ar as i16 - br as i16).unsigned_abs() as u16,
    )
}

fn min_distance_to_targets(to_id: &str, target_territory_ids: &[String]) -> u16 {
    target_territory_ids
        .iter()
        .filter_map(|target| manhattan_distance(to_id, target))
        .min()
        .unwrap_or(u16::MAX)
}

/// 攻撃行動を組み立てる（同一クライアントが送る JSON と同じ形式）
pub(crate) fn plan_attack(
    state: &GameState,
    bot_player_id: &str,
    target_player_id: &str,
) -> Option<Action> {
    let player = state.players.get(bot_player_id)?;
    let bot_owned: Vec<&Territory> = state
        .territories
        .iter()
        .filter(|t| t.owner_id.as_deref() == Some(bot_player_id))
        .collect();
    if bot_owned.is_empty() {
        return None;
    }

    let target_territories: Vec<String> = state
        .territories
        .iter()
        .filter(|t| t.owner_id.as_deref() == Some(target_player_id))
        .map(|t| t.id.clone())
        .collect();

    let base_owners = attack_base_owner_ids(state, bot_player_id);
    let mut candidates: Vec<(&Territory, &Territory, u8)> = Vec::new();
    for from in &bot_owned {
        for to in &state.territories {
            if !territories_are_adjacent(&from.id, &to.id) {
                continue;
            }
            if !is_attackable_target(&state.territories, &to.id, &base_owners) {
                continue;
            }
            if to.owner_id.as_deref() == Some(bot_player_id) {
                continue;
            }
            let priority = match to.owner_id.as_deref() {
                Some(owner) if owner == target_player_id => 0,
                None => 1,
                Some(_) => 2,
            };
            candidates.push((from, to, priority));
        }
    }
    candidates.sort_by(|(_, to_a, pri_a), (_, to_b, pri_b)| {
        pri_a
            .cmp(pri_b)
            .then_with(|| {
                min_distance_to_targets(to_a.id.as_str(), &target_territories)
                    .cmp(&min_distance_to_targets(to_b.id.as_str(), &target_territories))
            })
    });
    let (from, to, _) = candidates.first().copied()?;

    let formation = build_home_expedition(player, 3)?;
    if formation.count == 0 {
        return None;
    }

    Some(Action::Attack {
        from_territory_id: from.id.clone(),
        to_territory_id: to.id.clone(),
        count: formation.count,
        monsters_per_body: Some(formation.monster_counts),
        body_names: Some(formation.body_names),
        unit_name: Some("BOTユニット1".to_string()),
        speed_per_body: Some(formation.speeds),
        skills_per_body: Some(formation.skills),
        stats_per_body: Some(formation.stats),
        owned_card_indices: Some(formation.card_indices),
    })
}

struct BotFormation {
    count: u32,
    card_indices: Vec<usize>,
    monster_counts: Vec<u32>,
    body_names: Vec<String>,
    speeds: Vec<u32>,
    skills: Vec<crate::skills::SkillData>,
    stats: Vec<crate::cards::CardStats>,
}

fn build_home_expedition(
    player: &crate::model::PlayerData,
    max_bodies: usize,
) -> Option<BotFormation> {
    let mut card_indices = Vec::new();
    let mut monster_counts = Vec::new();
    let mut body_names = Vec::new();
    let mut speeds = Vec::new();
    let mut skills = Vec::new();
    let mut stats = Vec::new();
    let mut seen_cards = std::collections::HashSet::new();

    for (idx, &card_id) in player.owned_cards.iter().enumerate() {
        if card_indices.len() >= max_bodies {
            break;
        }
        if !seen_cards.insert(card_id) {
            continue;
        }
        let stamina = player.card_stamina.get(idx).copied().unwrap_or(config::max_card_stamina());
        if stamina < STAMINA_ATTACK {
            continue;
        }
        let card = get_card(card_id)?;
        let mc = player
            .card_monster_counts
            .get(idx)
            .copied()
            .unwrap_or(card.stats.monster_count)
            .max(1);
        card_indices.push(idx);
        monster_counts.push(mc);
        body_names.push(card.name.to_string());
        speeds.push(card.stats.speed);
        skills.push(get_card_skills(card_id));
        stats.push(card.stats.clone());
    }

    if card_indices.is_empty() {
        return None;
    }
    Some(BotFormation {
        count: card_indices.len() as u32,
        card_indices,
        monster_counts,
        body_names,
        speeds,
        skills,
        stats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ensure_player_in_game, GameState};

    #[test]
    fn plan_attack_prefers_target_player_territory() {
        let mut state = GameState::default();
        ensure_player_in_game(&mut state, "player").expect("first test player");
        ensure_player_in_game(&mut state, "offline_test").expect("second test player");
        let bot_home = state
            .players
            .get("player")
            .unwrap()
            .home_territory_id
            .clone();
        let human_home = state
            .players
            .get("offline_test")
            .unwrap()
            .home_territory_id
            .clone();
        if let Some(t) = state.territories.iter_mut().find(|t| t.id == human_home) {
            t.owner_id = Some("offline_test".to_string());
        }
        if let Some(t) = state.territories.iter_mut().find(|t| t.id == bot_home) {
            t.owner_id = Some("player".to_string());
        }
        let action = plan_attack(&state, "player", "offline_test");
        if territories_are_adjacent(&bot_home, &human_home) {
            let action = action.expect("adjacent homes should yield attack");
            if let Action::Attack { to_territory_id, .. } = action {
                assert_eq!(to_territory_id, human_home);
            } else {
                panic!("expected attack");
            }
        }
    }
}
