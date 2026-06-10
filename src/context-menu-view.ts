import { isEnemyHomeTile } from "./game/territories";
import type { Territory } from "./store";
import { getLocalPlayerId } from "./store";

function getDifficultyLabel(difficulty: string): string {
  switch (difficulty) {
    case "normal": return "★";
    case "rare": return "★★";
    case "legendary": return "★★★";
    default: return difficulty;
  }
}

export function formatRuinTimeLeft(expiresAt: number): string {
  const now = Date.now();
  const remaining = Math.max(0, expiresAt - now);
  const totalSec = Math.floor(remaining / 1000);
  const min = Math.floor(totalSec / 60);
  const sec = totalSec % 60;
  return `${min}:${sec.toString().padStart(2, "0")}`;
}

export function renderRuinContextMenu(
  territoryId: string,
  territory: Territory,
  attackable: boolean,
): string {
  const ruin = territory.ruin!;
  const timeLeftHtml = ruin.expires_at ? formatRuinTimeLeft(ruin.expires_at) : "";
  const enemyNames = ruin.enemy_names ?? ruin.enemies;

  return `
    <div class="context-menu-ruin">
      <div class="ruin-title">${ruin.formation_name}</div>
      <div class="ruin-difficulty ruin-${ruin.difficulty}">${getDifficultyLabel(ruin.difficulty)}</div>
      ${timeLeftHtml ? `<div class="ruin-time-left" data-expires-at="${ruin.expires_at}">残り ${timeLeftHtml}</div>` : ""}
      <div class="ruin-enemies">
        ${enemyNames.map((name) => `<span class="ruin-enemy">${name}</span>`).join("")}
      </div>
    </div>
    ${attackable ? `<button type="button" data-action="attack" data-to="${territoryId}">挑戦</button>` : ""}
  `;
}

export function renderOwnedTerritoryMenu(
  territoryId: string,
  territory: Territory,
): string {
  const isOwn = territory.owner_id === getLocalPlayerId();
  return `
    <button type="button" data-action="deploy" data-territory="${territoryId}">援軍</button>
    ${isOwn ? `<button type="button" data-action="attack-from" data-territory="${territoryId}">攻撃</button>` : ""}
  `;
}

function formatTerritoryMenuInfo(
  territoryId: string,
  territory: Territory,
  localPlayerId: string,
  players: Record<string, { home_territory_id: string }>,
): string {
  if (
    territory.owner_id
    && isEnemyHomeTile(territoryId, territory, localPlayerId, { players })
  ) {
    return territory.owner_id;
  }
  const statusText = territory.owner_id ? "敵占領" : "中立";
  return `Lv.${territory.level} ${territory.name}（${statusText}）`;
}

export function renderNeutralTerritoryMenu(
  territoryId: string,
  territory: Territory,
  attackable: boolean,
  players: Record<string, { home_territory_id: string }>,
  localPlayerId: string = getLocalPlayerId(),
): string {
  const info = formatTerritoryMenuInfo(territoryId, territory, localPlayerId, players);
  return `
    <div class="context-menu-info">${info}</div>
    ${attackable ? `<button type="button" data-action="attack" data-to="${territoryId}">攻撃</button>` : ""}
  `;
}
