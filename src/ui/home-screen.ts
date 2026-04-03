/**
 * 本拠地画面 — マップと同じマス状エリアに施設を建設する画面
 */

import {
  setCurrentScreen,
  gameState,
  bodyMonsterCounts,
  getHomeFacility,
  setHomeFacility,
  render,
} from "../store";
import { replaceLocalPlayerState } from "../store-actions";
import { DEFAULT_BODY_MONSTER_COUNT } from "../game/characters";
import { initHomeMapView, updateHomeMapView } from "../home-map-view";
import {
  FACILITIES,
  FACILITY_CATEGORIES,
  getFacilitiesByCategory,
  canBuildFacility,
  meetsExpansionRequirement,
  type FacilityId,
  type FacilityCategory,
} from "../game/facilities";
import {
  getExpansionLevel as getExpansionLevelForState,
  getFacilityBonusesForState,
  getFacilitiesForState,
  getHomeGridSize as getHomeGridSizeForState,
  getInventoryForState,
} from "../game/facility-selectors";
import { HOME_COL, HOME_ROW } from "../game/territories";
import { createFacilityBuildState } from "../home-screen-build";
import {
  renderFacilityCard,
  renderFacilityCosts,
  renderPanelHeader,
} from "../home-screen-panel";

let homeEl: HTMLDivElement;
let gridContainer: HTMLDivElement;
let buildMenuEl: HTMLDivElement;
let facilityPanelEl: HTMLDivElement;
let _buildMenuTarget: { col: number; row: number } | null = null;
let _buildMenuDocListener: ((e: MouseEvent) => void) | null = null;
void _buildMenuTarget;
void _buildMenuDocListener;
let homeMapInitialized = false;
let selectedCategory: FacilityCategory = "resource";
let buildingTimerId: number | null = null;
let selectedTile: { col: number; row: number } | null = null;

/** 城マス（本拠地の中心。ここで拡張を行う） */
function isCastleTile(col: number, row: number): boolean {
  return col === HOME_COL && row === HOME_ROW;
}

// 開発用：建設時間を短縮（5秒）
const DEV_MODE = true;
const DEV_BUILD_TIME_SECONDS = 5;

/** 現在の本拠地グリッドサイズを取得（施設ボーナス込み） */
function getHomeGridSize(): number {
  return getHomeGridSizeForState(gameState);
}

/** 現在の本拠地拡張レベルを取得 */
function getExpansionLevel(): number {
  return getExpansionLevelForState(gameState);
}

export function createHomeElement(): HTMLDivElement {
  homeEl = document.createElement("div");
  homeEl.className = "home-screen";
  homeEl.style.display = "none";

  // コンテンツ領域（マップ＋パネル）
  const contentEl = document.createElement("div");
  contentEl.className = "home-screen-content";

  gridContainer = document.createElement("div");
  gridContainer.className = "home-grid-container";
  contentEl.appendChild(gridContainer);

  facilityPanelEl = document.createElement("div");
  facilityPanelEl.className = "facility-panel";
  contentEl.appendChild(facilityPanelEl);

  homeEl.appendChild(contentEl);

  buildMenuEl = document.createElement("div");
  buildMenuEl.className = "home-build-menu";
  buildMenuEl.hidden = true;
  homeEl.appendChild(buildMenuEl);

  return homeEl;
}

export async function showHomeScreen(): Promise<void> {
  setCurrentScreen("home");
  renderHomeContent();
  const gridSize = getHomeGridSize();
  if (!homeMapInitialized && gridContainer) {
    await initHomeMapView(gridContainer, getHomeFacility, onTileClick);
    homeMapInitialized = true;
    updateHomeMapView(getHomeFacility, selectedTile, gridSize);
  } else {
    updateHomeMapView(getHomeFacility, selectedTile, gridSize);
  }
  startBuildingTimer();
  render();
}

export function closeHomeScreen(): void {
  setCurrentScreen("map");
  hideBuildMenu();
  stopBuildingTimer();
  render();
}

