//! 遠征の移動時間計算（クライアント `travel.ts` と同一式）

use crate::model::parse_territory_coords;

/// 1マスあたりの基準移動時間（秒）
pub const BASE_TRAVEL_TIME_PER_TILE_SEC: f64 = 2.0;
const REF_SPEED: u32 = 5;

fn manhattan_between(a_id: &str, b_id: &str) -> u32 {
    let (ac, ar) = match parse_territory_coords(a_id) {
        Some(v) => v,
        None => return 0,
    };
    let (bc, br) = match parse_territory_coords(b_id) {
        Some(v) => v,
        None => return 0,
    };
    ((ac - bc).abs() + (ar - br).abs()) as u32
}

/// 平均速さから移動時間（ミリ秒）を算出
pub fn travel_time_ms(from_territory_id: &str, to_territory_id: &str, avg_speed: u32) -> u64 {
    let distance = manhattan_between(from_territory_id, to_territory_id);
    if distance == 0 || avg_speed == 0 {
        return 0;
    }
    let sec_per_tile = BASE_TRAVEL_TIME_PER_TILE_SEC * (REF_SPEED as f64 / avg_speed as f64);
    let total_sec = distance as f64 * sec_per_tile;
    total_sec.max(0.0).round() as u64 * 1000
}

/// speed_per_body から平均速さを算出（未指定時は5）
pub fn average_speed(speed_per_body: Option<&[u32]>, count: u32) -> u32 {
    let count = count.max(1) as usize;
    match speed_per_body {
        Some(speeds) if speeds.len() == count => {
            let sum: u32 = speeds.iter().sum();
            (sum / count as u32).max(1)
        }
        _ => REF_SPEED,
    }
}
