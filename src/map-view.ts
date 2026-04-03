import {
    Application,
    Container,
    Graphics,
    Text,
    Assets,
    Texture,
    FederatedPointerEvent,
} from "pixi.js";
import type { GameState } from "./store";
import {
    GRID_COLS,
    GRID_ROWS,
    HOME_TERRITORY_ID,
    formatTerritoryId,
    isWithinWorldGrid,
    tryParseTerritoryId,
} from "./game/territories";
import {
    coordToScreen,
    diamondPoints,
    screenToCoord,
} from "./map-isometric";
import { formatTimeHHMMSS } from "./utils";
export { GRID_COLS, GRID_ROWS, HOME_TERRITORY_ID, parseTerritoryId } from "./game/territories";

// アイソメトリック: マスは◇型。45度回転して斜め上から見下ろす視点
// タイルの幅・高さ（◇の横・縦の長さ）
const TILE_WIDTH = 56;
const TILE_HEIGHT = 28;
const TILE_DIMENSIONS = { width: TILE_WIDTH, height: TILE_HEIGHT } as const;

// 地形テクスチャキャッシュ (level 1-6 に対応)
const TERRAIN_TEXTURES: (Texture | null)[] = [null, null, null, null, null, null];
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
let _overlayContainer: Container | null = null;
let _prevTerritoryJSON = "";
let _visible = false;

const DRAW_ORDER: { col: number; row: number }[] = [];
for (let sum = 0; sum < GRID_COLS + GRID_ROWS - 1; sum++) {
    for (let row = 0; row < GRID_ROWS; row++) {
        const col = sum - row;
        if (col >= 0 && col < GRID_COLS) DRAW_ORDER.push({ col, row });
    }
}

function terrainColor(level: number): number {
    switch (level) {
        case 1: return 0x5a6a38;
        case 2: return 0x8a7a48;
        case 3: return 0x2e5a28;
        case 4: return 0x6a6a6a;
        case 5: return 0x8a8a9a;
        case 6: return 0x3a6a7a;
        default: return 0x4a4838;
    }
}

// level 1-6 → 地形ファイル名（平原・丘陵・森・山地・山岳・川）
const TERRAIN_FILES: Record<number, string> = {
    1: "terrain-plains",
    2: "terrain-hills",
    3: "terrain-forest",
    4: "terrain-mountain",
    5: "terrain-alpine",
    6: "terrain-river",
};

/** 地形テクスチャをプリロード（public/terrain/ の terrain-*.png） */
async function loadTerrainTextures(): Promise<void> {
    for (let level = 1; level <= 6; level++) {
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

export async function initMapView(
    container: HTMLDivElement,
    options: {
        onTerritoryClick: (id: string, t: any, x: number, y: number) => void;
    }
) {
    const app = new Application();
    await app.init({ resizeTo: window, backgroundColor: 0x0a0a0f });
    // @ts-ignore
    container.appendChild(app.canvas || app.view);

    await loadTerrainTextures();

    _app = app;
    _onTerritoryClick = options.onTerritoryClick;

    _tileContainer = new Container();
    _tileContainer.eventMode = "static";

    _terrainGraphics = new Graphics();
    _tileContainer.addChild(_terrainGraphics);

    _overlayContainer = new Container();
    _tileContainer.addChild(_overlayContainer);

    const centerScreen = coordToScreen(24, 24, TILE_DIMENSIONS);
    _tileContainer.x = app.screen.width / 2 - centerScreen.x;
    _tileContainer.y = app.screen.height / 2 - centerScreen.y;

    let scale = 1;
    const MIN_SCALE = 1;
    const MAX_SCALE = 2.5;

    app.stage.addChild(_tileContainer);

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
        }
    });

    const onUp = (e: FederatedPointerEvent) => {
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
        }
    };

    app.stage.on("pointerup", onUp);
    app.stage.on("pointerupoutside", onUp);
}

export function setMapVisible(v: boolean) { _visible = v; }

export function updateMapView(
    state: GameState,
    travelingDestinations?: { targetId: string; secLeft: number; unitNames: string[] }[]
) {
    if (!_app || !_tileContainer || !_terrainGraphics || !_overlayContainer) return;
    if (!_visible) return;

    _lastTerritoryMap = new Map(state.territories.map(t => [t.id, t]));

    const territoryJSON = JSON.stringify(state.territories);
    if (territoryJSON !== _prevTerritoryJSON) {
        _prevTerritoryJSON = territoryJSON;
        redrawTerrain();
    }

    redrawOverlay(travelingDestinations);
}

