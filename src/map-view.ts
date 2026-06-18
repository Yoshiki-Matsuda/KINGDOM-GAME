import {
    Application,
    Container,
    Graphics,
    Text,
    Assets,
    Texture,
    Matrix,
    FederatedPointerEvent,
} from "pixi.js";
import type { GameState } from "./store";
import { gameState, getLocalPlayerId, isPlayerIdentityResolved } from "./store";
import {
    formatTerritoryId,
    getPlayerHomeTerritoryId,
    getWorldGridCols,
    getWorldGridRows,
    isAiOwnerId,
    isEnemyHomeTile,
    isAiHomeTile,
    isPlayerHomeTile,
    isWithinWorldGrid,
    tryParseTerritoryId,
} from "./game/territories";
import { isRiverLevel, pickRiverAxis, type RiverAxis } from "./game/river-orientation";
import {
    coordToScreen,
    diamondPoints,
    screenToCoord,
} from "./map-isometric";
import { formatTimeHHMMSS } from "./utils";
export { getWorldGridCols as GRID_COLS, getWorldGridRows as GRID_ROWS, HOME_TERRITORY_ID, parseTerritoryId } from "./game/territories";

// アイソメトリック: マスは◇型。45度回転して斜め上から見下ろす視点
// タイルの幅・高さ（◇の横・縦の長さ）
const TILE_WIDTH = 56;
const TILE_HEIGHT = 28;
const TILE_DIMENSIONS = { width: TILE_WIDTH, height: TILE_HEIGHT } as const;

// 地形テクスチャキャッシュ (level 1-6 に対応)
const TERRAIN_TEXTURES: (Texture | null)[] = Array.from({ length: 10 }, () => null);
// 本拠地用の城テクスチャ
let CASTLE_TEXTURE: Texture | null = null;

/** 遺跡の見た目定義（フォーメーション名から） */
interface RuinVisual {
    color: number;
    icon: string;
    name: string;
}

const RUIN_VISUALS: Record<string, RuinVisual> = {
    // ノーマル遺跡
    "石の番人": { color: 0x302a20, icon: "🗿", name: "石の番人" },
    "亡霊の群れ": { color: 0x1e1a30, icon: "👻", name: "亡霊の群れ" },
    "骸骨兵団": { color: 0x1a1a1a, icon: "💀", name: "骸骨兵団" },
    "スライムの巣": { color: 0x1a2a1a, icon: "🟢", name: "スライムの巣" },
    "蜘蛛の巣窟": { color: 0x201a12, icon: "🕸️", name: "蜘蛛の巣窟" },
    "炎の回廊": { color: 0x381808, icon: "🔥", name: "炎の回廊" },
    "氷結の間": { color: 0x142830, icon: "❄️", name: "氷結の間" },
    "小悪魔の遊び場": { color: 0x300808, icon: "😈", name: "小悪魔の遊び場" },
    // レア遺跡
    "闇の魔術師団": { color: 0x180e28, icon: "🧙", name: "闇の魔術師団" },
    "宝箱の罠": { color: 0x2a2410, icon: "📦", name: "宝箱の罠" },
    "混成警備隊": { color: 0x202020, icon: "⚔️", name: "混成警備隊" },
    "闇と骨の同盟": { color: 0x161616, icon: "☠️", name: "闇と骨の同盟" },
    "呪われた武具庫": { color: 0x1a0830, icon: "🛡️", name: "呪われた武具庫" },
    "暗殺者の隠れ家": { color: 0x0e0e18, icon: "🗡️", name: "暗殺者の隠れ家" },
    "精霊の聖域": { color: 0x142028, icon: "✨", name: "精霊の聖域" },
    "ガーゴイルの塔": { color: 0x1e2028, icon: "🗼", name: "ガーゴイルの塔" },
    "死者の墓所": { color: 0x141e1e, icon: "⚰️", name: "死者の墓所" },
    "クリスタルの洞窟": { color: 0x1e1430, icon: "💎", name: "クリスタルの洞窟" },
    "雷鳥の巣": { color: 0x2a2810, icon: "⚡", name: "雷鳥の巣" },
    "地底の主": { color: 0x201810, icon: "🐛", name: "地底の主" },
    // ボス遺跡
    "守護者の間": { color: 0x281a0e, icon: "🏛️", name: "守護者の間" },
    "暗黒の祭壇": { color: 0x100818, icon: "🌑", name: "暗黒の祭壇" },
    "最深部の番人": { color: 0x200808, icon: "👁️", name: "最深部の番人" },
    "竜の墓場": { color: 0x300808, icon: "🐉", name: "竜の墓場" },
    "死霊王の玉座": { color: 0x0a0a0a, icon: "👑", name: "死霊王の玉座" },
    "巨神の神殿": { color: 0x282010, icon: "🏛️", name: "巨神の神殿" },
    "混沌の深淵": { color: 0x0a0a14, icon: "🌀", name: "混沌の深淵" },
    "不死の軍団": { color: 0x101010, icon: "💀", name: "不死の軍団" },
    "終焉の間": { color: 0x080808, icon: "💀", name: "終焉の間" },
};

