//! ゲームログのタイムスタンプ付与と行数上限（model / skills 共通）

pub(crate) const MAX_LOG_LINES: usize = 2000;

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub(crate) fn push_log(log: &mut Vec<String>, line: String) {
    let ts = now_ms();
    log.push(format!("[ts:{}]{}", ts, line));
    if log.len() > MAX_LOG_LINES {
        log.drain(0..log.len() - MAX_LOG_LINES);
    }
}

/// 行動主体プレイヤー付きログ（クライアントが他プレイヤー/AIのログを戦歴から除外する）
pub(crate) fn push_actor_log(log: &mut Vec<String>, actor_player_id: &str, line: String) {
    push_log(log, format!("[p:{actor_player_id}]{line}"));
}

pub(crate) fn extract_log_ts(line: &str) -> Option<u64> {
    let rest = line.strip_prefix("[ts:")?;
    let num = rest.split(']').next()?;
    num.parse().ok()
}

/// タイムスタンプ未付与行を直前行の時刻で埋める（再起動後の戦闘ログ分割を防ぐ）
pub(crate) fn migrate_log_timestamps(log: &mut [String]) {
    let mut last_ts: Option<u64> = None;
    let mut migrated = 0usize;
    for line in log.iter_mut() {
        if let Some(ts) = extract_log_ts(line) {
            last_ts = Some(ts);
            continue;
        }
        let ts = last_ts.unwrap_or_else(now_ms);
        *line = format!("[ts:{}]{}", ts, line);
        last_ts = Some(ts);
        migrated += 1;
    }
    if migrated > 0 {
        println!("[kingdom-server] タイムスタンプ未付与のログ {migrated} 件を補正しました");
    }
}