function startBuildingTimer(): void {
  stopBuildingTimer();
  buildingTimerId = window.setInterval(() => {
    checkBuildingCompletion();
    updateBuildingTimers();
  }, 1000);
}

function stopBuildingTimer(): void {
  if (buildingTimerId !== null) {
    clearInterval(buildingTimerId);
    buildingTimerId = null;
  }
}

function checkBuildingCompletion(): void {
  const facilities = getFacilitiesForState(gameState);
  const now = Date.now();
  let changed = false;
  const nextFacilities = facilities.map((facility) => {
    if (facility.build_complete_at && facility.build_complete_at <= now) {
      changed = true;
      return {
        ...facility,
        build_complete_at: null,
      };
    }
    return facility;
  });

  if (changed) {
    replaceLocalPlayerState({ facilities: nextFacilities });
    renderFacilityPanel();
  }
}

function updateBuildingTimers(): void {
  const timerEls = facilityPanelEl.querySelectorAll(".facility-building-time");
  const now = Date.now();

  timerEls.forEach(el => {
    const completeAt = parseInt((el as HTMLElement).dataset.completeAt ?? "0", 10);
    if (completeAt > now) {
      const remaining = Math.ceil((completeAt - now) / 1000);
      const mins = Math.floor(remaining / 60);
      const secs = remaining % 60;
      el.textContent = `残り ${mins}:${secs.toString().padStart(2, "0")}`;
    } else {
      el.textContent = "完了！";
    }
  });
}

function _formatBuildTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  if (mins > 0) {
    return `${mins}分${secs}秒`;
  }
  return `${secs}秒`;
}
void _formatBuildTime;

function hideBuildMenu(): void {
  buildMenuEl.hidden = true;
  _buildMenuTarget = null;
  if (_buildMenuDocListener) {
    document.removeEventListener("click", _buildMenuDocListener);
    _buildMenuDocListener = null;
  }
}

function renderHomeContent(): void {
  const homeTroops = gameState.territories.find((t) => t.id === `c_${HOME_COL}_${HOME_ROW}`)?.troops ?? 0;
  const totalMonsters = Array.from({ length: homeTroops }, (_, i) => bodyMonsterCounts[i] ?? DEFAULT_BODY_MONSTER_COUNT).reduce((a, b) => a + b, 0);
  const bonuses = getFacilityBonusesForState(gameState);

  let header = homeEl.querySelector(".home-screen-header");
  if (!header) {
    header = document.createElement("header");
    header.className = "home-screen-header";
    homeEl.insertBefore(header, homeEl.firstChild);
  }
  header.innerHTML = `
    <h1 class="home-screen-title">本拠地</h1>
    <div class="home-screen-stats">
      <span>魔獣数: ${totalMonsters}</span>
      ${bonuses.monsterBonus > 0 ? `<span class="bonus">+${bonuses.monsterBonus}</span>` : ""}
      ${bonuses.monsterPercent > 0 ? `<span class="bonus">+${bonuses.monsterPercent}%</span>` : ""}
    </div>
    <button type="button" class="home-screen-back" data-home-back>マップへ戻る</button>
  `;
  header.querySelector("[data-home-back]")?.addEventListener("click", () => {
    closeHomeScreen();
    render();
  });

  renderFacilityPanel();
}

function findBuiltFacility(
  facilityId: FacilityId,
  facilities: ReturnType<typeof getFacilitiesForState>,
  tile: { col: number; row: number } | null,
) {
  return facilities.find((facility) => {
    if (facility.facility_id !== facilityId) return false;
    if (!tile) return true;
    if (facility.position) {
      return facility.position.col === tile.col && facility.position.row === tile.row;
    }
    return getHomeFacility(tile.col, tile.row) === facilityId;
  });
}

function renderFacilityBuildButton(
  facilityId: FacilityId,
  level: number,
  label: string,
  enabled: boolean,
): string {
  return `<button type="button" class="facility-build-btn" data-facility="${facilityId}" data-level="${level}" ${enabled ? "" : "disabled"}>${label}</button>`;
}

