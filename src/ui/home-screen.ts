/**
 * 本拠地画面 — マップと同じマス状エリアに施設を建設する画面
 */

import {
  setCurrentScreen,
  gameState,
  bodyEnergies,
  getHomeFacility,
  setHomeFacility,
  render,
} from "../store";
import { DEFAULT_BODY_ENERGY } from "../game/characters";
import { initHomeMapView, updateHomeMapView } from "../home-map-view";
import {
  FACILITIES,
  FACILITY_CATEGORIES,
  getFacilitiesByCategory,
  canBuildFacility,
  calculateFacilityBonuses,
  meetsExpansionRequirement,
  type FacilityId,
  type FacilityCategory,
} from "../game/facilities";
import { getItem, getItemCount } from "../game/items";

let homeEl: HTMLDivElement;
let gridContainer: HTMLDivElement;
let buildMenuEl: HTMLDivElement;
let facilityPanelEl: HTMLDivElement;
let _buildMenuTarget: { col: number; row: number } | null = null;
let _buildMenuDocListener: ((e: MouseEvent) => void) | null = null;
void _buildMenuTarget;
void _buildMenuDocListener;
let homeMapInitialized = false;
let selectedCategory: FacilityCategory = "production";
let buildingTimerId: number | null = null;
let selectedTile: { col: number; row: number } | null = null;

// 開発用：建設時間を短縮（5秒）
const DEV_MODE = true;
const DEV_BUILD_TIME_SECONDS = 5;

/** 建設済み施設をMapで取得 */
function getBuiltFacilitiesMap(): Map<FacilityId, number> {
  const builtFacilities = new Map<FacilityId, number>();
  for (const f of gameState.facilities ?? []) {
    if (!f.build_complete_at || f.build_complete_at <= Date.now()) {
      builtFacilities.set(f.facility_id as FacilityId, f.level);
    }
  }
  return builtFacilities;
}

/** 現在の本拠地グリッドサイズを取得（施設ボーナス込み） */
function getHomeGridSize(): number {
  return calculateFacilityBonuses(getBuiltFacilitiesMap()).homeSize;
}