const DEFAULT_RUIN_VISUAL: RuinVisual = { color: 0x1a1a1a, icon: "🏚️", name: "遺跡" };

/** 遺跡の見た目を取得 */
function getRuinVisual(formationName: string): RuinVisual {
    return RUIN_VISUALS[formationName] ?? DEFAULT_RUIN_VISUAL;
}

let _app: Application | null = null;
let _tileContainer: Container | null = null;
let _onTerritoryClick: ((id: string, t: any, x: number, y: number) => void) | null = null;
let _lastTerritoryMap: Map<string, any> = new Map();

let _terrainGraphics: Graphics | null = null;
let _borderGraphics: Graphics | null = null;
let _overlayContainer: Container | null = null;
let _prevTerritoryJSON = "";
let _prevMapPlayerId = "";
let _prevMapHomeId = "";
let _visible = false;
let _localHomeTerritoryId = "";
let _needsFocusOnOpen = true;
let _homeFocusTargetId: string | null = null;

function getMapViewportSize(): { width: number; height: number } {
    const parent = (_app?.canvas as HTMLCanvasElement | undefined)?.parentElement;
    if (parent) {
        const rect = parent.getBoundingClientRect();
        if (rect.width > 0 && rect.height > 0) {
            return { width: rect.width, height: rect.height };
        }
    }
    return { width: _app?.screen.width ?? 0, height: _app?.screen.height ?? 0 };
}

function focusMapOnTerritory(territoryId: string): boolean {
    if (!_app || !_tileContainer) return false;
    const pos = tryParseTerritoryId(territoryId);
    if (!pos) return false;
    const { width, height } = getMapViewportSize();
    if (width <= 0 || height <= 0) return false;
    const { x, y } = coordToScreen(pos.col, pos.row, TILE_DIMENSIONS);
    const scale = _tileContainer.scale.x;
    _tileContainer.x = width / 2 - x * scale;
    _tileContainer.y = height / 2 - y * scale;
    return true;
}

function tryApplyHomeFocus(): boolean {
    if (!_homeFocusTargetId) return false;
    if (!focusMapOnTerritory(_homeFocusTargetId)) return false;
    _needsFocusOnOpen = false;
    return true;
}

function requestHomeFocus(territoryId: string): void {
    _homeFocusTargetId = territoryId;
    _needsFocusOnOpen = true;
    tryApplyHomeFocus();
}

/** 自分の本拠を画面中央に寄せる（状態読み込み後に呼ぶ） */
export function focusMapOnPlayerHome(): void {
    if (!_app || !_tileContainer || !isPlayerIdentityResolved()) return;
    if (gameState.territories.length === 0) return;
    const playerId = getLocalPlayerId();
    if (!gameState.players[playerId]) return;
    const homeId = getPlayerHomeTerritoryId(gameState, playerId);
    requestHomeFocus(homeId);
}

let _drawOrder: { col: number; row: number }[] = [];
let _drawOrderKey = "";

function rebuildDrawOrder(cols: number, rows: number): void {
    const key = `${cols}x${rows}`;
    if (key === _drawOrderKey) return;
    _drawOrderKey = key;
    _drawOrder = [];
    for (let sum = 0; sum < cols + rows - 1; sum++) {
        for (let row = 0; row < rows; row++) {
            const col = sum - row;
            if (col >= 0 && col < cols) _drawOrder.push({ col, row });
        }
    }
}

