/**
 * 戦闘履歴パネル（グループ化された履歴表示）
 */

import { getBodyDisplayName } from "../game/characters";
import { getPlayerOwnedCards } from "../shared/game-state";
import { gameState, getLocalPlayerId } from "../store";
import { escapeHtml } from "../utils";

let historyList: HTMLDivElement;
let detailModal: HTMLDivElement;

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
  side?: "ally" | "enemy";
}

function detectLogSide(line: string): "ally" | "enemy" | undefined {
  if (line.includes("[味方]")) return "ally";
  if (line.includes("[敵]")) return "enemy";
  return undefined;
}

function buildPlayerUnitNames(): Set<string> {
  const names = new Set<string>();
  for (const cardIdx of getPlayerOwnedCards(gameState, getLocalPlayerId())) {
    names.add(getBodyDisplayName(cardIdx));
  }
  return names;
}

function parseSkillCharacterName(line: string): string {
  const m = line.match(/^[◆★]*\s*(?:\[味方\]\s*|\[敵\]\s*)?(.+?)の/);
  return m?.[1] ?? "";
}

function parseAttackAttackerName(line: string): string {
  const m = line.match(/^(?:\[味方\]\s*|\[敵\]\s*)?(.+?)が.+?に攻撃/);
  return m?.[1] ?? "";
}

function resolveActionSide(
  line: string,
  actorName: string,
  characterSides: Map<string, "ally" | "enemy">,
  playerUnitNames: Set<string>,
): "ally" | "enemy" | undefined {
  const tagged = detectLogSide(line);
  if (tagged) {
    if (actorName) characterSides.set(actorName, tagged);
    return tagged;
  }
  if (actorName) {
    const known = characterSides.get(actorName);
    if (known) return known;
    if (actorName.startsWith("味方ユニット")) return "ally";
    if (actorName.startsWith("敵ユニット")) return "enemy";
    if (playerUnitNames.has(actorName)) return "ally";
  }
  return undefined;
}

function actionSideClass(action: ActionGroup): string {
  if (!action.side) return "";
  if (action.type === "skill" || action.type === "attack") {
    return ` action-${action.type}-${action.side}`;
  }
  return "";
}

function actionSideBadge(action: ActionGroup): string {
  if (!action.side || (action.type !== "skill" && action.type !== "attack")) return "";
  const label = action.side === "ally" ? "味方" : "敵";
  return `<span class="action-side-badge action-side-${action.side}">${label}</span>`;
}

/** サーバー内部用のフェーズ区切り（ユーザー向けログには出さない） */
function isHiddenBattleDelimiter(line: string): boolean {
  if (line.startsWith("--- 戦利品 ---") || line.startsWith("--- 魔獣入手 ---")) {
    return false;
  }
  return /^--- .+ ---$/.test(line);
}

/** 侵攻開始を先頭に並べ替え（旧ログでスキルが前に付いていた場合の補正） */
function sortBattleActions(actions: ActionGroup[]): ActionGroup[] {
  const invasionIdx = actions.findIndex((a) => a.title === "侵攻開始");
  if (invasionIdx <= 0) return actions;
  const sorted = [...actions];
  const [invasion] = sorted.splice(invasionIdx, 1);
  return [invasion, ...sorted];
}

/** サーバーが埋め込む [ts:ミリ秒] プレフィックスを抽出し、本文とタイムスタンプを返す */
function parseLogLine(raw: string): { text: string; tsMs: number | null } {
  const m = raw.match(/^\[ts:(\d+)\]/);
  if (m) {
    return { text: raw.slice(m[0].length), tsMs: parseInt(m[1], 10) };
  }
  return { text: raw, tsMs: null };
}

/** 侵攻開始行の [p:player_id]（攻撃実行者）を除去 */
function parseActorPrefix(text: string): { actorId: string | null; text: string } {
  const m = text.match(/^\[p:([^\]]+)\]/);
  if (m) {
    return { actorId: m[1], text: text.slice(m[0].length) };
  }
  return { actorId: null, text };
}

function isOwnBattleActor(actorId: string | null): boolean {
  if (!actorId) return true;
  return actorId === getLocalPlayerId();
}