/** 現在の本拠地拡張レベルを取得 */
function getExpansionLevel(): number {
  return calculateFacilityBonuses(getBuiltFacilitiesMap()).expansionLevel;
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
  const facilities = gameState.facilities ?? [];
  const now = Date.now();
  let changed = false;

  for (const facility of facilities) {
    if (facility.build_complete_at && facility.build_complete_at <= now) {
      facility.build_complete_at = null;
      changed = true;
    }
  }

  if (changed) {
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
  const homeTroops = gameState.territories.find((t) => t.id === "c_24_24")?.troops ?? 0;
  const totalEnergy = Array.from({ length: homeTroops }, (_, i) => bodyEnergies[i] ?? DEFAULT_BODY_ENERGY).reduce((a, b) => a + b, 0);

  // 施設ボーナス計算
  const builtFacilities = new Map<FacilityId, number>();
  for (const f of gameState.facilities ?? []) {
    if (!f.build_complete_at || f.build_complete_at <= Date.now()) {
      builtFacilities.set(f.facility_id as FacilityId, f.level);
    }
  }
  const bonuses = calculateFacilityBonuses(builtFacilities);

  let header = homeEl.querySelector(".home-screen-header");
  if (!header) {
    header = document.createElement("header");
    header.className = "home-screen-header";
    homeEl.insertBefore(header, homeEl.firstChild);
  }
  header.innerHTML = `
    <h1 class="home-screen-title">本拠地</h1>
    <div class="home-screen-stats">
      <span>エナジー: ${totalEnergy}</span>
      ${bonuses.energyBonus > 0 ? `<span class="bonus">+${bonuses.energyBonus}</span>` : ""}
      ${bonuses.energyPercent > 0 ? `<span class="bonus">+${bonuses.energyPercent}%</span>` : ""}
    </div>
    <button type="button" class="home-screen-back" data-home-back>マップへ戻る</button>
  `;
  header.querySelector("[data-home-back]")?.addEventListener("click", () => {
    closeHomeScreen();
    render();
  });

  renderFacilityPanel();
}

function renderFacilityPanel(): void {
  const inventory = gameState.inventory ?? [];
  const facilities = gameState.facilities ?? [];
  const expansionLevel = getExpansionLevel();

  // 現在のカテゴリの施設一覧
  const categoryFacilities = getFacilitiesByCategory(selectedCategory);

  // マス未選択時はパネルを非表示
  if (!selectedTile) {
    facilityPanelEl.style.display = "none";
    return;
  }
  facilityPanelEl.style.display = "";

  // マス選択中は施設リストを表示
  facilityPanelEl.innerHTML = `
    <div class="facility-panel-header">
      <h2>施設建設</h2>
      <div class="selected-tile-info">📍 (${selectedTile.col - 21}, ${selectedTile.row - 21}) を選択中
        <button type="button" class="deselect-tile-btn">✕</button>
      </div>
    </div>
    <div class="facility-categories">
      ${FACILITY_CATEGORIES.map(cat => `
        <button type="button" class="facility-category-btn ${cat.id === selectedCategory ? 'is-active' : ''}" data-category="${cat.id}">
          <span class="cat-icon">${cat.icon}</span>
          <span class="cat-name">${cat.name}</span>
        </button>
      `).join("")}
    </div>
    <div class="facility-list">
      ${categoryFacilities.map(f => {
        const built = facilities.find(b => b.facility_id === f.id);
        const currentLevel = built?.level ?? 0;
        const isBuilding = built?.build_complete_at && built.build_complete_at > Date.now();
        const nextLevel = currentLevel + 1;
        const levelDef = f.levels[nextLevel - 1];
        const meetsRequirement = meetsExpansionRequirement(f.id, expansionLevel);
        const canBuild = levelDef && canBuildFacility(f.id, nextLevel, inventory, expansionLevel);
        const isMaxLevel = currentLevel >= f.maxLevel;
        const isNewBuild = currentLevel === 0;
        const canBuildNow = canBuild && !isBuilding;
        const requiredExpLevel = f.requiredExpansionLevel ?? 0;

        return `
          <div class="facility-card ${isBuilding ? 'is-building' : ''} ${isMaxLevel ? 'is-max' : ''} ${!meetsRequirement ? 'is-locked' : ''}">
            <div class="facility-icon">${f.icon}</div>
            <div class="facility-info">
              <div class="facility-name">${f.name}</div>
              <div class="facility-level">Lv.${currentLevel}${isMaxLevel ? ' (MAX)' : ''}</div>
              <div class="facility-desc">${f.description}</div>
              ${!meetsRequirement ? `
                <div class="facility-locked">
                  🔒 本拠地拡張 Lv.${requiredExpLevel} 必要
                </div>
              ` : ''}
              ${isBuilding && built?.build_complete_at ? `
                <div class="facility-building">
                  建設中 <span class="facility-building-time" data-complete-at="${built.build_complete_at}"></span>
                </div>
              ` : ''}
              ${meetsRequirement && !isMaxLevel && levelDef ? `
                <div class="facility-next">
                  <div class="next-effect">${levelDef.description}</div>
                  <div class="next-cost">
                    ${levelDef.cost.map(c => {
                      const item = getItem(c.itemId);
                      const have = getItemCount(inventory, c.itemId);
                      const enough = have >= c.count;
                      return `<span class="cost-item ${enough ? '' : 'not-enough'}">${item?.icon ?? '?'}${have}/${c.count}</span>`;
                    }).join("")}
                  </div>
                  <button type="button" class="facility-build-btn" data-facility="${f.id}" data-level="${nextLevel}" ${canBuildNow ? '' : 'disabled'}>
                    ${isBuilding ? '建設中' : isNewBuild ? '建設' : 'レベルアップ'}
                  </button>
                </div>
              ` : ''}
            </div>
          </div>
        `;
      }).join("")}
    </div>
  `;

  // 選択解除ボタン
  facilityPanelEl.querySelector(".deselect-tile-btn")?.addEventListener("click", () => {
    selectedTile = null;
    renderFacilityPanel();
    updateHomeMapView(getHomeFacility, selectedTile, getHomeGridSize());
    render();
  });

  // カテゴリボタンのイベント
  facilityPanelEl.querySelectorAll(".facility-category-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      selectedCategory = (btn as HTMLElement).dataset.category as FacilityCategory;
      renderFacilityPanel();
    });
  });

  // 建設ボタンのイベント
  facilityPanelEl.querySelectorAll(".facility-build-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      const facilityId = (btn as HTMLElement).dataset.facility;
      const level = parseInt((btn as HTMLElement).dataset.level ?? "1", 10);
      if (facilityId) {
        buildFacility(facilityId as FacilityId, level);
      }
    });
  });

  // 初期表示時に残り時間を更新
  updateBuildingTimers();
}