/** ビューポート内のタイルのみ描画（大マップ向けカリング） */
function getVisibleTileRange(): { minCol: number; maxCol: number; minRow: number; maxRow: number } | null {
    if (!_app || !_tileContainer) return null;
    const cols = getWorldGridCols();
    const rows = getWorldGridRows();
    const { width, height } = getMapViewportSize();
    if (width <= 0 || height <= 0) return null;

    const margin = 3;
    const corners = [
        _tileContainer.toLocal({ x: 0, y: 0 }),
        _tileContainer.toLocal({ x: width, y: 0 }),
        _tileContainer.toLocal({ x: 0, y: height }),
        _tileContainer.toLocal({ x: width, y: height }),
    ];
    let minCol = cols;
    let maxCol = 0;
    let minRow = rows;
    let maxRow = 0;
    for (const p of corners) {
        const { col, row } = screenToCoord(p.x, p.y, TILE_DIMENSIONS);
        minCol = Math.min(minCol, col - margin);
        maxCol = Math.max(maxCol, col + margin);
        minRow = Math.min(minRow, row - margin);
        maxRow = Math.max(maxRow, row + margin);
    }
    return {
        minCol: Math.max(0, minCol),
        maxCol: Math.min(cols - 1, maxCol),
        minRow: Math.max(0, minRow),
        maxRow: Math.min(rows - 1, maxRow),
    };
}

function terrainColor(level: number): number {
    switch (level) {
        case 1: return 0x5a6a38;
        case 2: return 0x8a7a48;
        case 3: return 0x2e5a28;
        case 4: return 0x3a6a7a;
        case 5: return 0x8a8a9a;
        case 6: return 0x6a6a6a;
        case 7: return 0x4a3058;
        case 8: return 0x5a2038;
        case 9: return 0x2a1848;
        default: return 0x4a4838;
    }
}

// level 1-9 → 地形テクスチャ（Lv5山岳はレガシー互換）
const TERRAIN_FILES: Record<number, string> = {
    1: "terrain-plains",
    2: "terrain-hills",
    3: "terrain-forest",
    4: "terrain-river",
    5: "terrain-alpine",
    6: "terrain-mountain",
    7: "terrain-peril",
    8: "terrain-demon",
    9: "terrain-deep",
};

/** 地形テクスチャをプリロード（public/terrain/ の terrain-*.png） */
async function loadTerrainTextures(): Promise<void> {
    for (let level = 1; level <= 9; level++) {
        const base = TERRAIN_FILES[level];
        for (const ext of [".png", ".svg"]) {
            try {
                TERRAIN_TEXTURES[level] = await Assets.load(`/terrain/${base}${ext}`);
                break;
            } catch {
                TERRAIN_TEXTURES[level] = null;
            }
        }
    }
    // 本拠地用の城イラスト
    for (const ext of [".png", ".svg"]) {
        try {
            CASTLE_TEXTURE = await Assets.load(`/terrain/terrain-castle${ext}`);
            break;
        } catch {
            CASTLE_TEXTURE = null;
        }
    }
}

export interface TravelingDestinationOverlay {
    overlayKey: string;
    targetId: string;
    /** 到着予定時刻（Unix ms）。ticker が残り時間を更新する */
    arrivesAt: number;
    secLeft: number;
    unitNames: string[];
    /** 攻撃・探索行軍時: 点線の起点（本拠地） */
    lineFromId?: string;
    ownerId?: string;
    /** マーカー色（旗・探索アイコン） */
    flagColor?: number;
    lineColor?: number;
    /** 攻撃=旗、探索=コンパス */
    marchKind?: "attack" | "explore";
}

let _mapContainer: HTMLDivElement | null = null;

/** マップコンテナがレイアウトされてサイズを持つまで待つ */
export function waitForMapContainerLayout(container: HTMLElement, timeoutMs = 5000): Promise<void> {
    return new Promise((resolve) => {
        const started = performance.now();
        const tick = () => {
            if (container.clientWidth > 0 && container.clientHeight > 0) {
                resolve();
                return;
            }
            if (performance.now() - started > timeoutMs) {
                resolve();
                return;
            }
            requestAnimationFrame(tick);
        };
        tick();
    });
}

/** display:none → block 直後にリサイズ＋再描画を促す（DevTools リサイズ相当） */
export function notifyMapContainerShown(): void {
    _needsFocusOnOpen = true;
    _prevTerritoryJSON = "";
    requestAnimationFrame(() => {
        requestAnimationFrame(() => {
            if (_mapContainer) syncMapRendererSize(_mapContainer);
            if (_visible && _app) tryApplyHomeFocus();
        });
    });
}

export function wakeMapView(
    state: GameState,
    travelingDestinations?: TravelingDestinationOverlay[],
): void {
    if (_mapContainer) syncMapRendererSize(_mapContainer);
    _needsFocusOnOpen = true;
    _prevTerritoryJSON = "";
    _visible = true;
    updateMapView(state, travelingDestinations);
    tryApplyHomeFocus();
    requestRender();
}

export function isMapViewReady(): boolean {
    return _app != null;
}

