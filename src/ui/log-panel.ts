/**
 * 戦闘履歴パネル（グループ化された履歴表示）
 */

import { gameState } from "../store";
import { escapeHtml } from "../utils";

let historyList: HTMLDivElement;
let detailModal: HTMLDivElement;
let battleTimestamps = new Map<number, Date>();

/** ログの種類を判定 */
type LogType = 
  | "skill_passive" | "skill_active" | "skill_unique" | "skill_effect"
  | "attack" | "defeat" | "damage" | "heal" | "status"
  | "battle_start" | "battle_end" | "normal";

/** 戦闘履歴 */
interface BattleHistory {
  id: number;
  title: string;
  timestamp: Date;
  result: "victory" | "defeat" | "ongoing";
  actions: ActionGroup[];
}

/** 行動グループ */
interface ActionGroup {
  type: "skill" | "attack" | "phase" | "result";
  title: string;
  icon: string;
  lines: string[];
}

function classifyLog(line: string): LogType {
  if (line.startsWith("◆◆") || line.includes("固有スキル")) return "skill_unique";
  if (line.startsWith("◆") || line.startsWith("★")) return "skill_passive";
  if (line.includes("が発動")) return "skill_active";
  if (line.startsWith("  →")) return "skill_effect";
  // 個別攻撃: 「1位目: 〇〇が△△に攻撃」または「〇〇が△△に攻撃」
  if (line.includes("に攻撃") || line.includes("攻撃！")) return "attack";
  if (line.includes("撃破") || line.includes("倒れた")) return "defeat";
  if (line.includes("ダメージ")) return "damage";
  if (line.includes("回復") || line.includes("吸収")) return "heal";
  if (line.includes("毒") || line.includes("炎上") || line.includes("凍結") || 
      line.includes("気絶") || line.includes("沈黙") || line.includes("シールド") ||
      line.includes("無敵") || line.includes("バフ") || line.includes("マーク")) return "status";
  // 戦闘開始（新形式・旧形式両対応）
  if (line.startsWith("【") && line.includes("侵攻戦】")) return "battle_start";
  // 「ユニットXが〇〇を攻撃しました」形式 = 戦闘開始
  if (/ユニット\d+が.+を攻撃しました/.test(line) || line.includes("へ侵攻開始")) return "battle_start";
  // 戦闘終了
  if (line.includes("占領しました") || line.includes("攻撃失敗") || line.includes("防衛に成功")) return "battle_end";
  return "normal";
}

function getLogIcon(type: LogType): string {
  switch (type) {
    case "skill_unique": return "💎";
    case "skill_passive": return "✨";
    case "skill_active": return "⚡";
    case "skill_effect": return "";
    case "attack": return "⚔️";
    case "defeat": return "💀";
    case "damage": return "💥";
    case "heal": return "💚";
    case "status": return "🔮";
    case "battle_start": return "🏁";
    case "battle_end": return "🏆";
    default: return "•";
  }
}

