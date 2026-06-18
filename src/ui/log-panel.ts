/**
 * 戦闘履歴パネル（グループ化された履歴表示）
 */

import { getBodyDisplayName } from "../game/characters";
import { getItem } from "../game/items";
import { aiFactionName, getPlayerOwnedCards, isAiOwnerId } from "../shared/game-state";
import { gameState, getLocalPlayerId } from "../store";
import { escapeHtml } from "../utils";
import { renderResourceDeltaHtml } from "./resource-display";
import { renderScreenHeaderTitle } from "./screen-header";

let historyList: HTMLDivElement;
let detailModal: HTMLDivElement;

/** ログの種類を判定 */
type LogType = 
  | "skill_passive" | "skill_active" | "skill_unique" | "skill_effect"
  | "attack" | "defeat" | "damage" | "heal" | "status"
  | "battle_start" | "battle_end" | "normal";

/** 戦闘・探索の履歴エントリ */
interface HistoryEntry {
  id: number;
  kind: "battle" | "explore";
  title: string;
  /** 探索: 領地名（派遣と完了の突合） */
  territoryName?: string;
  timestamp: Date;
  result: "victory" | "defeat" | "ongoing" | "success";
  actions: ActionGroup[];
}

/** 行動グループ */
interface ActionGroup {
  type: "skill" | "attack" | "phase" | "result";
  title: string;
  icon: string;
  lines: string[];
  side?: "ally" | "enemy";
  attackerName?: string;
  targetName?: string;
  targetSide?: "ally" | "enemy";
}

function detectLogSide(line: string, invertPerspective = false): "ally" | "enemy" | undefined {
  let side: "ally" | "enemy" | undefined;
  if (line.includes("[味方]")) side = "ally";
  else if (line.includes("[敵]")) side = "enemy";
  if (!side) return undefined;
  if (invertPerspective) return side === "ally" ? "enemy" : "ally";
  return side;
}

function formatBattleActorLabel(actorId: string | null): string {
  if (!actorId) return "";
  if (isAiOwnerId(actorId)) return aiFactionName(gameState, actorId) ?? actorId;
  return actorId;
}

/** 敵主体の侵攻ログがローカルプレイヤーへの攻撃か */
function isInvasionAgainstLocalPlayer(line: string): boolean {
  const localId = getLocalPlayerId();
  return line.includes(`（${localId}）へ侵攻開始`);
}