export async function initMapView(
    container: HTMLDivElement,
    options: {
        onTerritoryClick: (id: string, t: any, x: number, y: number) => void;
    }
) {
    if (_app) return;
    _mapContainer = container;
    await waitForMapContainerLayout(container);

    const initW = Math.max(container.clientWidth, 1);
    const initH = Math.max(container.clientHeight, 1);
    const app = new Application();
    await app.init({ backgroundColor: 0x0a0a0f, width: initW, height: initH, antialias: false });
    // 自動レンダリングを停止（必要な時のみ手動で app.render() を呼ぶ）
    app.ticker.stop();
    // @ts-ignore
    container.appendChild(app.canvas || app.view);

    await loadTerrainTextures();

    _app = app;
    _onTerritoryClick = options.onTerritoryClick;

    _tileContainer = new Container();
    _tileContainer.eventMode = "static";

    _terrainGraphics = new Graphics();
    _tileContainer.addChild(_terrainGraphics);

    _borderGraphics = new Graphics();
    _tileContainer.addChild(_borderGraphics);

    _overlayContainer = new Container();
    _tileContainer.addChild(_overlayContainer);

    let scale = 1;
    const MIN_SCALE = 1;
    const MAX_SCALE = 2.5;

    app.stage.addChild(_tileContainer);

    const resizeObserver = new ResizeObserver(() => {
        syncMapRendererSize(container);
        if (_needsFocusOnOpen) tryApplyHomeFocus();
    });
    resizeObserver.observe(container);

    // マウスホイールでズーム（カーソル位置を中心に）
    const canvas = app.canvas ?? (app as any).view;
    (canvas as HTMLElement).addEventListener("wheel", (e: Event) => {
        if (!_tileContainer || !_app) return;
        const we = e as WheelEvent;
        we.preventDefault();
        const rect = (canvas as HTMLElement).getBoundingClientRect();
        const mx = we.clientX - rect.left;
        const my = we.clientY - rect.top;
        const localX = (mx - _tileContainer.x) / scale;
        const localY = (my - _tileContainer.y) / scale;
        const delta = we.deltaY > 0 ? -0.1 : 0.1;
        const newScale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, scale + delta));
        _tileContainer.x = mx - localX * newScale;
        _tileContainer.y = my - localY * newScale;
        _tileContainer.scale.set(newScale);
        scale = newScale;
        requestRender();
    }, { passive: false });

    // ドラッグ
    let dragging = false;
    let dragStart = { x: 0, y: 0 };
    let containerStart = { x: 0, y: 0 };
    let clickStart = { x: 0, y: 0 };

    app.stage.eventMode = "static";
    app.stage.hitArea = app.screen;

    app.stage.on("pointerdown", (e) => {
        dragging = true;
        dragStart = { x: e.global.x, y: e.global.y };
        clickStart = { x: e.global.x, y: e.global.y };
        if (_tileContainer) containerStart = { x: _tileContainer.x, y: _tileContainer.y };
    });

    app.stage.on("pointermove", (e) => {
        if (dragging && _tileContainer) {
            const dx = e.global.x - dragStart.x;
            const dy = e.global.y - dragStart.y;
            _tileContainer.x = containerStart.x + dx;
            _tileContainer.y = containerStart.y + dy;
            requestRender();
        }
    });

    const onUp = (e: FederatedPointerEvent) => {
        const wasDragging = dragging;
        if (!dragging) return;
        dragging = false;

        const dist = Math.abs(e.global.x - clickStart.x) + Math.abs(e.global.y - clickStart.y);
        if (dist < 5 && _tileContainer) {
            const local = _tileContainer.toLocal(e.global);
            const { col, row } = screenToCoord(local.x, local.y, TILE_DIMENSIONS);

            if (isWithinWorldGrid(col, row)) {
                const id = formatTerritoryId(col, row);
                const t = _lastTerritoryMap.get(id);
                _onTerritoryClick?.(id, t, e.global.x, e.global.y);
            }
        } else if (wasDragging && dist >= 5) {
            redrawTerrain();
            redrawOverlay();
        }
    };

    app.stage.on("pointerup", onUp);
    app.stage.on("pointerupoutside", onUp);

    ensureOverlayTicker();
    syncMapRendererSize(container);
}

function syncMapRendererSize(container: HTMLElement): void {
    if (!_app) return;
    const w = container.clientWidth;
    const h = container.clientHeight;
    if (w > 0 && h > 0) {
        _app.renderer.resize(w, h);
        _app.stage.hitArea = _app.screen;
        requestRender();
    }
}

