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
import { formatTimeHHMMSS } from "./utils";

export const GRID_COLS = 48;
export const GRID_ROWS = 48;
export const HOME_TERRITORY_ID = "c_24_24";

// アイソメトリック: マスは◇型。45度回転して斜め上から見下ろす視点
// タイルの幅・高さ（◇の横・縦の長さ）
const TILE_WIDTH = 56;
const TILE_HEIGHT = 28;

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
    "石の番人": { color: 0x8b7355, icon: "🗿", name: "石の番人" },
    "亡霊の群れ": { color: 0x6b5b95, icon: "👻", name: "亡霊の群れ" },
    "骸骨兵団": { color: 0x4a4a4a, icon: "💀", name: "骸骨兵団" },
    "スライムの巣": { color: 0x7eb77f, icon: "🟢", name: "スライムの巣" },
    "蜘蛛の巣窟": { color: 0x5d4e37, icon: "🕸️", name: "蜘蛛の巣窟" },
    "炎の回廊": { color: 0xb8420f, icon: "🔥", name: "炎の回廊" },
    "氷結の間": { color: 0xa8d8ea, icon: "❄️", name: "氷結の間" },
    "小悪魔の遊び場": { color: 0x8b0000, icon: "😈", name: "小悪魔の遊び場" },
    // レア遺跡
    "闇の魔術師団": { color: 0x2d1b4e, icon: "🧙", name: "闇の魔術師団" },
    "宝箱の罠": { color: 0xc9b037, icon: "📦", name: "宝箱の罠" },
    "混成警備隊": { color: 0x696969, icon: "⚔️", name: "混成警備隊" },
    "闇と骨の同盟": { color: 0x3d3d3d, icon: "☠️", name: "闇と骨の同盟" },
    "呪われた武具庫": { color: 0x4b0082, icon: "🛡️", name: "呪われた武具庫" },
    "暗殺者の隠れ家": { color: 0x1a1a2e, icon: "🗡️", name: "暗殺者の隠れ家" },
    "精霊の聖域": { color: 0x87ceeb, icon: "✨", name: "精霊の聖域" },
    "ガーゴイルの塔": { color: 0x708090, icon: "🗼", name: "ガーゴイルの塔" },
    "死者の墓所": { color: 0x2f4f4f, icon: "⚰️", name: "死者の墓所" },
    "クリスタルの洞窟": { color: 0x9370db, icon: "💎", name: "クリスタルの洞窟" },
    "雷鳥の巣": { color: 0xffd700, icon: "⚡", name: "雷鳥の巣" },
    "地底の主": { color: 0x654321, icon: "🐛", name: "地底の主" },
    // ボス遺跡
    "守護者の間": { color: 0x8b4513, icon: "🏛️", name: "守護者の間" },
    "暗黒の祭壇": { color: 0x1a0a2e, icon: "🌑", name: "暗黒の祭壇" },
    "最深部の番人": { color: 0x4a0e0e, icon: "👁️", name: "最深部の番人" },
    "竜の墓場": { color: 0x8b0000, icon: "🐉", name: "竜の墓場" },
    "死霊王の玉座": { color: 0x0d0d0d, icon: "👑", name: "死霊王の玉座" },
    "巨神の神殿": { color: 0xc9b037, icon: "🏛️", name: "巨神の神殿" },
    "混沌の深淵": { color: 0x0f0f23, icon: "🌀", name: "混沌の深淵" },
    "不死の軍団": { color: 0x1c1c1c, icon: "💀", name: "不死の軍団" },
    "終焉の間": { color: 0x0a0a0a, icon: "💀", name: "終焉の間" },
};

const DEFAULT_RUIN_VISUAL: RuinVisual = { color: 0x6b6b6b, icon: "🏚️", name: "遺跡" };

/** 遺跡の見た目を取得 */
function getRuinVisual(formationName: string): RuinVisual {
    return RUIN_VISUALS[formationName] ?? DEFAULT_RUIN_VISUAL;
}