function classifyLog(line: string): LogType {
  if (line.startsWith("◆◆") || line.includes("固有スキル")) return "skill_unique";
  if (line.startsWith("◆") || line.startsWith("★")) return "skill_passive";
  if (line.includes("が発動")) return "skill_active";
  if (line.startsWith("  →")) return "skill_effect";
  if (line.startsWith("【") && line.includes("侵攻戦】")) return "battle_start";
  if (/ユニット\d+が.+を攻撃しました/.test(line) || line.includes("へ侵攻開始")) return "battle_start";
  if (
    line.includes("占領しました") ||
    line.includes("攻撃失敗") ||
    line.includes("防衛に成功") ||
    line.includes("ターン経過") ||
    line.includes("占領には至らなかった")
  ) {
    return "battle_end";
  }
  if (line.includes("に攻撃") || line.includes("攻撃！")) return "attack";
  if (line.includes("撃破") || line.includes("倒れた")) return "defeat";
  if (line.includes("ダメージ")) return "damage";
  if (line.includes("回復") || line.includes("吸収")) return "heal";
  if (line.includes("毒") || line.includes("炎上") || line.includes("凍結") ||
      line.includes("気絶") || line.includes("沈黙") || line.includes("シールド") ||
      line.includes("無敵") || line.includes("バフ") || line.includes("マーク")) return "status";
  if (/^--- Turn \d+ ---$/.test(line) || line.startsWith("--- 戦闘フェーズ ---") || line.startsWith("--- スキル発動フェーズ ---")) return "normal";
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

/** 占領/失敗の直後に続く戦利品・魔獣入手（同一攻撃の結果） */
function isLootRelatedLine(line: string): boolean {
  if (line.startsWith("--- 戦利品 ---")) return true;
  if (line.startsWith("--- 魔獣入手 ---")) return true;
  if (line.includes("を入手！")) return true;
  if (line === "遺跡を攻略しました！") return true;
  return false;
}

/** 同一戦闘とみなすタイムスタンプの最大差（ミリ秒） */
const BATTLE_GROUP_GAP_MS = 5000;

/** ログを戦闘履歴にパース */
function parseLogsToHistory(rawLogs: string[]): BattleHistory[] {
  const histories: BattleHistory[] = [];
  let currentBattle: BattleHistory | null = null;
  let currentAction: ActionGroup | null = null;
  let battleId = 0;
  let postBattleLootMode = false;
  /** 他プレイヤーの戦闘ログを読み飛ばす */
  let skippingBattle = false;
  /** 現在の戦闘グループの基準タイムスタンプ */
  let currentBattleTsMs: number | null = null;
  const playerUnitNames = buildPlayerUnitNames();
  let characterSides = new Map<string, "ally" | "enemy">();

  /** 進行中の魔獣入手ブロックを確定して histories へ追加 */
  function finalizeCurrent() {
    if (!currentBattle) return;
    if (currentAction) currentBattle.actions.push(currentAction);
    histories.push(currentBattle);
    currentBattle = null;
    currentAction = null;
    currentBattleTsMs = null;
  }

  let lastTsMs: number | null = null;
  for (let lineIdx = 0; lineIdx < rawLogs.length; lineIdx++) {
    const parsed = parseLogLine(rawLogs[lineIdx]);
    const { actorId, text: lineBody } = parseActorPrefix(parsed.text);
    const line = lineBody;
    const tsMs = parsed.tsMs ?? lastTsMs;
    if (parsed.tsMs != null) lastTsMs = parsed.tsMs;
    if (isHiddenBattleDelimiter(line)) {
      continue;
    }
    const type = classifyLog(line);

    if (type === "battle_start") {
      skippingBattle = !isOwnBattleActor(actorId);
      if (skippingBattle) {
        postBattleLootMode = false;
        continue;
      }
    } else if (skippingBattle) {
      continue;
    }

    if (postBattleLootMode && currentBattle && !isLootRelatedLine(line) && type !== "battle_start") {
      finalizeCurrent();
      postBattleLootMode = false;
    }

    if (postBattleLootMode && currentBattle && isLootRelatedLine(line)) {
      if (line.startsWith("--- 戦利品 ---")) {
        if (currentAction) currentBattle.actions.push(currentAction);
        currentAction = {
          type: "phase",
          title: "戦利品",
          icon: "🎁",
          lines: [],
        };
      } else if (line.startsWith("--- 魔獣入手 ---")) {
        if (currentAction) currentBattle.actions.push(currentAction);
        currentAction = {
          type: "phase",
          title: "魔獣入手",
          icon: "🃏",
          lines: [],
        };
      } else if (line === "遺跡を攻略しました！") {
        if (!currentAction) {
          currentAction = {
            type: "phase",
            title: "遺跡",
            icon: "🏛️",
            lines: [line],
          };
        } else {
          currentAction.lines.push(line);
        }
      } else {
        if (!currentAction) {
          currentAction = {
            type: "phase",
            title: "戦利品",
            icon: "🎁",
            lines: [line],
          };
        } else {
          currentAction.lines.push(line);
        }
      }
      continue;
    }

    // タイムスタンプが大きく離れていたら別イベントとして分割
    if (
      currentBattle &&
      tsMs != null &&
      currentBattleTsMs != null &&
      Math.abs(tsMs - currentBattleTsMs) > BATTLE_GROUP_GAP_MS &&
      type !== "battle_end"
    ) {
      // battle_start は自身の分割ロジックがあるのでここではスキップ
      if (type !== "battle_start") {
        finalizeCurrent();
        postBattleLootMode = false;
      }
    }

    if (type === "battle_start") {
      postBattleLootMode = false;
      characterSides = new Map();
      let preludeActions: ActionGroup[] = [];
      if (currentBattle) {
        if (currentAction) currentBattle.actions.push(currentAction);
        currentAction = null;
        if (currentBattle.title === "戦闘" && currentBattle.result === "ongoing") {
          // 侵攻開始より前に流れてきたスキルは別戦闘の残骸。侵攻開始の後ろへ回す
          preludeActions = currentBattle.actions.filter((a) => a.type !== "skill");
        } else {
          histories.push(currentBattle);
        }
        currentBattle = null;
        currentBattleTsMs = null;
      }

      let title = "戦闘";
      const newFormatMatch = line.match(/【(.+?<\d+,\d+>)侵攻戦】/);
      if (newFormatMatch) {
        title = newFormatMatch[1];
      } else {
        const oldFormatMatch = line.match(/【(.+?)侵攻戦】/);
        if (oldFormatMatch) {
          title = oldFormatMatch[1];
        } else {
          const attackMatch = line.match(/ユニット\d+が(.+?)を攻撃しました/);
          if (attackMatch) {
            title = attackMatch[1];
          } else {
            const invasionMatch = line.match(/が(.+?)へ侵攻/);
            if (invasionMatch) {
              title = invasionMatch[1];
            }
          }
        }
      }

      currentBattleTsMs = tsMs ?? Date.now();
      currentBattle = {
        id: battleId++,
        title: title,
        timestamp: new Date(currentBattleTsMs),
        result: "ongoing",
        actions: preludeActions.slice(),
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
      currentBattleTsMs = tsMs ?? Date.now();
      currentBattle = {
        id: battleId++,
        title: "戦闘",
        timestamp: new Date(currentBattleTsMs),
        result: "ongoing",
        actions: [],
      };
    }

    if (type === "battle_end") {
      if (currentAction) {
        currentBattle.actions.push(currentAction);
      }
      const isVictory = line.includes("占領しました");
      const isPartialOcc = line.includes("占領には至らなかった");
      if (isVictory) {
        currentBattle.result = "victory";
      } else {
        currentBattle.result = "defeat";
      }
      currentAction = {
        type: "result",
        title: isVictory ? "占領成功" : isPartialOcc ? "敵撃破・未占領" : "攻撃失敗",
        icon: isVictory ? "🏆" : isPartialOcc ? "🛡️" : "💀",
        lines: [line],
      };
      currentBattle.actions.push(currentAction);
      currentAction = null;
      postBattleLootMode = isVictory;
      if (!isVictory) {
        histories.push(currentBattle);
        currentBattle = null;
        currentBattleTsMs = null;
      }
      continue;
    }

    if (type === "skill_passive" || type === "skill_active" || type === "skill_unique") {
      if (currentAction) {
        currentBattle.actions.push(currentAction);
      }
      const skillMatch = line.match(/「(.+?)」/);
      const charName = parseSkillCharacterName(line);
      const side = resolveActionSide(line, charName, characterSides, playerUnitNames);
      currentAction = {
        type: "skill",
        title: `${charName}の${skillMatch?.[1] ?? "スキル"}`,
        icon: getLogIcon(type),
        lines: [line],
        side,
      };
      continue;
    }

    if (type === "attack") {
      if (currentAction) {
        currentBattle.actions.push(currentAction);
      }
      const attackerName = parseAttackAttackerName(line);
      const attackMatch = line.match(/^(?:\[味方\]\s*|\[敵\]\s*)?(.+?)が(.+?)に攻撃/);
      const side = resolveActionSide(line, attackerName, characterSides, playerUnitNames);
      currentAction = {
        type: "attack",
        title: attackMatch ? `${attackMatch[1]} → ${attackMatch[2]}` : "攻撃",
        icon: "⚔️",
        lines: [line],
        side,
      };
      continue;
    }

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

/** 戦闘履歴の1エントリをレンダリング */
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
          ${sortBattleActions(battle.actions).map((action, idx) => `
            <div class="action-group action-${action.type}${actionSideClass(action)}" data-action-idx="${idx}">
              <div class="action-header">
                <span class="action-icon">${action.icon}</span>
                ${actionSideBadge(action)}
                <span class="action-title">${escapeHtml(action.title)}</span>
                <span class="action-toggle">▼</span>
              </div>
              <div class="action-body">
                ${action.lines
                  .filter((line) => !isHiddenBattleDelimiter(line))
                  .map(line => {
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
  
  detailModal.querySelectorAll(".action-group").forEach(group => {
    const header = group.querySelector(".action-header");
    header?.addEventListener("click", () => {
      group.classList.toggle("is-collapsed");
    });
  });
  
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
  
  detailModal = document.createElement("div");
  detailModal.className = "battle-detail-modal";
  document.body.appendChild(detailModal);
  
  const header = document.createElement("div");
  header.className = "history-header";
  header.innerHTML = `
    <div class="history-title">
      <span class="history-title-icon">📜</span>
      <span>戦闘履歴</span>
    </div>
  `;
  
  historyList = document.createElement("div");
  historyList.className = "history-list";
  
  wrapper.appendChild(header);
  wrapper.appendChild(historyList);
  
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
  battleHistories = parseLogsToHistory(logs);
  historyList.innerHTML = battleHistories.slice().reverse().map(renderHistoryCard).join("");
}