export function setMapVisible(v: boolean) {
    const wasVisible = _visible;
    if (v && !wasVisible) _needsFocusOnOpen = true;
    _visible = v;
    if (v && !wasVisible && _app) {
        const parent = (_app.canvas as HTMLCanvasElement | undefined)?.parentElement;
        if (parent) syncMapRendererSize(parent);
    }
}

export function updateMapView(
    state: GameState,
    travelingDestinations?: TravelingDestinationOverlay[],
) {
    if (!_app || !_tileContainer || !_terrainGraphics || !_borderGraphics || !_overlayContainer) return;
    if (!_visible) return;
    if (!isPlayerIdentityResolved()) return;

    _lastTerritoryMap = new Map(state.territories.map(t => [t.id, t]));
    const playerId = getLocalPlayerId();
    _localHomeTerritoryId = getPlayerHomeTerritoryId(state, playerId);

    if (playerId !== _prevMapPlayerId) _needsFocusOnOpen = true;
    if (
        _needsFocusOnOpen
        && _localHomeTerritoryId
        && state.territories.length > 0
        && state.players[playerId]
    ) {
        _homeFocusTargetId = _localHomeTerritoryId;
    }

    const territoryJSON = JSON.stringify(state.territories);
    const shouldRedrawTerrain =
        territoryJSON !== _prevTerritoryJSON
        || playerId !== _prevMapPlayerId
        || _localHomeTerritoryId !== _prevMapHomeId;
    if (shouldRedrawTerrain) {
        _prevTerritoryJSON = territoryJSON;
        _prevMapPlayerId = playerId;
        _prevMapHomeId = _localHomeTerritoryId;
        redrawTerrain();
    }

    if (_needsFocusOnOpen) tryApplyHomeFocus();

    redrawOverlay(travelingDestinations);
}

const BORDER_STROKE_INNER = { alignment: 1 } as const;

function strokeTerritoryBorder(
    g: Graphics,
    points: number[],
    id: string,
    t: { owner_id?: string | null; is_base?: boolean; ruin?: { difficulty: string } } | undefined,
    playerId: string,
): void {
    const isHome = isPlayerHomeTile(id, t, playerId, _localHomeTerritoryId);

    if (t?.ruin) {
        const diffColor = t.ruin.difficulty === "extreme" ? 0x8a1a1a
            : t.ruin.difficulty === "hard" ? 0x8a4010
            : t.ruin.difficulty === "normal" ? 0x6a5a20
            : 0x2a5a2a;
        g.poly(points, true).stroke({ width: 0.75, color: diffColor, alpha: 0.8, ...BORDER_STROKE_INNER });
        return;
    }

    if (isHome) {
        g.poly(points, true).stroke({ width: 1.5, color: 0x1a0f00, alpha: 0.94, ...BORDER_STROKE_INNER });
        g.poly(points, true).stroke({ width: 0.9375, color: 0xffe8a8, alpha: 1, ...BORDER_STROKE_INNER });
    } else if (t?.owner_id === playerId) {
        g.poly(points, true).stroke({ width: 1.5, color: 0x020810, alpha: 0.94, ...BORDER_STROKE_INNER });
        g.poly(points, true).stroke({ width: 0.9375, color: 0x9ae8ff, alpha: 1, ...BORDER_STROKE_INNER });
        if (t.is_base) {
            g.poly(points, true).stroke({ width: 0.5625, color: 0xffcc66, alpha: 0.95, ...BORDER_STROKE_INNER });
        }
    } else if (t?.owner_id && t.owner_id !== "barbarian") {
        g.poly(points, true).stroke({ width: 1.3125, color: 0x140306, alpha: 0.94, ...BORDER_STROKE_INNER });
        g.poly(points, true).stroke({ width: 0.75, color: 0xd84050, alpha: 0.98, ...BORDER_STROKE_INNER });
    } else {
        g.poly(points, true).stroke({ width: 1.125, color: 0x101014, alpha: 0.45, ...BORDER_STROKE_INNER });
        g.poly(points, true).stroke({ width: 0.65625, color: 0xb0b4bc, alpha: 0.55, ...BORDER_STROKE_INNER });
    }
}

function territoryLevelAt(col: number, row: number): number | undefined {
    if (!isWithinWorldGrid(col, row)) return undefined;
    return _lastTerritoryMap.get(formatTerritoryId(col, row))?.level;
}

/** 標高データは持たないが、◇タイル上で光沢・陰影を足して立体感を出す */
function applyTerrainReliefShading(g: Graphics, points: number[], cx: number, cy: number) {
    const px = (i: number) => points[i * 2];
    const py = (i: number) => points[i * 2 + 1];
    g.poly([px(0), py(0), px(1), py(1), cx, cy, px(3), py(3)], true)
        .fill({ color: 0xffffff, alpha: 0.08 });
    g.poly([px(3), py(3), cx, cy, px(1), py(1), px(2), py(2)], true)
        .fill({ color: 0x000000, alpha: 0.16 });
}