/** ログを戦闘履歴にパース */
function parseLogsToHistory(logs: string[]): BattleHistory[] {
  const histories: BattleHistory[] = [];
  let currentBattle: BattleHistory | null = null;
  let currentAction: ActionGroup | null = null;
  let battleId = 0;

  for (const line of logs) {
    const type = classifyLog(line);

    // 新しい戦闘の開始
    if (type === "battle_start") {
      if (currentBattle) {
        if (currentAction) currentBattle.actions.push(currentAction);
        histories.push(currentBattle);
      }
      
      // マス名と座標を抽出
      let title = "戦闘";
      
      // 新形式: 【マス名<col,row>侵攻戦】 → 「マス名<col,row>」を抽出
      const newFormatMatch = line.match(/【(.+?<\d+,\d+>)侵攻戦】/);
      if (newFormatMatch) {
        title = newFormatMatch[1];
      } else {
        // 旧形式: 【マス名侵攻戦】
        const oldFormatMatch = line.match(/【(.+?)侵攻戦】/);
        if (oldFormatMatch) {
          title = oldFormatMatch[1];
        } else {
          // 形式: ユニットXが△△を攻撃しました → △△を抽出
          const attackMatch = line.match(/ユニット\d+が(.+?)を攻撃しました/);
          if (attackMatch) {
            title = attackMatch[1];
          } else {
            // 形式: 〇〇へ侵攻開始 → 〇〇を抽出
            const invasionMatch = line.match(/が(.+?)へ侵攻/);
            if (invasionMatch) {
              title = invasionMatch[1];
            }
          }
        }
      }
      
      // 時刻を記録
      if (!battleTimestamps.has(battleId)) {
        battleTimestamps.set(battleId, new Date());
      }
      
      currentBattle = {
        id: battleId++,
        title: title,
        timestamp: battleTimestamps.get(battleId - 1) ?? new Date(),
        result: "ongoing",
        actions: [],
      };
      currentAction = {
        type: "phase",
        title: "侵攻開始",
        icon: "🏁",
        lines: [line],
      };
      continue;
    }

    if (!currentBattle) {
      currentBattle = {
        id: battleId++,
        title: "戦闘",
        timestamp: new Date(),
        result: "ongoing",
        actions: [],
      };
    }

    // 戦闘終了
    if (type === "battle_end") {
      if (currentAction) {
        currentBattle.actions.push(currentAction);
      }
      const isVictory = line.includes("占領しました");
      currentBattle.result = isVictory ? "victory" : "defeat";
      currentAction = {
        type: "result",
        title: isVictory ? "占領成功" : "攻撃失敗",
        icon: isVictory ? "🏆" : "💀",
        lines: [line],
      };
      currentBattle.actions.push(currentAction);
      histories.push(currentBattle);
      currentBattle = null;
      currentAction = null;
      continue;
    }

    // スキル発動（新しいアクショングループ）
    if (type === "skill_passive" || type === "skill_active" || type === "skill_unique") {
      if (currentAction) {
        currentBattle.actions.push(currentAction);
      }
      const skillMatch = line.match(/「(.+?)」/);
      const charMatch = line.match(/^[◆★]*\s*(.+?)の/);
      currentAction = {
        type: "skill",
        title: `${charMatch?.[1] ?? ""}の${skillMatch?.[1] ?? "スキル"}`,
        icon: getLogIcon(type),
        lines: [line],
      };
      continue;
    }

    // 攻撃（新しいアクショングループ）
    if (type === "attack") {
      if (currentAction) {
        currentBattle.actions.push(currentAction);
      }
      const attackMatch = line.match(/(.+?)が(.+?)に攻撃/);
      currentAction = {
        type: "attack",
        title: attackMatch ? `${attackMatch[1]} → ${attackMatch[2]}` : "攻撃",
        icon: "⚔️",
        lines: [line],
      };
      continue;
    }

    // それ以外は現在のアクションに追加
    if (currentAction) {
      currentAction.lines.push(line);
    } else {
      currentAction = {
        type: "phase",
        title: "行動",
        icon: "•",
        lines: [line],
      };
    }
  }

  // 残りを追加
  if (currentBattle) {
    if (currentAction) currentBattle.actions.push(currentAction);
    histories.push(currentBattle);
  }

  return histories;
}

/** 時刻をフォーマット */
function formatTime(date: Date): string {
  const h = date.getHours().toString().padStart(2, "0");
  const m = date.getMinutes().toString().padStart(2, "0");
  const s = date.getSeconds().toString().padStart(2, "0");
  return `${h}:${m}:${s}`;
}