function redrawTerrain() {
    const g = _terrainGraphics!;
    const container = _overlayContainer!;
    g.clear();

    while (container.children.length > 0) container.removeChildAt(0);

    for (const { col, row } of DRAW_ORDER) {
        const id = formatTerritoryId(col, row);
        const t = _lastTerritoryMap.get(id);
        const level = t?.level ?? 1;
        const isRuin = !!t?.ruin;

        const { x: cx, y: cy } = coordToScreen(col, row, TILE_DIMENSIONS);
        const points = diamondPoints(cx, cy, TILE_DIMENSIONS);

        if (isRuin) {
            const ruinVisual = getRuinVisual(t.ruin.formation_name);
            g.poly(points, true).fill(ruinVisual.color);

            const diffColor = t.ruin.difficulty === "extreme" ? 0x8a1a1a
                : t.ruin.difficulty === "hard" ? 0x8a4010
                : t.ruin.difficulty === "normal" ? 0x6a5a20
                : 0x2a5a2a;
            g.poly(points, true).stroke({ width: 2, color: diffColor, alpha: 0.8 });

            const ruinText = new Text({
                text: ruinVisual.icon,
                style: { fontSize: 18 },
            });
            ruinText.anchor.set(0.5);
            ruinText.x = cx;
            ruinText.y = cy;
            container.addChild(ruinText);
        } else {
            let color = 0x4a4838;
            if (t) {
                if (t.owner_id === "player") color = 0x2a3a5a;
                else if (t.owner_id && t.owner_id !== "barbarian") color = 0x5a2a30;
                else color = terrainColor(level);
            }

            const tex = id === HOME_TERRITORY_ID ? CASTLE_TEXTURE : TERRAIN_TEXTURES[level];

            if (tex) {
                g.poly(points, true).fill({ texture: tex, textureSpace: "local" });
            } else {
                g.poly(points, true).fill(color);
            }

            // テクスチャを隠さないよう装飾は枠線のみ（二重線で視認性を確保）
            if (id === HOME_TERRITORY_ID) {
                g.poly(points, true).stroke({ width: 4, color: 0x1a0f00, alpha: 0.94 });
                g.poly(points, true).stroke({ width: 2.5, color: 0xffe8a8, alpha: 1 });
            } else if (t?.owner_id === "player") {
                g.poly(points, true).stroke({ width: 4, color: 0x020810, alpha: 0.94 });
                g.poly(points, true).stroke({ width: 2.5, color: 0x9ae8ff, alpha: 1 });
                if (t.is_base) {
                    g.poly(points, true).stroke({ width: 1.5, color: 0xffcc66, alpha: 0.95 });
                }
            } else if (t?.owner_id && t.owner_id !== "barbarian") {
                g.poly(points, true).stroke({ width: 3.5, color: 0x200508, alpha: 0.92 });
                g.poly(points, true).stroke({ width: 2, color: 0xff9aaa, alpha: 0.98 });
            } else {
                g.poly(points, true).stroke({ width: 1, color: 0x2a2418, alpha: 0.55 });
            }
        }
    }
}

const _travelPool: { bg: Graphics; text: Text }[] = [];
let _travelContainer: Container | null = null;

function redrawOverlay(
    travelingDestinations?: { targetId: string; secLeft: number; unitNames: string[] }[]
) {
    if (!_tileContainer) return;

    if (!_travelContainer) {
        _travelContainer = new Container();
        _tileContainer.addChild(_travelContainer);
    }

    const travelMap = new Map<string, { secLeft: number; unitNames: string[] }>();
    if (travelingDestinations) {
        for (const d of travelingDestinations) travelMap.set(d.targetId, d);
    }

    let poolIdx = 0;

    for (const [_id, travel] of travelMap) {
        const parsed = tryParseTerritoryId(_id);
        if (!parsed) continue;
        const { col, row } = parsed;

        const { x: cx, y: cy } = coordToScreen(col, row, TILE_DIMENSIONS);

        let entry = _travelPool[poolIdx];
        if (!entry) {
            const bg = new Graphics();
            const text = new Text({ text: "", style: { fontSize: 11, fill: 0xffffff, fontWeight: "bold" } });
            text.anchor.set(0.5);
            entry = { bg, text };
            _travelPool.push(entry);
            _travelContainer.addChild(bg);
            _travelContainer.addChild(text);
        }

        const timeStr = formatTimeHHMMSS(travel.secLeft);
        entry.text.text = timeStr;
        entry.text.x = cx;
        entry.text.y = cy + 6;

        entry.bg.clear();
        const poleH = 14;
        entry.bg.moveTo(cx, cy).lineTo(cx, cy - poleH).stroke({ width: 2.5, color: 0x5d4037 });
        entry.bg.moveTo(cx, cy - poleH).lineTo(cx + 9, cy - poleH - 4).lineTo(cx, cy - poleH + 3).closePath().fill(0xff4444).stroke({ width: 1, color: 0xcc2222 });

        const pad = 2;
        entry.bg.roundRect(
            cx - entry.text.width / 2 - pad,
            cy + 6 - entry.text.height / 2 - pad,
            entry.text.width + pad * 2,
            entry.text.height + pad * 2, 2
        ).fill({ color: 0x000000, alpha: 0.65 });

        entry.bg.visible = true;
        entry.text.visible = true;
        poolIdx++;
    }

    for (let i = poolIdx; i < _travelPool.length; i++) {
        _travelPool[i].bg.visible = false;
        _travelPool[i].text.visible = false;
    }
}