/** 川タイルはグリッド軸に応じてテクスチャを90度回転（row軸=既定 / col軸=回転） */
function fillTerrainDiamond(
    g: Graphics,
    points: number[],
    cx: number,
    cy: number,
    texture: Texture,
    riverAxis: RiverAxis | null,
) {
    if (riverAxis === "col") {
        const matrix = new Matrix();
        matrix.translate(cx, cy);
        matrix.rotate(Math.PI / 2);
        matrix.translate(-cx, -cy);
        g.poly(points, true).fill({ texture, matrix, textureSpace: "local" });
    } else {
        g.poly(points, true).fill({ texture, textureSpace: "local" });
    }
    applyTerrainReliefShading(g, points, cx, cy);
}

function redrawTerrain() {
    const g = _terrainGraphics!;
    const borders = _borderGraphics!;
    const container = _overlayContainer!;
    g.clear();
    borders.clear();

    while (container.children.length > 0) container.removeChildAt(0);

    const cols = getWorldGridCols();
    const rows = getWorldGridRows();
    rebuildDrawOrder(cols, rows);
    const visible = getVisibleTileRange();

    const playerId = getLocalPlayerId();
    const tileDrawData: {
        points: number[];
        id: string;
        t: { owner_id?: string | null; is_base?: boolean; ruin?: { difficulty: string; formation_name: string } } | undefined;
        level: number;
    }[] = [];

    for (const { col, row } of _drawOrder) {
        if (visible) {
            if (col < visible.minCol || col > visible.maxCol || row < visible.minRow || row > visible.maxRow) {
                continue;
            }
        }
        const id = formatTerritoryId(col, row);
        const t = _lastTerritoryMap.get(id);
        const level = t?.level ?? 1;
        const { x: cx, y: cy } = coordToScreen(col, row, TILE_DIMENSIONS);
        const points = diamondPoints(cx, cy, TILE_DIMENSIONS);
        tileDrawData.push({ points, id, t, level });

        if (t?.ruin) {
            const ruinVisual = getRuinVisual(t.ruin.formation_name);
            g.poly(points, true).fill(ruinVisual.color);

            const ruinText = new Text({
                text: ruinVisual.icon,
                style: { fontSize: 18 },
            });
            ruinText.anchor.set(0.5);
            ruinText.x = cx;
            ruinText.y = cy;
            container.addChild(ruinText);
            continue;
        }

        const isOwnHome = isPlayerHomeTile(id, t, playerId, _localHomeTerritoryId);
        const isEnemyHome = isEnemyHomeTile(id, t, playerId, gameState);
        const isAiHome = isAiHomeTile(id, t, gameState);
        const useCastleIcon = isOwnHome || isEnemyHome || isAiHome;
        let color = 0x4a4838;
        if (t) {
            if (t.owner_id === playerId) color = 0x2a3a5a;
            else if (t.owner_id && isAiOwnerId(t.owner_id)) {
                const aiIndex = gameState.ai_factions?.findIndex((f) => `ai_${f.faction_id}` === t.owner_id) ?? 0;
                const faction = gameState.ai_factions?.[aiIndex];
                color = faction?.color ?? 0x4a3a2a;
            }
            else if (t.owner_id && t.owner_id !== "barbarian") color = 0x5a2a30;
            else color = terrainColor(level);
        }

        const tex = useCastleIcon ? CASTLE_TEXTURE : TERRAIN_TEXTURES[level];
        if (tex) {
            const riverAxis =
                isRiverLevel(level) && !useCastleIcon
                    ? pickRiverAxis(col, row, territoryLevelAt)
                    : null;
            fillTerrainDiamond(g, points, cx, cy, tex, riverAxis);
        } else {
            g.poly(points, true).fill(color);
            applyTerrainReliefShading(g, points, cx, cy);
        }
    }

    // 枠線は専用レイヤーに奥→手前の逆順で描く（下半分が隣タイルに隠れない）
    for (let i = tileDrawData.length - 1; i >= 0; i--) {
        const { points, id, t } = tileDrawData[i];
        strokeTerritoryBorder(borders, points, id, t, playerId);
    }

    requestRender();
}

const ATTACK_FLAG_POLE_H = 14;