function renderFacilityPanel(): void {
  const inventory = getInventoryForState(gameState);
  const facilities = getFacilitiesForState(gameState);
  const expansionLevel = getExpansionLevel();

  if (!selectedTile) {
    facilityPanelEl.style.display = "none";
    return;
  }
  facilityPanelEl.style.display = "";

  if (isCastleTile(selectedTile.col, selectedTile.row)) {
    facilityPanelEl.innerHTML = renderPanelHeader({
      title: "🏰 城",
      subtitle: "本拠地の中心",
    });
    bindFacilityPanelListeners();
    updateBuildingTimers();
    return;
  }

  const tileFacilityId = getHomeFacility(selectedTile.col, selectedTile.row);
  if (tileFacilityId) {
    const facility = FACILITIES[tileFacilityId as FacilityId];
    if (facility) {
      const built = findBuiltFacility(facility.id, facilities, selectedTile);
      const currentLevel = built?.level ?? 0;
      const isBuilding = !!(built?.build_complete_at && built.build_complete_at > Date.now());
      const nextLevel = currentLevel + 1;
      const levelDef = facility.levels[nextLevel - 1];
      const isMaxLevel = currentLevel >= facility.maxLevel;
      const canBuildNow = !!(levelDef && canBuildFacility(facility.id, nextLevel, inventory, expansionLevel) && !isBuilding);

      const cardHtml = renderFacilityCard({
        icon: facility.icon,
        name: facility.name,
        description: facility.description,
        currentLevel,
        isBuilding,
        isMaxLevel,
        buildCompleteAt: built?.build_complete_at,
        nextEffectHtml: !isMaxLevel && levelDef ? levelDef.description : undefined,
        costHtml: !isMaxLevel && levelDef ? renderFacilityCosts(levelDef.cost, inventory) : undefined,
        buttonHtml: !isMaxLevel && levelDef
          ? renderFacilityBuildButton(
              facility.id,
              nextLevel,
              isBuilding ? "建設中" : "レベルアップ",
              canBuildNow,
            )
          : undefined,
      });

      facilityPanelEl.innerHTML = `
        ${renderPanelHeader({
          title: "施設レベルアップ",
          subtitle: `📍 (${selectedTile.col - 21}, ${selectedTile.row - 21}) の ${facility.name}`,
        })}
        <div class="facility-list facility-list--expansion">
          ${cardHtml}
        </div>
      `;
      bindFacilityPanelListeners();
      updateBuildingTimers();
      return;
    }
  }

  const categoryFacilities = getFacilitiesByCategory(selectedCategory).filter(
    (facility) => !facilities.some((builtFacility) => builtFacility.facility_id === facility.id),
  );
  const categoryButtonsHtml = FACILITY_CATEGORIES.map((category) => `
    <button type="button" class="facility-category-btn ${category.id === selectedCategory ? "is-active" : ""}" data-category="${category.id}">
      <span class="cat-icon">${category.icon}</span>
      <span class="cat-name">${category.name}</span>
    </button>
  `).join("");

  const cardsHtml = categoryFacilities.map((facility) => {
    const built = findBuiltFacility(facility.id, facilities, selectedTile);
    const currentLevel = built?.level ?? 0;
    const isBuilding = !!(built?.build_complete_at && built.build_complete_at > Date.now());
    const nextLevel = currentLevel + 1;
    const levelDef = facility.levels[nextLevel - 1];
    const meetsRequirement = meetsExpansionRequirement(facility.id, expansionLevel);
    const isMaxLevel = currentLevel >= facility.maxLevel;
    const canBuildNow = !!(levelDef && canBuildFacility(facility.id, nextLevel, inventory, expansionLevel) && !isBuilding);
    const requiredExpLevel = facility.requiredExpansionLevel ?? 0;

    return renderFacilityCard({
      icon: facility.icon,
      name: facility.name,
      description: facility.description,
      currentLevel,
      isBuilding,
      isMaxLevel,
      buildCompleteAt: built?.build_complete_at,
      nextEffectHtml: meetsRequirement && !isMaxLevel && levelDef ? levelDef.description : undefined,
      costHtml: meetsRequirement && !isMaxLevel && levelDef ? renderFacilityCosts(levelDef.cost, inventory) : undefined,
      buttonHtml: meetsRequirement && !isMaxLevel && levelDef
        ? renderFacilityBuildButton(
            facility.id,
            nextLevel,
            isBuilding ? "建設中" : currentLevel === 0 ? "建設" : "レベルアップ",
            canBuildNow,
          )
        : undefined,
      extraClasses: !meetsRequirement ? ["is-locked"] : [],
    }).replace(
      '<div class="facility-desc">',
      `${!meetsRequirement ? `<div class="facility-locked">🔒 本拠地拡張 Lv.${requiredExpLevel} 必要</div>` : ""}<div class="facility-desc">`,
    );
  }).join("");

  facilityPanelEl.innerHTML = `
    ${renderPanelHeader({
      title: "施設建設",
      subtitle: `📍 (${selectedTile.col - 21}, ${selectedTile.row - 21}) を選択中`,
    })}
    <div class="facility-categories">
      ${categoryButtonsHtml}
    </div>
    <div class="facility-list">
      ${cardsHtml}
    </div>
  `;

  bindFacilityPanelListeners();
  updateBuildingTimers();
}