let _app: Application | null = null;
let _tileContainer: Container | null = null;
let _onTerritoryClick: ((id: string, t: any, x: number, y: number) => void) | null = null;
let _lastTerritoryMap: Map<string, any> = new Map();

/** グリッド座標 → アイソメトリック画面座標（タイル中心） */
function coordToScreen(col: number, row: number): { x: number; y: number } {
    return {
        x: (col - row) * (TILE_WIDTH / 2),
        y: (col + row) * (TILE_HEIGHT / 2),
    };
}

/** 画面座標 → グリッド座標（逆変換） */
function screenToCoord(screenX: number, screenY: number): { col: number; row: number } {
    const halfW = TILE_WIDTH / 2;
    const halfH = TILE_HEIGHT / 2;
    const col = (screenX / halfW + screenY / halfH) / 2;
    const row = (screenY / halfH - screenX / halfW) / 2;
    return { col: Math.round(col), row: Math.round(row) };
}

/** ◇の4頂点を返す（中心 cx, cy） */
function diamondPoints(cx: number, cy: number): number[] {
    const hw = TILE_WIDTH / 2;
    const hh = TILE_HEIGHT / 2;
    return [
        cx, cy - hh,      // 上
        cx + hw, cy,      // 右
        cx, cy + hh,      // 下
        cx - hw, cy,      // 左
    ];
}

/** 地形レベル（1-6）に対応する色（テクスチャがない時のフォールバック） */
function terrainColor(level: number): number {
    switch (level) {
        case 1: return 0xeeeeee; // 平原
        case 2: return 0xddddaa; // 丘陵
        case 3: return 0xaaddaa; // 森
        case 4: return 0xaaaaaa; // 山地
        case 5: return 0x888888; // 山岳
        case 6: return 0x4488cc; // 川
        default: return 0xcccccc;
    }
}

// Parse c_col_row
export function parseTerritoryId(id: string): { col: number; row: number } {
    if (id.startsWith("c_")) {
        const parts = id.substring(2).split("_");
        if (parts.length === 2) {
            return { col: parseInt(parts[0], 10), row: parseInt(parts[1], 10) };
        }
    }
    return { col: 0, row: 0 };
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
    await app.init({ resizeTo: window, backgroundColor: 0x111111 });
    // @ts-ignore
    container.appendChild(app.canvas || app.view);

    await loadTerrainTextures();

    _app = app;
    _onTerritoryClick = options.onTerritoryClick;

    _tileContainer = new Container();
    _tileContainer.eventMode = "static";

    // アイソメトリックで中央(24,24)が画面中央に来るように
    const centerScreen = coordToScreen(24, 24);
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
            const { col, row } = screenToCoord(local.x, local.y);

            if (col >= 0 && col < GRID_COLS && row >= 0 && row < GRID_ROWS) {
                const id = `c_${col}_${row}`;
                const t = _lastTerritoryMap.get(id);
                _onTerritoryClick?.(id, t, e.global.x, e.global.y);
            }
        }
    };

    app.stage.on("pointerup", onUp);
    app.stage.on("pointerupoutside", onUp);
}

