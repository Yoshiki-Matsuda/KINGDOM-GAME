/**
 * 魔獣の実効ステータス（マスタ + 配分ボーナス + 施設 + 強化★）
 */

import { getCharacterStats, type CardStats } from "./characters";
import { calculateFacilityBonuses } from "./facilities";
import {
  DEFAULT_PLAYER_ID,
  type CardStatBonuses,
  type GameState,
  type PlayerData,
} from "../shared/game-state";

export type { CardStatBonuses };

export type AllocatableStatKey = "speed" | "attack" | "intelligence" | "defense" | "magic_defense";

const EMPTY_BONUSES: CardStatBonuses = {
  speed: 0,
  attack: 0,
  intelligence: 0,
  defense: 0,
  magic_defense: 0,
};

export function getPlayerCardStatBonuses(
  state: GameState,
  bodySlot: number,
  playerId: string = DEFAULT_PLAYER_ID,
): CardStatBonuses {
  const raw = state.players[playerId]?.card_stat_bonuses?.[bodySlot];
  if (!raw) return { ...EMPTY_BONUSES };
  return {
    speed: raw.speed ?? 0,
    attack: raw.attack ?? 0,
    intelligence: raw.intelligence ?? 0,
    defense: raw.defense ?? 0,
    magic_defense: raw.magic_defense ?? 0,
  };
}

export function getPlayerCardStatusPoints(
  state: GameState,
  bodySlot: number,
  playerId: string = DEFAULT_PLAYER_ID,
): number {
  return state.players[playerId]?.card_status_points?.[bodySlot] ?? 0;
}

export function getPlayerCardEnhanced(
  state: GameState,
  bodySlot: number,
  playerId: string = DEFAULT_PLAYER_ID,
): boolean {
  return state.players[playerId]?.card_enhanced?.[bodySlot] ?? false;
}

function facilityBonusesForPlayer(player: PlayerData | undefined) {
  return calculateFacilityBonuses(
    new Map((player?.facilities ?? []).map((f) => [f.facility_id as never, f.level])),
  );
}

function computeCardStats(
  cardId: number,
  bonuses: CardStatBonuses,
  facilitySpeedBonus: number,
  enhanced: boolean,
): CardStats {
  const base = getCharacterStats(cardId);
  let stats: CardStats = {
    ...base,
    speed: base.speed + bonuses.speed + facilitySpeedBonus,
    attack: base.attack + bonuses.attack,
    intelligence: base.intelligence + bonuses.intelligence,
    defense: base.defense + bonuses.defense,
    magicDefense: base.magicDefense + bonuses.magic_defense,
  };

  if (enhanced) {
    const mul = (v: number) => Math.round(v * 1.1);
    stats = {
      ...stats,
      monster_count: mul(stats.monster_count),
      speed: mul(stats.speed),
      attack: mul(stats.attack),
      intelligence: mul(stats.intelligence),
      defense: mul(stats.defense),
      magicDefense: mul(stats.magicDefense),
      occupationPower: mul(stats.occupationPower),
    };
  }

  return stats;
}

/** 配分ボーナスを除いた実効値（マスタ + 施設 + 強化★） */
export function getCardCoreStats(
  cardId: number,
  bodySlot: number,
  state: GameState,
  playerId: string = DEFAULT_PLAYER_ID,
): CardStats {
  const player = state.players[playerId];
  const facility = facilityBonusesForPlayer(player);
  const enhanced = getPlayerCardEnhanced(state, bodySlot, playerId);
  return computeCardStats(cardId, EMPTY_BONUSES, facility.speedBonus, enhanced);
}

/** マスタ・配分・施設・強化★を反映した戦闘・移動用ステータス */
export function getEffectiveCardStats(
  cardId: number,
  bodySlot: number,
  state: GameState,
  playerId: string = DEFAULT_PLAYER_ID,
): CardStats {
  const bonuses = getPlayerCardStatBonuses(state, bodySlot, playerId);
  const player = state.players[playerId];
  const facility = facilityBonusesForPlayer(player);
  const enhanced = getPlayerCardEnhanced(state, bodySlot, playerId);
  return computeCardStats(cardId, bonuses, facility.speedBonus, enhanced);
}

export function statValueFromCard(stats: CardStats, key: AllocatableStatKey): number {
  if (key === "magic_defense") return stats.magicDefense;
  return stats[key];
}

/** 振り分け済みボーナスが実効値に与えている増分（強化★込み） */
export function getDisplayedAllocationBonus(
  cardId: number,
  bodySlot: number,
  state: GameState,
  key: AllocatableStatKey,
  playerId: string = DEFAULT_PLAYER_ID,
): number {
  const core = statValueFromCard(getCardCoreStats(cardId, bodySlot, state, playerId), key);
  const effective = statValueFromCard(getEffectiveCardStats(cardId, bodySlot, state, playerId), key);
  return Math.max(0, effective - core);
}

/** 振り分けダイアログ用「基礎 (+振り分け)」。(+n) 部分は HTML スパン で着色 */
export function formatStatAllocationHtml(
  coreValue: number,
  allocatedBonus: number,
  pendingAdd = 0,
): string {
  const bonus = allocatedBonus + pendingAdd;
  if (bonus > 0) {
    return `${coreValue} <span class="formation-stat-alloc-bonus">(+${bonus})</span>`;
  }
  return String(coreValue);
}

export function cardStatsToPayload(stats: CardStats) {
  return {
    monster_count: stats.monster_count,
    speed: stats.speed,
    attack: stats.attack,
    intelligence: stats.intelligence,
    defense: stats.defense,
    magic_defense: stats.magicDefense,
    range: stats.range,
    cost: stats.cost,
    occupation_power: stats.occupationPower,
  };
}
