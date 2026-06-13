import { countOwnedSlotsByCardId, getBodyDisplayName, getCardRarityClass, getCharacterIllustrationPath } from "./game/characters";
import {
  type AllocatableStatKey,
  formatStatAllocationHtml,
  getCardCoreStats,
  getDisplayedAllocationBonus,
  getEffectiveCardStats,
  getPlayerCardStatusPoints,
  statValueFromCard,
} from "./game/effective-stats";
import { getCharacterSkills } from "./game/skills";
import {
  FOOD_PER_MONSTER_PRODUCE,
  expNeededForLevel,
  getPlayerCardExp,
  getPlayerCardLevel,
  getPlayerCardMonsterCounts,
  getPlayerCardStamina,
  getPlayerFood,
  getPlayerOwnedCards,
  isCardSlotOnMarch,
  MAX_CARD_STAMINA,
  MAX_MONSTER_COUNT_PER_CARD_SLOT,
  MIN_MONSTER_COUNT_PER_CARD_SLOT,
} from "./shared/game-state";
import { gameState, getLocalPlayerId } from "./store";
import { escapeHtml } from "./utils";
import { renderBasicResourceHtml, RESOURCE_ICONS } from "./ui/resource-display";

const STAT_KEYS: AllocatableStatKey[] = [
  "speed",
  "attack",
  "intelligence",
  "defense",
  "magic_defense",
];
const STAT_LABELS: Record<AllocatableStatKey, string> = {
  speed: "速さ",
  attack: "攻撃",
  intelligence: "知力",
  defense: "防御",
  magic_defense: "魔防",
};

function statLine(
  cardId: number,
  bodySlot: number,
  key: AllocatableStatKey,
): string {
  const effective = getEffectiveCardStats(cardId, bodySlot, gameState, getLocalPlayerId());
  return String(statValueFromCard(effective, key));
}

/** `bodySlot`: 本拠の体インデックス（`owned_cards` の添字と一致） */
export function buildFormationCardDetailHtml(bodySlot: number): string {
  const owned = getPlayerOwnedCards(gameState, getLocalPlayerId());
  const cardId = owned[bodySlot] ?? bodySlot;
  const core = getCardCoreStats(cardId, bodySlot, gameState, getLocalPlayerId());
  const unspent = getPlayerCardStatusPoints(gameState, bodySlot, getLocalPlayerId());
  const skills = getCharacterSkills(cardId);
  const counts = getPlayerCardMonsterCounts(gameState, getLocalPlayerId());
  const currentMc = Math.min(
    Math.max(
      counts[bodySlot] ?? getEffectiveCardStats(cardId, bodySlot, gameState, getLocalPlayerId()).monster_count,
      MIN_MONSTER_COUNT_PER_CARD_SLOT,
    ),
    MAX_MONSTER_COUNT_PER_CARD_SLOT,
  );
  const level = getPlayerCardLevel(gameState, bodySlot, getLocalPlayerId());
  const exp = getPlayerCardExp(gameState, bodySlot, getLocalPlayerId());
  const expNeed = expNeededForLevel(level);
  const stamina = getPlayerCardStamina(gameState, bodySlot, getLocalPlayerId());
  const roomToCap = MAX_MONSTER_COUNT_PER_CARD_SLOT - currentMc;
  const food = getPlayerFood(gameState, getLocalPlayerId());
  const maxByFood =
    FOOD_PER_MONSTER_PRODUCE > 0 ? Math.floor(food / FOOD_PER_MONSTER_PRODUCE) : 0;
  const maxProduce = Math.max(0, Math.min(roomToCap, maxByFood));
  const onMarch = isCardSlotOnMarch(gameState, bodySlot, getLocalPlayerId());
  const duplicateCount = countOwnedSlotsByCardId(owned, cardId);
  const produceBlockReason = onMarch
    ? "遠征中のため魔獣を生産できません。"
    : roomToCap <= 0
      ? `魔獣数が上限（${MAX_MONSTER_COUNT_PER_CARD_SLOT}体）に達しています。`
      : food < FOOD_PER_MONSTER_PRODUCE
        ? `食料が足りません（1体あたり${FOOD_PER_MONSTER_PRODUCE}必要・所持${food}）。`
        : null;
  const duplicateBadge =
    duplicateCount > 1
      ? `<span class="formation-card-duplicate-badge" title="重複所持">+${duplicateCount - 1}</span>`
      : "";
  const rarityClass = getCardRarityClass(cardId);
  const skillLines: string[] = [];

  if (skills.passive) {
    skillLines.push(`[P] ${skills.passive.name}: ${skills.passive.description}`);
  }
  skillLines.push(`[A] ${skills.active.name}: ${skills.active.description}`);
  if (skills.unique) {
    skillLines.push(`[U] ${skills.unique.name}: ${skills.unique.description}`);
  }

  const trainBtn =
    unspent > 0
      ? `<button type="button" class="formation-train-btn" data-train-open>育成（未配分 ${unspent}pt）</button>`
      : "";

  return `
    <div class="formation-card-detail-header">
      <div class="formation-card-detail-img-wrap">
        <img src="${getCharacterIllustrationPath(cardId)}" alt="${escapeHtml(getBodyDisplayName(cardId))}" class="formation-card-detail-img" />
        ${duplicateBadge}
      </div>
      <div class="formation-card-detail-name ${rarityClass}">${escapeHtml(getBodyDisplayName(cardId))}</div>
    </div>
    <div class="formation-card-detail-stats">
      <div>Lv ${level}（EXP ${exp} / 次Lvまで ${expNeed}）</div>
      <div>スタミナ: ${stamina} / ${MAX_CARD_STAMINA}</div>
      <div>現在魔獣数: ${currentMc} / 上限 ${MAX_MONSTER_COUNT_PER_CARD_SLOT}</div>
      <div>速さ: ${statLine(cardId, bodySlot, "speed")} / 射程: ${core.range}</div>
      <div>攻撃: ${statLine(cardId, bodySlot, "attack")} / 知力: ${statLine(cardId, bodySlot, "intelligence")}</div>
      <div>防御: ${statLine(cardId, bodySlot, "defense")} / 魔防: ${statLine(cardId, bodySlot, "magic_defense")}</div>
    </div>
    ${trainBtn}
    <div class="formation-card-detail-skills">
      <div class="formation-card-detail-skills-title">スキル</div>
      ${skillLines.map((line) => `<div class="formation-card-detail-skill">${escapeHtml(line)}</div>`).join("")}
    </div>
    <div class="formation-card-detail-produce">
      ${
        produceBlockReason
          ? `<p class="formation-produce-meta">${escapeHtml(produceBlockReason)}</p>
        <button type="button" class="formation-produce-btn" disabled>魔獣を生産</button>`
          : `<button type="button" class="formation-produce-btn" data-produce-start>魔獣を生産</button>
      <div class="formation-produce-panel" data-produce-panel hidden>
        <p class="formation-produce-error" data-produce-error hidden></p>
        <label class="formation-produce-label" for="formation-produce-amt-${bodySlot}">生産数</label>
        <input
          type="number"
          id="formation-produce-amt-${bodySlot}"
          class="formation-produce-input"
          data-produce-amount
          min="1"
          max="${maxProduce}"
          value="1"
          step="1"
          inputmode="numeric"
        />
        <p class="formation-produce-meta">
          魔獣を1体生産するごとに、<span class="formation-produce-meta-resource" aria-label="食料"><span class="resource-icon" aria-hidden="true">${RESOURCE_ICONS.food}</span><strong>${FOOD_PER_MONSTER_PRODUCE}</strong>個</span>消費します。
        </p>
        <div class="formation-produce-cost">
          <span class="formation-produce-label">消費</span>
          <span data-produce-cost>${renderBasicResourceHtml("food", FOOD_PER_MONSTER_PRODUCE, "resource-value formation-produce-cost-value")}</span>
        </div>
        <button type="button" class="formation-produce-btn" data-produce-submit>生産する</button>
        <button type="button" class="formation-produce-cancel" data-produce-cancel>戻る</button>
      </div>`
      }
    </div>
  `;
}