function parseInvasionBattleTitle(line: string, actorId: string | null): string {
  const localId = getLocalPlayerId();
  const includeActor = actorId != null && actorId !== localId;
  const actorLabel = includeActor ? formatBattleActorLabel(actorId) : "";

  const placeMatch = line.match(/【(.+?<(\d+),(\d+)>)侵攻戦】/);
  const invasionWithOwner = line.match(/】(.+?)が.+?（(.+?)）へ侵攻開始/);
  const invasionPlain = line.match(/】(.+?)が(.+?)へ侵攻開始/);
  const invasion = invasionWithOwner ?? invasionPlain;
  if (!invasion) {
    return placeMatch?.[1] ?? "戦闘";
  }
  const attacker = invasion[1];
  const place = placeMatch?.[1] ?? "";
  const defender = place || invasionWithOwner?.[2] || invasionPlain?.[2] || "";
  if (actorLabel && defender) return `${actorLabel} / ${attacker} → ${defender}`;
  if (defender) return `${attacker} → ${defender}`;
  return place || attacker;
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

function parseAttackTargetName(line: string): string {
  const m = line.match(/^(?:\[味方\]\s*|\[敵\]\s*)?(.+?)が(.+?)に攻撃/);
  return m?.[2] ?? "";
}

function parseAttackAttackerName(line: string): string {
  const m = line.match(/^(?:\[味方\]\s*|\[敵\]\s*)?(.+?)が.+?に攻撃/);
  return m?.[1] ?? "";
}

function formatAttackLineHtml(
  line: string,
  attackerSide: "ally" | "enemy",
  targetSide: "ally" | "enemy",
): string | null {
  const m = line.match(/^(?:\[味方\]\s*|\[敵\]\s*)?(.+?)が(.+?)に攻撃([！!])(.*)$/);
  if (!m) return null;
  const [, attacker, target, punct, rest] = m;
  return `${renderSideBadge(attackerSide)}<span class="log-actor-name">${escapeHtml(attacker)}</span>が${renderSideBadge(targetSide)}<span class="log-actor-name">${escapeHtml(target)}</span>に攻撃${punct}${escapeHtml(rest)}`;
}

/** 行内の [味方]/[敵] テキストを色付きバッジへ置換 */
function replaceSideTagsWithBadges(line: string): string {
  return line
    .split(/(\[味方\]|\[敵\])/)
    .map((part) => {
      if (part === "[味方]") return renderSideBadge("ally");
      if (part === "[敵]") return renderSideBadge("enemy");
      return escapeHtml(part);
    })
    .join("");
}

function resolveActionSide(
  line: string,
  actorName: string,
  characterSides: Map<string, "ally" | "enemy">,
  playerUnitNames: Set<string>,
  invertPerspective = false,
): "ally" | "enemy" | undefined {
  const tagged = detectLogSide(line, invertPerspective);
  if (tagged) {
    if (actorName) characterSides.set(actorName, tagged);
    return tagged;
  }
  if (actorName) {
    const known = characterSides.get(actorName);
    if (known) return invertPerspective ? (known === "ally" ? "enemy" : "ally") : known;
    if (actorName.startsWith("味方ユニット")) return invertPerspective ? "enemy" : "ally";
    if (actorName.startsWith("敵ユニット")) return invertPerspective ? "ally" : "enemy";
    if (playerUnitNames.has(actorName)) return invertPerspective ? "enemy" : "ally";
  }
  return undefined;
}

function actionSideClass(action: ActionGroup): string {
  if (!action.side) return "";
  if (action.type === "skill" || action.type === "attack" || action.type === "phase") {
    return ` action-${action.type}-${action.side}`;
  }
  return "";
}

function actionSideBadge(action: ActionGroup): string {
  if (!action.side || (action.type !== "skill" && action.type !== "attack" && action.type !== "phase")) return "";
  const label = action.side === "ally" ? "味方" : "敵";
  return `<span class="action-side-badge action-side-${action.side}">${label}</span>`;
}

function renderSideBadge(side: "ally" | "enemy"): string {
  const label = side === "ally" ? "味方" : "敵";
  return `<span class="action-side-badge action-side-${side}">${label}</span>`;
}

function resolveCharacterSide(
  name: string,
  characterSides: Map<string, "ally" | "enemy">,
  playerUnitNames: Set<string>,
  invertPerspective: boolean,
): "ally" | "enemy" | undefined {
  if (!name) return undefined;
  const known = characterSides.get(name);
  if (known) return invertPerspective ? (known === "ally" ? "enemy" : "ally") : known;
  if (name.startsWith("味方ユニット")) return invertPerspective ? "enemy" : "ally";
  if (name.startsWith("敵ユニット")) return invertPerspective ? "ally" : "enemy";
  if (playerUnitNames.has(name)) return invertPerspective ? "enemy" : "ally";
  return undefined;
}

function resolveTargetSide(
  targetName: string,
  attackerSide: "ally" | "enemy" | undefined,
  characterSides: Map<string, "ally" | "enemy">,
  playerUnitNames: Set<string>,
  invertPerspective: boolean,
): "ally" | "enemy" | undefined {
  const known = resolveCharacterSide(targetName, characterSides, playerUnitNames, invertPerspective);
  if (known) return known;
  if (attackerSide) return attackerSide === "ally" ? "enemy" : "ally";
  return undefined;
}

function registerEnemyRosterName(
  line: string,
  characterSides: Map<string, "ally" | "enemy">,
): void {
  const m = line.match(/^\[敵\]\s+(.+?)（/);
  if (m) characterSides.set(m[1], "enemy");
}

function renderAttackTitle(action: ActionGroup): string {
  if (action.attackerName && action.targetName && action.side && action.targetSide) {
    return `${renderSideBadge(action.side)}<span class="action-actor-name">${escapeHtml(action.attackerName)}</span>`
      + `<span class="action-arrow">→</span>`
      + `${renderSideBadge(action.targetSide)}<span class="action-actor-name">${escapeHtml(action.targetName)}</span>`;
  }
  return escapeHtml(action.title);
}

function renderActionTitle(action: ActionGroup): string {
  if (action.type === "attack") return renderAttackTitle(action);
  return escapeHtml(action.title);
}

/** サーバー内部用のフェーズ区切り（ユーザー向けログには出さない） */
function isHiddenBattleDelimiter(line: string): boolean {
  if (
    line.startsWith("--- 戦利品 ---") ||
    line.startsWith("--- 魔獣入手 ---") ||
    line.startsWith("--- 敵編成 ---")
  ) {
    return false;
  }
  return /^--- .+ ---$/.test(line);
}

/** 戦闘フェーズ区切りをアクショングループに変換（スタートアップスキルとターン行の混同を防ぐ） */
function pushCurrentAction(entry: HistoryEntry, action: ActionGroup | null): ActionGroup | null {
  if (action) {
    entry.actions.push(action);
  }
  return null;
}

function handleBattlePhaseDelimiter(
  line: string,
): { handled: boolean; currentAction: ActionGroup | null } {
  if (line === "--- スタートアップフェーズ ---") {
    return {
      handled: true,
      currentAction: {
        type: "phase",
        title: "スタートアップ",
        icon: "✨",
        lines: [],
      },
    };
  }
  if (line === "--- 戦闘フェーズ ---") {
    return {
      handled: true,
      currentAction: {
        type: "phase",
        title: "戦闘フェーズ",
        icon: "⚔️",
        lines: [],
      },
    };
  }
  const turnMatch = line.match(/^--- Turn (\d+) ---$/);
  if (turnMatch) {
    return {
      handled: true,
      currentAction: {
        type: "phase",
        title: `ターン ${turnMatch[1]}`,
        icon: "🔄",
        lines: [],
      },
    };
  }
  const waveMatch = line.match(/^--- 第(\d+)戦 ---$/);
  if (waveMatch) {
    return {
      handled: true,
      currentAction: {
        type: "phase",
        title: `第${waveMatch[1]}戦`,
        icon: "🌊",
        lines: [],
      },
    };
  }
  return { handled: false, currentAction: null };
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
    line.includes("占領には至らなかった") ||
    line.includes("演習戦")
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

/** 占領/失敗の直後に続く戦利品・魔獣入手・占領報酬（同一攻撃の結果） */
function isLootRelatedLine(line: string): boolean {
  if (line.startsWith("占領報酬:")) return true;
  if (line.startsWith("--- 戦利品 ---")) return true;
  if (line.startsWith("--- 魔獣入手 ---")) return true;
  if (line.includes("を入手！")) return true;
  if (line === "遺跡を攻略しました！") return true;
  return false;
}

function isIgnorableSystemLine(line: string): boolean {
  return line === "スタミナが足りない魔獣が含まれています。";
}

function formatConquestRewardLine(line: string): string | null {
  const m = line.match(/^占領報酬:\s*食料\+(\d+)・木\+(\d+)・石\+(\d+)・鉄\+(\d+)/);
  if (!m) return null;
  const parts = [
    renderResourceDeltaHtml("food", Number(m[1])),
    renderResourceDeltaHtml("wood", Number(m[2])),
    renderResourceDeltaHtml("stone", Number(m[3])),
    renderResourceDeltaHtml("iron", Number(m[4])),
  ];
  return `<span class="log-conquest-reward"><span class="log-conquest-reward-label">占領報酬:</span> ${parts.join(" ")}</span>`;
}

function formatLootPickupLine(line: string): string | null {
  const goldMatch = line.match(/^ゴールド\+(\d+) を入手！$/);
  if (goldMatch) {
    return `<span class="log-loot-pickup">${renderResourceDeltaHtml("gold", Number(goldMatch[1]), "resource-value resource-delta log-loot-gold")} <span class="log-loot-suffix">を入手！</span></span>`;
  }
  const itemMatch = line.match(/^(.+?)x(\d+) を入手！$/);
  if (!itemMatch) return null;
  const itemKey = itemMatch[1];
  const count = itemMatch[2];
  const def = getItem(itemKey);
  const label = def?.name ?? itemKey;
  const icon = def?.icon ?? "📦";
  return `<span class="log-loot-pickup"><span class="log-loot-icon" aria-hidden="true">${icon}</span> <span class="log-loot-name">${escapeHtml(label)}</span><span class="resource-delta-amount">x${count}</span> <span class="log-loot-suffix">を入手！</span></span>`;
}

function formatExploreRewardLine(line: string): string | null {
  const m = line.match(/^(.+?)の探索が完了。食料\+(\d+)・木\+(\d+)・石\+(\d+)・鉄\+(\d+)$/);
  if (!m) return null;
  const parts = [
    renderResourceDeltaHtml("food", Number(m[2])),
    renderResourceDeltaHtml("wood", Number(m[3])),
    renderResourceDeltaHtml("stone", Number(m[4])),
    renderResourceDeltaHtml("iron", Number(m[5])),
  ];
  return `<span class="log-explore-reward"><span class="log-explore-reward-label">${escapeHtml(m[1])}の探索完了:</span> ${parts.join(" ")}</span>`;
}

function isExploreDispatchLine(line: string): boolean {
  return /探索を.+へ派遣しました。/.test(line);
}

function parseExploreDispatchLine(line: string): { territory: string } | null {
  const m = line.match(/探索を(.+?)へ派遣しました。/);
  if (!m) return null;
  return { territory: m[1] };
}

function parseExploreCompleteLine(line: string): {
  territory: string;
  food: number;
  wood: number;
  stone: number;
  iron: number;
} | null {
  const m = line.match(/^(.+?)の探索が完了。食料\+(\d+)・木\+(\d+)・石\+(\d+)・鉄\+(\d+)$/);
  if (!m) return null;
  return {
    territory: m[1],
    food: Number(m[2]),
    wood: Number(m[3]),
    stone: Number(m[4]),
    iron: Number(m[5]),
  };
}

function isExploreLevelUpLine(line: string): boolean {
  return line.includes("探索経験が溜まり、探索レベルが");
}

function findPendingExploreEntry(histories: HistoryEntry[], territory: string): HistoryEntry | undefined {
  for (let i = histories.length - 1; i >= 0; i--) {
    const h = histories[i];
    if (h.kind === "explore" && h.result === "ongoing" && h.territoryName === territory) {
      return h;
    }
  }
  return undefined;
}

function completeExploreEntry(entry: HistoryEntry, line: string, tsMs: number | null): void {
  entry.result = "success";
  entry.actions.push({
    type: "result",
    title: "探索完了",
    icon: "✅",
    lines: [line],
  });
  if (tsMs != null) {
    entry.timestamp = new Date(tsMs);
  }
}
function formatBattleLogLineHtml(line: string, action?: ActionGroup): string {
  if (action?.type === "attack" && action.side && action.targetSide) {
    const attackHtml = formatAttackLineHtml(line, action.side, action.targetSide);
    if (attackHtml) return attackHtml;
  }
  const formatted = formatConquestRewardLine(line) ?? formatExploreRewardLine(line) ?? formatLootPickupLine(line);
  if (formatted) return formatted;
  if (line.includes("[味方]") || line.includes("[敵]")) {
    return replaceSideTagsWithBadges(line);
  }
  return escapeHtml(line);
}

/** 同一戦闘とみなすタイムスタンプの最大差（ミリ秒） */
const BATTLE_GROUP_GAP_MS = 5000;

/** ログを戦闘・探索履歴にパース */
function parseLogsToHistory(rawLogs: string[]): HistoryEntry[] {
  const histories: HistoryEntry[] = [];
  let currentBattle: HistoryEntry | null = null;
  let currentExplore: HistoryEntry | null = null;
  let currentAction: ActionGroup | null = null;
  let historyId = 0;
  let postBattleLootMode = false;
  let postExploreLevelUpMode = false;
  let lastCompletedExplore: HistoryEntry | null = null;
  /** 他プレイヤーの戦闘ログを読み飛ばす（自領への敵侵攻は除く） */
  let skippingBattle = false;
  /** 敵主体ログを防衛視点で表示 */
  let viewingAsDefender = false;
  /** 現在の戦闘グループの基準タイムスタンプ */
  let currentBattleTsMs: number | null = null;
  const playerUnitNames = buildPlayerUnitNames();
  let characterSides = new Map<string, "ally" | "enemy">();

  /** 進行中の戦闘を確定して histories へ追加 */
  function finalizeCurrent() {
    if (!currentBattle) return;
    if (currentAction) currentBattle.actions.push(currentAction);
    histories.push(currentBattle);
    currentBattle = null;
    currentAction = null;
    currentBattleTsMs = null;
  }

  function finalizeCurrentExplore() {
    if (!currentExplore) return;
    histories.push(currentExplore);
    currentExplore = null;
  }

  let lastTsMs: number | null = null;
  for (let lineIdx = 0; lineIdx < rawLogs.length; lineIdx++) {
    const parsed = parseLogLine(rawLogs[lineIdx]);
    const { actorId, text: lineBody } = parseActorPrefix(parsed.text);
    const line = lineBody;
    const tsMs = parsed.tsMs ?? lastTsMs;
    if (parsed.tsMs != null) lastTsMs = parsed.tsMs;
    if (isIgnorableSystemLine(line)) {
      continue;
    }
    registerEnemyRosterName(line, characterSides);
    const type = classifyLog(line);

    // 他プレイヤー/AI主体: 自領への侵攻のみ戦歴に載せる
    if (actorId && !isOwnBattleActor(actorId)) {
      if (type === "battle_start") {
        if (!isInvasionAgainstLocalPlayer(line)) {
          skippingBattle = true;
          viewingAsDefender = false;
        } else {
          skippingBattle = false;
          viewingAsDefender = true;
        }
      }
      if (skippingBattle) {
        continue;
      }
    } else {
      viewingAsDefender = false;
    }

    if (type === "battle_start") {
      skippingBattle = false;
    } else if (skippingBattle) {
      continue;
    }

    if (actorId && !isOwnBattleActor(actorId)) {
      if (isExploreDispatchLine(line) || parseExploreCompleteLine(line) || isExploreLevelUpLine(line)) {
        continue;
      }
    }

    if (postExploreLevelUpMode) {
      if (isExploreLevelUpLine(line) && (!actorId || isOwnBattleActor(actorId))) {
        const target = lastCompletedExplore ?? histories.filter((h) => h.kind === "explore").at(-1);
        if (target) {
          target.actions.push({
            type: "phase",
            title: "探索レベルアップ",
            icon: "⬆️",
            lines: [line],
          });
        }
        continue;
      }
      postExploreLevelUpMode = false;
      lastCompletedExplore = null;
    }

    if (isExploreDispatchLine(line) && (!actorId || isOwnBattleActor(actorId))) {
      finalizeCurrentExplore();
      finalizeCurrent();
      const dispatch = parseExploreDispatchLine(line)!;
      currentExplore = {
        id: historyId++,
        kind: "explore",
        title: dispatch.territory,
        territoryName: dispatch.territory,
        timestamp: new Date(tsMs ?? Date.now()),
        result: "ongoing",
        actions: [{
          type: "phase",
          title: "探索派遣",
          icon: "🧭",
          lines: [line],
        }],
      };
      continue;
    }

    const exploreComplete = parseExploreCompleteLine(line);
    if (exploreComplete && (!actorId || isOwnBattleActor(actorId))) {
      finalizeCurrent();
      finalizeCurrentExplore();
      let entry: HistoryEntry;
      const pending = findPendingExploreEntry(histories, exploreComplete.territory);
      if (currentExplore && currentExplore.territoryName === exploreComplete.territory) {
        entry = currentExplore;
        currentExplore = null;
      } else if (pending) {
        entry = pending;
        histories.splice(histories.indexOf(pending), 1);
      } else {
        entry = {
          id: historyId++,
          kind: "explore",
          title: exploreComplete.territory,
          territoryName: exploreComplete.territory,
          timestamp: new Date(tsMs ?? Date.now()),
          result: "ongoing",
          actions: [],
        };
      }
      completeExploreEntry(entry, line, tsMs);
      histories.push(entry);
      lastCompletedExplore = entry;
      postExploreLevelUpMode = true;
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

    // タイムスタンプが大きく離れていたら別イベントとして分割（戦利品収集中は除く）
    if (
      currentBattle &&
      !postBattleLootMode &&
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

    if (line.startsWith("--- 敵編成 ---")) {
      if (!currentBattle) {
        continue;
      }
      if (currentAction) {
        currentBattle.actions.push(currentAction);
      }
      currentAction = {
        type: "phase",
        title: "敵編成",
        icon: "👹",
        lines: [],
        side: "enemy",
      };
      continue;
    }

    if (type === "battle_start") {
      postBattleLootMode = false;
      finalizeCurrentExplore();
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

      let title = parseInvasionBattleTitle(line, actorId);
      if (title === "戦闘") {
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
      }

      currentBattleTsMs = tsMs ?? Date.now();
      currentBattle = {
        id: historyId++,
        kind: "battle",
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
      continue;
    }

    const phaseDelimiter = handleBattlePhaseDelimiter(line);
    if (phaseDelimiter.handled) {
      currentAction = pushCurrentAction(currentBattle, currentAction);
      currentAction = phaseDelimiter.currentAction;
      continue;
    }

    if (isHiddenBattleDelimiter(line)) {
      continue;
    }

    if (type === "battle_end") {
      if (currentAction) {
        currentBattle.actions.push(currentAction);
      }
      const isVictory = viewingAsDefender
        ? line.includes("攻撃失敗") || line.includes("防衛に成功")
        : line.includes("占領しました") || line.includes("演習戦に勝利");
      const isPartialOcc = line.includes("占領には至らなかった");
      const isPracticeBattle = line.includes("演習戦");
      if (viewingAsDefender && line.includes("占領しました")) {
        currentBattle.result = "defeat";
      } else if (isVictory) {
        currentBattle.result = "victory";
      } else {
        currentBattle.result = "defeat";
      }
      currentAction = {
        type: "result",
        title: viewingAsDefender
          ? isVictory
            ? "防衛成功"
            : isPartialOcc
              ? "防衛（部分）"
              : "防衛失敗"
          : isPracticeBattle
            ? (isVictory ? "演習勝利" : "演習敗北")
            : isVictory
              ? "占領成功"
              : isPartialOcc
                ? "敵撃破・未占領"
                : "攻撃失敗",
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
      currentAction = pushCurrentAction(currentBattle, currentAction);
      const skillMatch = line.match(/「(.+?)」/);
      const charName = parseSkillCharacterName(line);
      const side = resolveActionSide(line, charName, characterSides, playerUnitNames, viewingAsDefender);
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
      const targetName = parseAttackTargetName(line);
      const side = resolveActionSide(line, attackerName, characterSides, playerUnitNames, viewingAsDefender);
      const targetSide = resolveTargetSide(
        targetName,
        side,
        characterSides,
        playerUnitNames,
        viewingAsDefender,
      );
      if (targetName && targetSide) {
        characterSides.set(targetName, targetSide);
      }
      currentAction = {
        type: "attack",
        title: targetName ? `${attackerName} → ${targetName}` : "攻撃",
        icon: "⚔️",
        lines: [line],
        side,
        attackerName,
        targetName: targetName || undefined,
        targetSide,
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

  finalizeCurrent();
  finalizeCurrentExplore();

  return histories;
}

/** 時刻をフォーマット */
function formatTime(date: Date): string {
  const h = date.getHours().toString().padStart(2, "0");
  const m = date.getMinutes().toString().padStart(2, "0");
  const s = date.getSeconds().toString().padStart(2, "0");
  return `${h}:${m}:${s}`;
}

/** 履歴の1エントリをレンダリング */
function renderHistoryCard(entry: HistoryEntry): string {
  const resultClass = entry.kind === "explore"
    ? entry.result === "success" ? "history-explore" : "history-ongoing"
    : entry.result === "victory" ? "history-victory"
      : entry.result === "defeat" ? "history-defeat" : "history-ongoing";
  const resultIcon = entry.kind === "explore"
    ? "🧭"
    : entry.result === "victory" ? "🏆"
      : entry.result === "defeat" ? "💀" : "⚔️";
  const title = entry.kind === "explore"
    ? `探索: ${entry.title}`
    : entry.title;

  const timeStr = formatTime(entry.timestamp);

  return `
    <div class="history-card ${resultClass}" data-battle-id="${entry.id}">
      <span class="history-card-time">${timeStr}</span>
      <span class="history-card-icon">${resultIcon}</span>
      <span class="history-card-title">${escapeHtml(title)}</span>
    </div>
  `;
}

/** 戦闘・探索詳細モーダルを表示 */
function showHistoryDetail(entry: HistoryEntry): void {
  const resultClass = entry.kind === "explore"
    ? entry.result === "success" ? "detail-explore" : "detail-ongoing"
    : entry.result === "victory" ? "detail-victory"
      : entry.result === "defeat" ? "detail-defeat" : "detail-ongoing";
  const modalTitle = entry.kind === "explore" ? `探索: ${entry.title}` : entry.title;
  
  detailModal.innerHTML = `
    <div class="detail-overlay" data-close="true">
      <div class="detail-modal ${resultClass}">
        <div class="detail-header">
          <h2 class="detail-title">${escapeHtml(modalTitle)}</h2>
          <button class="detail-close" data-close="true">✕</button>
        </div>
        <div class="detail-content">
          ${sortBattleActions(entry.actions).map((action, idx) => `
            <div class="action-group action-${action.type}${actionSideClass(action)}" data-action-idx="${idx}">
              <div class="action-header">
                <span class="action-icon">${action.icon}</span>
                ${action.type === "attack" ? "" : actionSideBadge(action)}
                <span class="action-title">${renderActionTitle(action)}</span>
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
                    <span class="line-text">${formatBattleLogLineHtml(line, action)}</span>
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

let historyEntries: HistoryEntry[] = [];

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
      ${renderScreenHeaderTitle("history", "戦歴")}
    </div>
  `;
  
  historyList = document.createElement("div");
  historyList.className = "history-list";
  
  wrapper.appendChild(header);
  wrapper.appendChild(historyList);
  
  historyList.addEventListener("click", (e) => {
    const card = (e.target as HTMLElement).closest(".history-card");
    if (card) {
      const entryId = parseInt(card.getAttribute("data-battle-id") ?? "0");
      const entry = historyEntries.find((b) => b.id === entryId);
      if (entry) {
        showHistoryDetail(entry);
      }
    }
  });
  
  return wrapper;
}

export function renderLog(): void {
  const logs = gameState.log ?? [];
  historyEntries = parseLogsToHistory(logs);
  historyList.innerHTML = historyEntries
    .slice()
    .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
    .map(renderHistoryCard)
    .join("");
}
