import {
    Application,
    Container,
    Graphics,
    Text,
    FederatedPointerEvent,
} from "pixi.js";
import { getFacilityVisual } from "./game/facility-visuals";
import { HOME_COL, HOME_ROW } from "./game/territories";
import {
    coordToScreen,
    diamondPoints,
    screenToCoord,
} from "./map-isometric";

// 本拠地グリッドサイズ（デフォルト7x7、施設で拡張可能）
const DEFAULT_HOME_GRID_SIZE = 7;

// アイソメトリック: タイルサイズ
const TILE_WIDTH = 80;
const TILE_HEIGHT = 40;
const TILE_DIMENSIONS = { width: TILE_WIDTH, height: TILE_HEIGHT } as const;

let _app: Application | null = null;
let _tileContainer: Container | null = null;
let _getFacility: ((col: number, row: number) => string | null) | null = null;
let _onTileClick: ((col: number, row: number, facility: string | null, screenX: number, screenY: number) => void) | null = null;
let _selectedTile: { col: number; row: number } | null = null;
let _gridSize: number = DEFAULT_HOME_GRID_SIZE;

export async function initHomeMapView(
    container: HTMLDivElement,
    getFacility: (col: number, row: number) => string | null,
    onTileClick: (col: number, row: number, facility: string | null, screenX: number, screenY: number) => void
) {
    _getFacility = getFacility;
    _onTileClick = onTileClick;

    // 既存のキャンバスがあれば削除
    if (_app) {
        _app.destroy(true);
        _app = null;
    }

    container.innerHTML = "";

    const app = new Application();
    await app.init({ 
        width: 500, 
        height: 350, 
        backgroundColor: 0x0a0a0f,
        resolution: window.devicePixelRatio || 1,
        autoDensity: true,
    });

    container.appendChild(app.canvas as HTMLCanvasElement);
    _app = app;

    _tileContainer = new Container();
    _tileContainer.eventMode = "static";

    // 中央に配置
    const centerOffset = Math.floor(_gridSize / 2);
    const centerScreen = coordToScreen(centerOffset, centerOffset, TILE_DIMENSIONS);
    _tileContainer.x = app.screen.width / 2 - centerScreen.x;
    _tileContainer.y = app.screen.height / 2 - centerScreen.y;

    app.stage.addChild(_tileContainer);

    // クリックイベント
    let clickStart = { x: 0, y: 0 };
    let dragging = false;

    app.stage.eventMode = "static";
    app.stage.hitArea = app.screen;

    app.stage.on("pointerdown", (e) => {
        clickStart = { x: e.global.x, y: e.global.y };
        dragging = true;
    });

    app.stage.on("pointerup", (e: FederatedPointerEvent) => {
        if (!dragging) return;
        dragging = false;

        const dist = Math.abs(e.global.x - clickStart.x) + Math.abs(e.global.y - clickStart.y);
        if (dist < 5 && _tileContainer) {
            const local = _tileContainer.toLocal(e.global);
            const { col, row } = screenToCoord(local.x, local.y, TILE_DIMENSIONS);

            // グリッドサイズに基づいた座標オフセット
            const centerOffset = Math.floor(_gridSize / 2);
            const baseCol = HOME_COL - centerOffset;
            const baseRow = HOME_ROW - centerOffset;

            if (col >= 0 && col < _gridSize && row >= 0 && row < _gridSize) {
                const realCol = col + baseCol;
                const realRow = row + baseRow;
                const facility = _getFacility?.(realCol, realRow) ?? null;
                _onTileClick?.(realCol, realRow, facility, e.global.x, e.global.y);
            }
        }
    });

    renderHomeGrid();
}

export function updateHomeMapView(
    getFacility: (col: number, row: number) => string | null,
    selectedTile?: { col: number; row: number } | null,
    gridSize?: number
) {
    _getFacility = getFacility;
    _selectedTile = selectedTile ?? null;
    _gridSize = gridSize ?? DEFAULT_HOME_GRID_SIZE;

    // グリッドサイズに応じてコンテナの中央位置を更新
    if (_app && _tileContainer) {
        const centerOffset = Math.floor(_gridSize / 2);
        const centerScreen = coordToScreen(centerOffset, centerOffset, TILE_DIMENSIONS);
        _tileContainer.x = _app.screen.width / 2 - centerScreen.x;
        _tileContainer.y = _app.screen.height / 2 - centerScreen.y;
    }

    renderHomeGrid();
}

function renderHomeGrid() {
    if (!_app || !_tileContainer || !_getFacility) return;

    _tileContainer.removeChildren();

    const g = new Graphics();
    _tileContainer.addChild(g);

    // グリッドサイズに基づいた座標オフセット（中央を24,24に固定）
    const centerOffset = Math.floor(_gridSize / 2);
    const baseCol = HOME_COL - centerOffset;
    const baseRow = HOME_ROW - centerOffset;

    // アイソメトリック順で描画
    for (let sum = 0; sum < _gridSize * 2 - 1; sum++) {
        for (let row = 0; row < _gridSize; row++) {
            const col = sum - row;
            if (col >= 0 && col < _gridSize) {
                const realCol = col + baseCol;
                const realRow = row + baseRow;
                const facility = _getFacility(realCol, realRow);

                const { x: cx, y: cy } = coordToScreen(col, row, TILE_DIMENSIONS);
                const points = diamondPoints(cx, cy, TILE_DIMENSIONS);

                const visual = getFacilityVisual(facility);
                const isCenter = realCol === HOME_COL && realRow === HOME_ROW;
                const isSelected = _selectedTile && _selectedTile.col === realCol && _selectedTile.row === realRow;

                const tileColor = facility || isCenter ? visual.color : 0x1a1810;
                g.poly(points, true).fill(tileColor);

                // タイル下辺に影を入れて立体感を出す
                const bottomEdge = [points[2], points[3], points[4], points[5], points[6], points[7]];
                g.poly(bottomEdge, false).stroke({ width: 2, color: 0x000000, alpha: 0.5 });

                if (isSelected) {
                    g.poly(points, true).stroke({ width: 2, color: 0xc9a84c });
                } else if (isCenter) {
                    g.poly(points, true).stroke({ width: 2, color: 0xc9a84c, alpha: 0.8 });
                } else if (facility) {
                    g.poly(points, true).stroke({ width: 1, color: 0xc9a84c, alpha: 0.35 });
                } else {
                    g.poly(points, true).stroke({ width: 1, color: 0x3a3020, alpha: 0.6 });
                }

                if (facility || isCenter) {
                    const icon = isCenter ? "🏰" : visual.icon;
                    const iconText = new Text({
                        text: icon,
                        style: { fontSize: 20 },
                    });
                    iconText.anchor.set(0.5);
                    iconText.x = cx;
                    iconText.y = cy;
                    _tileContainer.addChild(iconText);
                }
            }
        }
    }
}