/** ステータス振り分けダイアログ（育成ボタンから開く） */
export function buildStatAllocDialogHtml(bodySlot: number): string {
  const owned = getPlayerOwnedCards(gameState, getLocalPlayerId());
  const cardId = owned[bodySlot] ?? bodySlot;
  const unspent = getPlayerCardStatusPoints(gameState, bodySlot, getLocalPlayerId());

  const statAllocRows = STAT_KEYS.map((key) => {
    const core = getCardCoreStats(cardId, bodySlot, gameState, getLocalPlayerId());
    const coreVal = key === "magic_defense" ? core.magicDefense : core[key];
    const alloc = getDisplayedAllocationBonus(cardId, bodySlot, gameState, key, getLocalPlayerId());
    const currentText = formatStatAllocationHtml(coreVal, alloc);
    return `
      <div class="formation-stat-alloc-row">
        <span class="formation-stat-alloc-label">${STAT_LABELS[key]}</span>
        <span class="formation-stat-alloc-current" data-stat-preview="${key}">${currentText}</span>
        <input
          type="number"
          id="formation-stat-${key}-${bodySlot}"
          class="formation-stat-alloc-input"
          data-stat-key="${key}"
          data-stat-core="${coreVal}"
          data-stat-allocated="${alloc}"
          min="0"
          max="${unspent}"
          value="0"
          step="1"
          inputmode="numeric"
          aria-label="${STAT_LABELS[key]}へ振り分けるポイント"
        />
      </div>
    `;
  }).join("");

  return `
    <div class="formation-stat-alloc-dialog">
      <div class="formation-stat-alloc-dialog-title">ステータス振り分け</div>
      <p class="formation-stat-alloc-dialog-name ${getCardRarityClass(cardId)}">${escapeHtml(getBodyDisplayName(cardId))}</p>
      <p class="formation-stat-alloc-meta">未配分ポイント: <strong data-stat-unspent>${unspent}</strong></p>
      <p class="formation-stat-alloc-meta">使用予定: <strong data-stat-spend>0</strong> / ${unspent}</p>
      <p class="formation-stat-alloc-hint">現在値は「基礎 (+振り分け)」。入力分は (+) のみ増えます。</p>
      <p class="formation-stat-alloc-error" data-stat-alloc-error hidden></p>
      <div class="formation-stat-alloc-grid formation-stat-alloc-grid--dialog">
        ${statAllocRows}
      </div>
      <div class="formation-stat-alloc-actions">
        <button type="button" class="formation-stat-alloc-btn" data-stat-alloc-submit>振り分ける</button>
        <button type="button" class="formation-stat-alloc-cancel" data-stat-alloc-cancel>戻る</button>
      </div>
    </div>
  `;
}