function bindFacilityPanelListeners(): void {
  facilityPanelEl.querySelector(".deselect-tile-btn")?.addEventListener("click", () => {
    selectedTile = null;
    renderFacilityPanel();
    updateHomeMapView(getHomeFacility, selectedTile, getHomeGridSize());
    render();
  });

  facilityPanelEl.querySelectorAll(".facility-category-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      selectedCategory = (btn as HTMLElement).dataset.category as FacilityCategory;
      renderFacilityPanel();
    });
  });

  facilityPanelEl.querySelectorAll(".facility-build-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      const facilityId = (btn as HTMLElement).dataset.facility;
      const level = parseInt((btn as HTMLElement).dataset.level ?? "1", 10);
      if (facilityId) {
        buildFacility(facilityId as FacilityId, level);
      }
    });
  });
}

function buildFacility(facilityId: FacilityId, level: number): void {
  const result = createFacilityBuildState({
    facilityId,
    level,
    selectedTile,
    existingFacilities: getFacilitiesForState(gameState),
    inventory: getInventoryForState(gameState),
    expansionLevel: getExpansionLevel(),
    getHomeFacility,
    isCastleTile,
    devMode: DEV_MODE,
    devBuildTimeSeconds: DEV_BUILD_TIME_SECONDS,
  });
  if (!result) return;

  if (result.placedFacility) {
    setHomeFacility(
      result.placedFacility.col,
      result.placedFacility.row,
      result.placedFacility.facilityId,
    );
  }

  replaceLocalPlayerState({
    inventory: result.inventory,
    facilities: result.facilities,
  });

  // 選択解除
  selectedTile = null;

  renderFacilityPanel();
  updateHomeMapView(getHomeFacility, selectedTile, getHomeGridSize());
  render();
}

function onTileClick(col: number, row: number, facility: string | null, _screenX: number, _screenY: number): void {
  // 城マス：拡張専用。常に選択可能
  if (isCastleTile(col, row)) {
    selectedTile = { col, row };
    renderFacilityPanel();
    updateHomeMapView(getHomeFacility, selectedTile, getHomeGridSize());
    render();
    return;
  }
  // マスを選択（施設あり→レベルアップ、空き→建設）
  selectedTile = { col, row };
  // 施設があるマスの場合、その施設のカテゴリを自動選択
  if (facility) {
    const def = FACILITIES[facility as FacilityId];
    if (def) selectedCategory = def.category;
  }
  renderFacilityPanel();
  updateHomeMapView(getHomeFacility, selectedTile, getHomeGridSize());
  render();
}

export function isHomeScreenVisible(): boolean {
  return homeEl.style.display !== "none";
}
