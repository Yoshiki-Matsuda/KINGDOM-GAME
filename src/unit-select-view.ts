import { formedUnitsList, travelingUnits } from "./store";
import { getBodyDisplayName } from "./game/characters";
import { escapeHtml, formatTimeHHMMSS } from "./utils";

export interface UnitSelectSnapshot {
  completeUnits: typeof formedUnitsList;
  availableUnits: typeof formedUnitsList;
  returningUnits: typeof travelingUnits;
}

export function getUnitSelectSnapshot(now: number = Date.now()): UnitSelectSnapshot {
  const travelingUnitIds = new Set(travelingUnits.map((traveling) => traveling.unitId));
  const completeUnits = formedUnitsList.filter((unit) => unit.indices.every((index) => index >= 0));
  return {
    completeUnits,
    availableUnits: completeUnits.filter((unit) => !travelingUnitIds.has(unit.id)),
    returningUnits: travelingUnits.filter(
      (traveling) => traveling.actionType === "return" && (traveling.arrivalTime - now) / 1000 > 0,
    ),
  };
}

export function renderReturningUnits(container: Element, now: number, units: typeof travelingUnits): void {
  container.innerHTML = "";
  units.forEach((traveling) => {
    const secLeft = Math.ceil((traveling.arrivalTime - now) / 1000);
    const timeStr = secLeft > 0 ? formatTimeHHMMSS(secLeft) : "00:00:00";
    const row = document.createElement("div");
    row.className = "unit-select-returning-item";
    row.textContent = `${traveling.unitName}（帰還中・残り${timeStr}）`;
    container.appendChild(row);
  });
}

export function renderAvailableUnits(container: Element, units: typeof formedUnitsList): void {
  container.innerHTML = "";
  units.forEach((unit) => {
    const label = document.createElement("label");
    label.className = "unit-select-unit-item";
    const memberNames = unit.indices.map((index) => getBodyDisplayName(index)).join("・");
    label.innerHTML = `<input type="radio" name="unit-select-one" data-unit-id="${unit.id}" /> <span>${escapeHtml(unit.name)}（${escapeHtml(memberNames)}） エナジー${unit.energy} SPEED${unit.avgSpeed.toFixed(1)}</span>`;
    container.appendChild(label);
  });
}
