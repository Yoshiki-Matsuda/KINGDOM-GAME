/**
 * 編成画面UI
 */

import {
  bodyEnergies, bodySpeeds,
  formedUnitsList, setFormedUnitsList,
  formationSelected, setFormationSelected,
  getNextFormedUnitId,
  render,
} from "../store";
import { DEFAULT_BODY_ENERGY, DEFAULT_BODY_SPEED, getBodyDisplayName } from "../game/characters";
import { getCharacterSkills } from "../game/skills";
import { getHomeTroops, validateFormedUnits } from "../game/formation";
import { escapeHtml } from "../utils";

let formationEl: HTMLDivElement;

export function createFormationElement(): HTMLDivElement {
  formationEl = document.createElement("div");
  formationEl.className = "formation-overlay";
  formationEl.innerHTML = `
    <div class="formation-modal">
      <div class="formation-title">編成画面</div>
      <div class="formation-desc">キャラ3体を選んで1ユニットに編成します</div>
      <div class="formation-troops" data-formation-troops>本拠地: 0 体</div>
      <div class="formation-cards" data-formation-cards></div>
      <div class="formation-building" data-formation-building>
        <span class="formation-building-label">編成中:</span>
        <span data-formation-selected>0 / 3 体選択中</span>
        <button type="button" class="formation-confirm-unit" data-formation-confirm-unit disabled>この3体で編成</button>
      </div>
      <div class="formation-list" data-formation-list></div>
      <button type="button" class="formation-close" data-formation-close>編成画面を閉じる</button>
    </div>
  `;
  setupFormationScreen();
  return formationEl;
}

export function showFormationScreen(): void {
  validateFormedUnits();
  formationEl.classList.add("is-open");
  renderFormationContent();
  (formationEl.querySelector("[data-formation-close]") as HTMLElement)?.focus();
}

export function closeFormationScreen(): void {
  formationEl.classList.remove("is-open");
  setFormationSelected([]);
}

function renderFormationContent(): void {
  const homeTroops = getHomeTroops();
  const usedSet = new Set(formedUnitsList.flatMap((u) => u.indices));
  const troopsEl = formationEl.querySelector("[data-formation-troops]")!;
  const cardsEl = formationEl.querySelector("[data-formation-cards]")!;
  const selectedEl = formationEl.querySelector("[data-formation-selected]")!;
  const confirmBtn = formationEl.querySelector<HTMLButtonElement>("[data-formation-confirm-unit]")!;
  const listEl = formationEl.querySelector("[data-formation-list]")!;

  troopsEl.textContent = `本拠地: ${homeTroops} 体`;
  selectedEl.textContent = `${formationSelected.length} / 3 体選択中`;
  confirmBtn.disabled = formationSelected.length !== 3;

  cardsEl.innerHTML = "";
  for (let i = 0; i < homeTroops; i++) {
    const used = usedSet.has(i);
    const selected = formationSelected.includes(i);
    const canToggle = !used && (selected || formationSelected.length < 3);
    const card = document.createElement("button");
    card.type = "button";
    card.className = "formation-card" + (used ? " is-used" : "") + (selected ? " is-selected" : "");
    card.dataset.formationIndex = String(i);
    const energy = bodyEnergies[i] ?? DEFAULT_BODY_ENERGY;
    const speed = bodySpeeds[i] ?? DEFAULT_BODY_SPEED;
    const skills = getCharacterSkills(i);

    const skillNames: string[] = [];
    if (skills.passive) skillNames.push(`[P]${skills.passive.name}`);
    skillNames.push(`[A]${skills.active.name}`);
    if (skills.unique) skillNames.push(`[U]${skills.unique.name}`);

    card.innerHTML = `
      <div class="formation-card-name">${escapeHtml(getBodyDisplayName(i))}</div>
      <div class="formation-card-stats">エナジー${energy} SPEED${speed}</div>
      <div class="formation-card-skills">${escapeHtml(skillNames.join(" "))}</div>
    `;
    if (!canToggle) card.disabled = true;
    cardsEl.appendChild(card);
  }

  listEl.innerHTML = "";
  formedUnitsList.forEach((u) => {
    const row = document.createElement("div");
    row.className = "formation-list-item";
    const memberNames = u.indices.map((i) => getBodyDisplayName(i)).join("・");
    row.innerHTML = `
      <span class="formation-list-name">${escapeHtml(u.name)}（${escapeHtml(memberNames)}） エナジー${u.energy} SPEED${u.avgSpeed.toFixed(1)}</span>
      <button type="button" class="formation-dissolve" data-dissolve-id="${u.id}">解体</button>
    `;
    listEl.appendChild(row);
  });
}

function setupFormationScreen(): void {
  formationEl.querySelector("[data-formation-confirm-unit]")?.addEventListener("click", () => {
    if (formationSelected.length !== 3) return;
    const id = `unit-${getNextFormedUnitId()}`;
    const name = `ユニット${formedUnitsList.length + 1}`;
    const energy =
      (bodyEnergies[formationSelected[0]] ?? DEFAULT_BODY_ENERGY) +
      (bodyEnergies[formationSelected[1]] ?? DEFAULT_BODY_ENERGY) +
      (bodyEnergies[formationSelected[2]] ?? DEFAULT_BODY_ENERGY);
    const avgSpeed = (
      (bodySpeeds[formationSelected[0]] ?? DEFAULT_BODY_SPEED) +
      (bodySpeeds[formationSelected[1]] ?? DEFAULT_BODY_SPEED) +
      (bodySpeeds[formationSelected[2]] ?? DEFAULT_BODY_SPEED)
    ) / 3;
    formedUnitsList.push({
      id,
      name,
      indices: [formationSelected[0], formationSelected[1], formationSelected[2]],
      energy,
      avgSpeed,
    });
    setFormationSelected([]);
    renderFormationContent();
  });

  formationEl.querySelector("[data-formation-close]")?.addEventListener("click", () => {
    closeFormationScreen();
    render();
  });

  const cardsEl = formationEl.querySelector("[data-formation-cards]")!;
  cardsEl.addEventListener("click", (e) => {
    const card = (e.target as HTMLElement).closest<HTMLButtonElement>("button[data-formation-index]");
    if (!card || card.disabled) return;
    const i = parseInt(card.dataset.formationIndex ?? "-1", 10);
    if (i < 0) return;
    if (formationSelected.includes(i)) {
      setFormationSelected(formationSelected.filter((x) => x !== i));
    } else if (formationSelected.length < 3) {
      setFormationSelected([...formationSelected, i].sort((a, b) => a - b));
    }
    renderFormationContent();
  });

  const listEl = formationEl.querySelector("[data-formation-list]")!;
  listEl.addEventListener("click", (e) => {
    const btn = (e.target as HTMLElement).closest<HTMLButtonElement>("button[data-dissolve-id]");
    if (!btn) return;
    const dissolveId = btn.dataset.dissolveId;
    if (!dissolveId) return;
    setFormedUnitsList(formedUnitsList.filter((x) => x.id !== dissolveId));
    renderFormationContent();
  });

  formationEl.addEventListener("click", (e) => {
    if (e.target === formationEl) {
      closeFormationScreen();
      render();
    }
  });
}