export function updateMapView(
    state: GameState,
    travelingDestinations?: { targetId: string; secLeft: number; unitNames: string[] }[]
) {
    if (!_app || !_tileContainer) return;

    _tileContainer.removeChildren();
    _lastTerritoryMap = new Map(state.territories.map(t => [t.id, t]));

    const g = new Graphics();
    _tileContainer.addChild(g);

    // アイソメトリック: 奥( row+col 小) → 手前( row+col 大) の順で描画
    const order: { col: number; row: number }[] = [];
    for (let sum = 0; sum < GRID_COLS + GRID_ROWS - 1; sum++) {
        for (let row = 0; row < GRID_ROWS; row++) {
            const col = sum - row;
            if (col >= 0 && col < GRID_COLS) order.push({ col, row });
        }
    }

    for (const { col, row } of order) {
        const id = `c_${col}_${row}`;
        const t = _lastTerritoryMap.get(id);
        const level = t?.level ?? 1;
        const isRuin = !!t?.ruin;

        const { x: cx, y: cy } = coordToScreen(col, row);
        const points = diamondPoints(cx, cy);

        // 遺跡マスは専用の色を使用
        if (isRuin) {
            const ruinVisual = getRuinVisual(t.ruin.formation_name);
            g.poly(points, true).fill(ruinVisual.color);

            // 難易度に応じた枠線色
            const diffColor = t.ruin.difficulty === "extreme" ? 0xff0000 
                : t.ruin.difficulty === "hard" ? 0xff8800 
                : t.ruin.difficulty === "normal" ? 0xffcc00 
                : 0x88ff88;
            g.poly(points, true).stroke({ width: 2, color: diffColor });

            // 遺跡アイコン
            const ruinText = new Text({
                text: ruinVisual.icon,
                style: { fontSize: 18 },
            });
            ruinText.anchor.set(0.5);
            ruinText.x = cx;
            ruinText.y = cy;
            _tileContainer.addChild(ruinText);
        } else {
            // 通常マス
            let color = 0xcccccc;
            if (t) {
                if (t.owner_id === "player") color = 0xccccff;
                else if (t.owner_id && t.owner_id !== "barbarian") color = 0xffcccc;
                else color = terrainColor(level);
            }

            // 本拠地は城イラスト、それ以外は地形イラスト
            const tex = id === HOME_TERRITORY_ID ? CASTLE_TEXTURE : TERRAIN_TEXTURES[level];

            if (tex) {
                g.poly(points, true).fill({ texture: tex, textureSpace: "local" });
            } else {
                g.poly(points, true).fill(color);
            }

            // 枠線（本拠地＝黄、自領＝青、他勢力＝赤、中立＝グレー）
            if (id === HOME_TERRITORY_ID) {
                g.poly(points, true).stroke({ width: 2, color: 0xffff00 });
            } else if (t?.owner_id === "player") {
                g.poly(points, true).stroke({ width: 2, color: 0x4488ff });
            } else if (t?.owner_id && t.owner_id !== "barbarian") {
                g.poly(points, true).stroke({ width: 2, color: 0xff4488 });
            } else {
                g.poly(points, true).stroke({ width: 1, color: 0x888888, alpha: 0.3 });
            }
        }

        // 行軍先: 旗＋到達までの時間だけで十分（オーバーレイなし）
        const travel = travelingDestinations?.find(d => d.targetId === id);
        if (travel) {
            // 旗（やや大きめで目立たせる）
            const poleH = 14;
            g.moveTo(cx, cy).lineTo(cx, cy - poleH).stroke({ width: 2.5, color: 0x5d4037 });
            g.moveTo(cx, cy - poleH).lineTo(cx + 9, cy - poleH - 4).lineTo(cx, cy - poleH + 3).closePath().fill(0xff4444).stroke({ width: 1, color: 0xcc2222 });
            // 到達までの時間（ぎりぎり背景付き）
            const timeStr = formatTimeHHMMSS(travel.secLeft);
            const timeText = new Text({
                text: timeStr,
                style: { fontSize: 11, fill: 0xffffff, fontWeight: "bold" },
            });
            timeText.anchor.set(0.5);
            timeText.x = cx;
            timeText.y = cy + 6;
            const pad = 2;
            const bg = new Graphics();
            bg.roundRect(cx - timeText.width / 2 - pad, cy + 6 - timeText.height / 2 - pad, timeText.width + pad * 2, timeText.height + pad * 2, 2).fill(0x000000, 0.65);
            _tileContainer.addChild(bg);
            _tileContainer.addChild(timeText);
        }
    }
}
