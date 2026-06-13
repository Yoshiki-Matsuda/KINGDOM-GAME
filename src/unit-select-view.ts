import { formedUnitsList, gameState, bodyMonsterCounts, bodySpeeds, getLocalPlayerId } from "./store";
import { getBodyDisplayName, getCardRarityClass } from "./game/characters";
import { isKcUnitReadyToDeploy, recalcUnitStats } from "./game/formation";
import { getPlayerMarches, getPlayerOwnedCards, getMarchLockedCardSlots } from "./shared/game-state";
import type { MarchMission } from "./shared/game-state";
import { escapeHtml, formatTimeHHMMSS } from "./utils";

export interface UnitSelectSnapshot {
  completeUnits: typeof formedUnitsList;
  availableUnits: typeof formedUnitsList;
  returningUnits: MarchMission[];
}

export function getUnitSelectSnapshot(now: number = Date.now()): UnitSelectSnapshot {
  const marches = getPlayerMarches(gameState, getLocalPlayerId());
  const returningMarches = marches.filter((m) => m.kind === "return" && m.arrives_at > now);
  const busyUnitIds = new Set(
    marches
      .filter((m) => m.arrives_at > now)
      .map((m) => m.formed_unit_id)
      .filter((id): id is string => !!id),
  );
  const returningUnitNames = new Set(
    returningMarches
      .map((m) => m.unit_name)
      .filter((name): name is string => !!name),
  );
  const busySlots = getMarchLockedCardSlots(gameState, getLocalPlayerId(), now);
  const completeUnits = formedUnitsList.filter((unit) => isKcUnitReadyToDeploy(unit.indices));
  const unitUsesBusySlot = (unit: (typeof formedUnitsList)[number]) =>
    unit.indices.some((index) => index >= 0 && busySlots.has(index));
  return {
    completeUnits,
    availableUnits: completeUnits.filter(
      (unit) =>
        !busyUnitIds.has(unit.id) &&
        !returningUnitNames.has(unit.name) &&
        !unitUsesBusySlot(unit),
    ),
    returningUnits: returningMarches,
  };
}

export function renderReturningUnits(container: Element, now: number, units: MarchMission[]): void {
  container.innerHTML = "";
  units.forEach((march) => {
    const secLeft = Math.ceil((march.arrives_at - now) / 1000);
    const timeStr = secLeft > 0 ? formatTimeHHMMSS(secLeft) : "00:00:00";
    const row = document.createElement("div");
    row.className = "unit-select-returning-item";
    const name = march.unit_name ?? "遠征隊";
    row.textContent = `${name}（帰還中・残り${timeStr}）`;
    container.appendChild(row);
  });
}

export function renderAvailableUnits(container: Element, units: typeof formedUnitsList): void {
  container.innerHTML = "";
  const owned = getPlayerOwnedCards(gameState, getLocalPlayerId());
  units.forEach((unit) => {
    const label = document.createElement("label");
    label.className = "unit-select-unit-item";
    const memberNames = unit.indices
      .filter((index) => index >= 0)
      .map((index) => {
        const cardId = owned[index] ?? 0;
        return `<span class="${getCardRarityClass(cardId)}">${escapeHtml(getBodyDisplayName(cardId))}</span>`;
      })
      .join("・");
    const { monster_count, avgSpeed } = recalcUnitStats(unit.indices, bodyMonsterCounts, bodySpeeds);
    label.innerHTML = `<input type="radio" name="unit-select-one" data-unit-id="${unit.id}" /> <span>${escapeHtml(unit.name)}（${memberNames}） 魔獣数${monster_count} 速さ${avgSpeed.toFixed(1)}</span>`;
    container.appendChild(label);
  });
  const first = container.querySelector<HTMLInputElement>('input[name="unit-select-one"]');
  const anyChecked = container.querySelector<HTMLInputElement>('input[name="unit-select-one"]:checked');
  if (first && !anyChecked) {
    first.checked = true;
  }
}