function drawAttackFlag(g: Graphics, flagX: number, cy: number, fill: number) {
    const stroke = ((fill >> 1) & 0x7f7f7f);
    g.moveTo(flagX, cy).lineTo(flagX, cy - ATTACK_FLAG_POLE_H).stroke({ width: 2.5, color: 0x5d4037 });
    g.moveTo(flagX, cy - ATTACK_FLAG_POLE_H)
        .lineTo(flagX + 9, cy - ATTACK_FLAG_POLE_H - 4)
        .lineTo(flagX, cy - ATTACK_FLAG_POLE_H + 3)
        .closePath()
        .fill(fill)
        .stroke({ width: 1, color: stroke });
}

/** 探索遠征用マーカー（コンパス） */
function drawExploreMarker(g: Graphics, cx: number, cy: number, fill: number) {
    const iy = cy - 10;
    const r = 8;
    const rim = ((fill >> 1) & 0x7f7f7f) | 0x303030;
    g.circle(cx, iy, r)
        .fill({ color: 0x142028, alpha: 0.9 })
        .stroke({ width: 2, color: fill });
    g.moveTo(cx, iy).lineTo(cx + 4, iy - 5).stroke({ width: 2, color: fill, cap: "round" });
    g.moveTo(cx, iy).lineTo(cx - 3, iy + 4).stroke({ width: 2, color: 0x9ca3af, cap: "round" });
    g.circle(cx, iy, 2).fill(fill).stroke({ width: 0.5, color: rim });
}

type TravelPoolEntry = {
    bg: Graphics;
    text: Text;
    flagX: number;
    cy: number;
    arrivesAt: number;
    active: boolean;
};

const _travelPool: TravelPoolEntry[] = [];
let _travelContainer: Container | null = null;
let _attackLineGraphics: Graphics | null = null;
let _lastTravelingDestinations: TravelingDestinationOverlay[] | undefined;
let _attackLines: { fromId: string; toId: string; color: number; arrivesAt: number }[] = [];
let _overlayTickerAdded = false;
let _activeTravelPoolCount = 0;
let _lastFlagTimerSec = -1;

/** 手動レンダリング — 必要なタイミングでのみ app.render() を呼ぶ */
function requestRender(): void {
    if (_app && _visible) _app.render();
}

function attackLineBlinkAlpha(): number {
    // 1秒ごとに on/off を切り替える（GPU負荷を大幅削減）
    return (Math.floor(Date.now() / 1000) % 2 === 0) ? 0.42 : 0.2;
}

function strokeDottedLine(
    g: Graphics,
    x0: number,
    y0: number,
    x1: number,
    y1: number,
    opts: { dash: number; gap: number; width: number; color: number; alpha: number },
) {
    const dx = x1 - x0;
    const dy = y1 - y0;
    const len = Math.hypot(dx, dy);
    if (len < 1) return;
    const ux = dx / len;
    const uy = dy / len;
    const step = opts.dash + opts.gap;
    for (let pos = 0; pos < len; pos += step) {
        const end = Math.min(pos + opts.dash, len);
        g.moveTo(x0 + ux * pos, y0 + uy * pos)
            .lineTo(x0 + ux * end, y0 + uy * end)
            .stroke({ width: opts.width, color: opts.color, alpha: opts.alpha, cap: "round" });
    }
}

function drawAttackLines(alpha: number) {
    if (!_travelContainer) return;
    if (!_attackLineGraphics) {
        _attackLineGraphics = new Graphics();
        _travelContainer.addChildAt(_attackLineGraphics, 0);
    }
    const g = _attackLineGraphics;
    g.clear();
    if (_attackLines.length === 0) {
        g.visible = false;
        return;
    }
    g.visible = true;
    for (const { fromId, toId, color } of _attackLines) {
        const from = tryParseTerritoryId(fromId);
        const to = tryParseTerritoryId(toId);
        if (!from || !to) continue;
        const fromScreen = coordToScreen(from.col, from.row, TILE_DIMENSIONS);
        const toScreen = coordToScreen(to.col, to.row, TILE_DIMENSIONS);
        strokeDottedLine(
            g,
            fromScreen.x,
            fromScreen.y,
            toScreen.x,
            toScreen.y,
            { dash: 4, gap: 5, width: 1.5, color, alpha },
        );
    }
}

function syncAttackLines(destinations: TravelingDestinationOverlay[] | undefined) {
    _attackLines = [];
    if (destinations) {
        for (const d of destinations) {
            if (!d.lineFromId) continue;
            _attackLines.push({
                fromId: d.lineFromId,
                toId: d.targetId,
                color: d.lineColor ?? 0xff8888,
                arrivesAt: d.arrivesAt,
            });
        }
    }
    drawAttackLines(attackLineBlinkAlpha());
}

