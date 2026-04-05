import { getBodyDisplayName, getCharacterIllustrationPath, getCharacterStats } from "./game/characters";
import { getCharacterSkills } from "./game/skills";
import {
  FOOD_PER_MONSTER_PRODUCE,
  getPlayerCardMonsterCounts,
  getPlayerFood,
  getPlayerOwnedCards,
  MAX_MONSTER_COUNT_PER_CARD_SLOT,
  MIN_MONSTER_COUNT_PER_CARD_SLOT,
} from "./shared/game-state";
import { gameState } from "./store";
import { escapeHtml } from "./utils";

/** `bodySlot`: 本拠の体インデックス（`owned_cards` の添字と一致） */
export function buildFormationCardDetailHtml(bodySlot: number): string {
  const owned = getPlayerOwnedCards(gameState);
  const cardId = owned[bodySlot] ?? bodySlot;
  const stats = getCharacterStats(cardId);
  const skills = getCharacterSkills(cardId);
  const counts = getPlayerCardMonsterCounts(gameState);
  const currentMc = Math.min(
    Math.max(counts[bodySlot] ?? stats.monster_count, MIN_MONSTER_COUNT_PER_CARD_SLOT),
    MAX_MONSTER_COUNT_PER_CARD_SLOT
  );
  const roomToCap = MAX_MONSTER_COUNT_PER_CARD_SLOT - currentMc;
  const food = getPlayerFood(gameState);
  const maxByFood =
    FOOD_PER_MONSTER_PRODUCE > 0 ? Math.floor(food / FOOD_PER_MONSTER_PRODUCE) : 0;
  const maxProduce = Math.max(0, Math.min(roomToCap, maxByFood));
  const skillLines: string[] = [];

  if (skills.passive) {
    skillLines.push(`[P] ${skills.passive.name}: ${skills.passive.description}`);
  }
  skillLines.push(`[A] ${skills.active.name}: ${skills.active.description}`);
  if (skills.unique) {
    skillLines.push(`[U] ${skills.unique.name}: ${skills.unique.description}`);
  }

  return `
    <div class="formation-card-detail-header">
      <img src="${getCharacterIllustrationPath(cardId)}" alt="${escapeHtml(getBodyDisplayName(cardId))}" class="formation-card-detail-img" />
      <div class="formation-card-detail-name">${escapeHtml(getBodyDisplayName(cardId))}</div>
    </div>
    <div class="formation-card-detail-stats">
      <div>現在魔獣数: ${currentMc} / 上限 ${MAX_MONSTER_COUNT_PER_CARD_SLOT}</div>
      <div>SPEED: ${stats.speed} / 射程: ${stats.range}</div>
      <div>攻撃: ${stats.attack} / 知力: ${stats.intelligence}</div>
      <div>防御: ${stats.defense} / 魔防: ${stats.magicDefense}</div>
    </div>
    <div class="formation-card-detail-skills">
      <div class="formation-card-detail-skills-title">スキル</div>
      ${skillLines.map((line) => `<div class="formation-card-detail-skill">${escapeHtml(line)}</div>`).join("")}
    </div>
    <div class="formation-card-detail-produce">
      ${
        maxProduce === 0
          ? `<button type="button" class="formation-produce-btn" disabled>魔獣を生産</button>`
          : `<button type="button" class="formation-produce-btn" data-produce-start>魔獣を生産</button>
      <div class="formation-produce-panel" data-produce-panel hidden>
        <label class="formation-produce-label" for="formation-produce-amt-${bodySlot}">生産する数</label>
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
          魔獣を1体生産するごとに、食料を<strong>${FOOD_PER_MONSTER_PRODUCE}</strong>個消費します。
        </p>
        <button type="button" class="formation-produce-btn" data-produce-submit>生産する</button>
        <button type="button" class="formation-produce-cancel" data-produce-cancel>戻る</button>
      </div>`
      }
    </div>
  `;
}