function buildFacility(facilityId: FacilityId, level: number): void {
  // 新規建設（Lv1）の場合、タイル選択が必要
  if (level === 1 && !selectedTile) return;

  const facility = FACILITIES[facilityId];
  if (!facility) return;

  const levelDef = facility.levels[level - 1];
  if (!levelDef) return;

  const inventory = gameState.inventory ?? [];
  if (!canBuildFacility(facilityId, level, inventory)) return;

  // インベントリから素材を消費（ローカル更新）
  const newInventory = [...inventory];
  for (const cost of levelDef.cost) {
    const idx = newInventory.findIndex(i => i.item_id === cost.itemId);
    if (idx >= 0) {
      newInventory[idx] = {
        ...newInventory[idx],
        count: newInventory[idx].count - cost.count,
      };
      if (newInventory[idx].count <= 0) {
        newInventory.splice(idx, 1);
      }
    }
  }

  // 施設を追加/更新
  const facilities = [...(gameState.facilities ?? [])];
  const existingIdx = facilities.findIndex(f => f.facility_id === facilityId);
  const buildTimeSeconds = DEV_MODE ? DEV_BUILD_TIME_SECONDS : levelDef.buildTime;
  const buildCompleteAt = Date.now() + buildTimeSeconds * 1000;

  if (existingIdx >= 0) {
    // 既存施設のレベルアップ
    facilities[existingIdx] = {
      ...facilities[existingIdx],
      level,
      build_complete_at: buildCompleteAt,
    };
  } else {
    // 新規建設 - 選択したタイルに配置
    facilities.push({
      facility_id: facilityId,
      level,
      build_complete_at: buildCompleteAt,
      position: selectedTile ? { col: selectedTile.col, row: selectedTile.row } : undefined,
    });

    // ホームマップにも反映
    if (selectedTile) {
      setHomeFacility(selectedTile.col, selectedTile.row, facilityId);
    }
  }

  // ローカル状態更新
  gameState.inventory = newInventory;
  gameState.facilities = facilities;

  // 選択解除
  selectedTile = null;

  renderFacilityPanel();
  updateHomeMapView(getHomeFacility, selectedTile, getHomeGridSize());
  render();
}

function onTileClick(col: number, row: number, facility: string | null, _screenX: number, _screenY: number): void {
  // 既に施設がある場合は選択解除
  if (facility) {
    selectedTile = null;
  } else {
    // 空きタイルをクリック → 選択状態にする
    selectedTile = { col, row };
  }
  renderFacilityPanel();
  updateHomeMapView(getHomeFacility, selectedTile, getHomeGridSize());
  render();
}

export function isHomeScreenVisible(): boolean {
  return homeEl.style.display !== "none";
}