function updateMarchOverlayAnimation() {
    if (!_visible) return;

    const now = Date.now();
    _attackLines = _attackLines.filter((line) => line.arrivesAt > now);

    if (_attackLines.length > 0) {
        drawAttackLines(attackLineBlinkAlpha());
    } else if (_attackLineGraphics) {
        _attackLineGraphics.clear();
        _attackLineGraphics.visible = false;
    }

    if (_activeTravelPoolCount === 0) return;
    const sec = Math.floor(now / 1000);
    if (sec === _lastFlagTimerSec) return;
    _lastFlagTimerSec = sec;

    for (let i = 0; i < _activeTravelPoolCount; i++) {
        const entry = _travelPool[i];
        if (!entry.active) continue;
        const secLeft = (entry.arrivesAt - now) / 1000;
        if (secLeft <= 0) {
            entry.active = false;
            entry.bg.visible = false;
            entry.text.visible = false;
            continue;
        }
        entry.text.text = formatTimeHHMMSS(secLeft);
    }
}

function ensureOverlayTicker() {
    if (!_app || _overlayTickerAdded) return;
    _overlayTickerAdded = true;
    // setIntervalで1秒ごとに更新（GPU負荷削減。tickerは初期化時に停止済み）
    setInterval(() => {
        if (!_visible) return;
        updateMarchOverlayAnimation();
        requestRender();
    }, 1000);
}

function redrawOverlay(travelingDestinations?: TravelingDestinationOverlay[]) {
    if (!_tileContainer) return;

    if (travelingDestinations !== undefined) {
        _lastTravelingDestinations = travelingDestinations;
    }
    const destinations = travelingDestinations ?? _lastTravelingDestinations;

    if (!_travelContainer) {
        _travelContainer = new Container();
        _tileContainer.addChild(_travelContainer);
        ensureOverlayTicker();
    }

    syncAttackLines(destinations);

    const overlays = destinations ?? [];
    const targetGroups = new Map<string, TravelingDestinationOverlay[]>();
    for (const d of overlays) {
        const group = targetGroups.get(d.targetId) ?? [];
        group.push(d);
        targetGroups.set(d.targetId, group);
    }

    let poolIdx = 0;

    for (const travel of overlays) {
        if (travel.secLeft <= 0) continue;
        const parsed = tryParseTerritoryId(travel.targetId);
        if (!parsed) continue;
        const { col, row } = parsed;

        const group = targetGroups.get(travel.targetId) ?? [travel];
        const groupIndex = group.indexOf(travel);
        const offsetX = (groupIndex - (group.length - 1) / 2) * 12;

        const { x: cx, y: cy } = coordToScreen(col, row, TILE_DIMENSIONS);
        const flagX = cx + offsetX;

        let entry = _travelPool[poolIdx];
        if (!entry) {
            const bg = new Graphics();
            const text = new Text({ text: "", style: { fontSize: 11, fill: 0xffffff, fontWeight: "bold" } });
            text.anchor.set(0.5);
            entry = { bg, text, flagX: 0, cy: 0, arrivesAt: 0, active: false };
            _travelPool.push(entry);
            _travelContainer.addChild(bg);
            _travelContainer.addChild(text);
        }

        entry.flagX = flagX;
        entry.cy = cy;
        entry.arrivesAt = travel.arrivesAt;
        entry.active = true;

        entry.text.text = formatTimeHHMMSS(Math.max(0, travel.secLeft));
        entry.text.x = flagX;
        entry.text.y = cy + 6;

        const markerFill = travel.flagColor ?? 0xff4444;

        entry.bg.clear();
        if (travel.marchKind === "explore") {
            drawExploreMarker(entry.bg, flagX, cy, markerFill);
        } else {
            drawAttackFlag(entry.bg, flagX, cy, markerFill);
        }

        const pad = 2;
        entry.bg.roundRect(
            flagX - entry.text.width / 2 - pad,
            cy + 6 - entry.text.height / 2 - pad,
            entry.text.width + pad * 2,
            entry.text.height + pad * 2, 2
        ).fill({ color: 0x000000, alpha: 0.65 });

        entry.bg.visible = true;
        entry.text.visible = true;
        poolIdx++;
    }

    _activeTravelPoolCount = poolIdx;
    _lastFlagTimerSec = -1;

    for (let i = poolIdx; i < _travelPool.length; i++) {
        _travelPool[i].active = false;
        _travelPool[i].bg.visible = false;
        _travelPool[i].text.visible = false;
    }

    requestRender();
}
