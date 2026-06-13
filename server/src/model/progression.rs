use super::*;

/// KC準拠: カードレベル→次のレベルに必要な累積経験値
/// Lv1→Lv2: 100, Lv2→Lv3: 230, ... 緩やかな指数増加（Lv99で最大）
pub fn exp_needed_for_level(current_level: u32) -> u64 {
    let lv = current_level.max(1);
    let base = 100_u64;
    let curve = (lv as f64).powf(1.4);
    (base as f64 * curve).round() as u64
}

/// 累積経験値を元に、現在のレベルを算出して返す（Lvアップが発生した場合 true）
/// 覚醒済み（awakened=true）のときは上限99→上限120まで解放
pub fn process_level_up(
    current_level: &mut u32,
    current_exp: &mut u64,
    status_points: &mut u32,
    awakened: bool,
    log_name: &str,
    log: &mut Vec<String>,
) -> bool {
    let mut leveled = false;
    let cap = if awakened { 120 } else { 99 };
    if *current_level == 0 {
        *current_level = 1;
    }
    loop {
        if *current_level >= cap {
            break;
        }
        let need = exp_needed_for_level(*current_level);
        if *current_exp < need {
            break;
        }
        *current_exp -= need;
        *current_level += 1;
        *status_points = status_points.saturating_add(10);
        leveled = true;
        push_log(
            log,
            format!(
                "「{}」がLv{}にアップ！ステータスポイント+10",
                log_name, *current_level
            ),
        );
    }
    leveled
}