/** 戦闘履歴カードをレンダリング */
function renderHistoryCard(battle: BattleHistory): string {
  const resultClass = battle.result === "victory" ? "history-victory" : 
                      battle.result === "defeat" ? "history-defeat" : "history-ongoing";
  const resultIcon = battle.result === "victory" ? "🏆" : 
                     battle.result === "defeat" ? "💀" : "⚔️";
  
  const timeStr = formatTime(battle.timestamp);
  
  return `
    <div class="history-card ${resultClass}" data-battle-id="${battle.id}">
      <span class="history-card-time">${timeStr}</span>
      <span class="history-card-icon">${resultIcon}</span>
      <span class="history-card-title">${escapeHtml(battle.title)}</span>
    </div>
  `;
}

/** 戦闘詳細モーダルを表示 */
function showBattleDetail(battle: BattleHistory): void {
  const resultClass = battle.result === "victory" ? "detail-victory" : 
                      battle.result === "defeat" ? "detail-defeat" : "detail-ongoing";
  
  detailModal.innerHTML = `
    <div class="detail-overlay" data-close="true">
      <div class="detail-modal ${resultClass}">
        <div class="detail-header">
          <h2 class="detail-title">${escapeHtml(battle.title)}</h2>
          <button class="detail-close" data-close="true">✕</button>
        </div>
        <div class="detail-content">
          ${battle.actions.map((action, idx) => `
            <div class="action-group action-${action.type}" data-action-idx="${idx}">
              <div class="action-header">
                <span class="action-icon">${action.icon}</span>
                <span class="action-title">${escapeHtml(action.title)}</span>
                <span class="action-toggle">▼</span>
              </div>
              <div class="action-body">
                ${action.lines.map(line => {
                  const type = classifyLog(line);
                  const icon = getLogIcon(type);
                  return `<div class="action-line action-line-${type}">
                    ${icon ? `<span class="line-icon">${icon}</span>` : ""}
                    <span class="line-text">${escapeHtml(line)}</span>
                  </div>`;
                }).join("")}
              </div>
            </div>
          `).join("")}
        </div>
      </div>
    </div>
  `;
  
  detailModal.classList.add("is-open");
  
  // アクショングループの折りたたみ
  detailModal.querySelectorAll(".action-group").forEach(group => {
    const header = group.querySelector(".action-header");
    header?.addEventListener("click", () => {
      group.classList.toggle("is-collapsed");
    });
  });
  
  // モーダルを閉じる
  detailModal.querySelectorAll("[data-close]").forEach(el => {
    el.addEventListener("click", (e) => {
      if (e.target === el) {
        detailModal.classList.remove("is-open");
      }
    });
  });
}

let battleHistories: BattleHistory[] = [];

export function createLogElement(): HTMLDivElement {
  const wrapper = document.createElement("div");
  wrapper.className = "history-screen";
  wrapper.style.display = "none";
  
  // 詳細モーダル
  detailModal = document.createElement("div");
  detailModal.className = "battle-detail-modal";
  document.body.appendChild(detailModal);
  
  // ヘッダー
  const header = document.createElement("div");
  header.className = "history-header";
  header.innerHTML = `
    <div class="history-title">
      <span class="history-title-icon">📜</span>
      <span>戦闘履歴</span>
    </div>
  `;
  
  // 履歴リスト
  historyList = document.createElement("div");
  historyList.className = "history-list";
  
  wrapper.appendChild(header);
  wrapper.appendChild(historyList);
  
  // カードクリックイベント
  historyList.addEventListener("click", (e) => {
    const card = (e.target as HTMLElement).closest(".history-card");
    if (card) {
      const battleId = parseInt(card.getAttribute("data-battle-id") ?? "0");
      const battle = battleHistories.find(b => b.id === battleId);
      if (battle) {
        showBattleDetail(battle);
      }
    }
  });
  
  return wrapper;
}

export function renderLog(): void {
  const logs = gameState.log ?? [];
  
  // 履歴をパース
  battleHistories = parseLogsToHistory(logs);
  
  // 履歴リスト描画（新しい順）
  historyList.innerHTML = battleHistories.slice().reverse().map(renderHistoryCard).join("");
}
