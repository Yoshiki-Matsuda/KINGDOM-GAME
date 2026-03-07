import {
    Application,
    Container,
    Graphics,
    Text,
    FederatedPointerEvent,
} from "pixi.js";

// 本拠地グリッドサイズ（デフォルト7x7、施設で拡張可能）
const DEFAULT_HOME_GRID_SIZE = 7;

// アイソメトリック: タイルサイズ
const TILE_WIDTH = 80;
const TILE_HEIGHT = 40;

let _app: Application | null = null;
let _tileContainer: Container | null = null;
let _getFacility: ((col: number, row: number) => string | null) | null = null;
let _onTileClick: ((col: number, row: number, facility: string | null, screenX: number, screenY: number) => void) | null = null;
let _selectedTile: { col: number; row: number } | null = null;
let _gridSize: number = DEFAULT_HOME_GRID_SIZE;

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

/** 施設の色 */
function getFacilityColor(facility: string | null): number {
    if (!facility) return 0x3a5a40; // 空き地（緑）
    switch (facility) {
        case "barracks": return 0x8b4513; // 兵舎（茶）
        case "training": return 0x4169e1; // 訓練場（青）
        case "workshop": return 0x708090; // 工房（灰）
        case "energy_well": return 0x00ced1; // エナジーの泉（シアン）
        case "crystal_mine": return 0x9370db; // 水晶鉱山（紫）
        case "lumber_mill": return 0x8b4513; // 製材所（茶）
        case "training_ground": return 0xdc143c; // 訓練場（赤）
        case "armory": return 0x696969; // 武器庫（グレー）
        case "research_lab": return 0x4682b4; // 研究所（青）
        case "magic_tower": return 0x9932cc; // 魔法塔（紫）
        case "skill_shrine": return 0xff6347; // スキルの祠（オレンジ）
        case "warehouse": return 0xdaa520; // 倉庫（ゴールド）
        case "watchtower": return 0x2e8b57; // 見張り塔（緑）
        case "altar": return 0xff1493; // 祭壇（ピンク）
        default: return 0x556b2f;
    }
}

/** 施設のアイコン */
function getFacilityIcon(facility: string | null): string {
    if (!facility) return "";
    switch (facility) {
        case "barracks": return "🏠";
        case "training": return "⚔️";
        case "workshop": return "🔧";
        case "energy_well": return "⛲";
        case "crystal_mine": return "💎";
        case "lumber_mill": return "🪓";
        case "training_ground": return "⚔️";
        case "armory": return "🗡️";
        case "research_lab": return "🔬";
        case "magic_tower": return "🗼";
        case "skill_shrine": return "⛩️";
        case "warehouse": return "🏪";
        case "watchtower": return "🗼";
        case "altar": return "🛕";
        default: return "🏗️";
    }
}

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
        backgroundColor: 0x1a1a2e,
        resolution: window.devicePixelRatio || 1,
        autoDensity: true,
    });

    container.appendChild(app.canvas as HTMLCanvasElement);
    _app = app;

    _tileContainer = new Container();
    _tileContainer.eventMode = "static";

    // 中央に配置
    const centerOffset = Math.floor(_gridSize / 2);
    const centerScreen = coordToScreen(centerOffset, centerOffset);
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
            const { col, row } = screenToCoord(local.x, local.y);

            // グリッドサイズに基づいた座標オフセット
            const centerOffset = Math.floor(_gridSize / 2);
            const baseCol = 24 - centerOffset;
            const baseRow = 24 - centerOffset;

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
        const centerScreen = coordToScreen(centerOffset, centerOffset);
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
    const baseCol = 24 - centerOffset;
    const baseRow = 24 - centerOffset;

    // アイソメトリック順で描画
    for (let sum = 0; sum < _gridSize * 2 - 1; sum++) {
        for (let row = 0; row < _gridSize; row++) {
            const col = sum - row;
            if (col >= 0 && col < _gridSize) {
                const realCol = col + baseCol;
                const realRow = row + baseRow;
                const facility = _getFacility(realCol, realRow);

                const { x: cx, y: cy } = coordToScreen(col, row);
                const points = diamondPoints(cx, cy);

                const color = getFacilityColor(facility);
                const isCenter = realCol === 24 && realRow === 24;
                const isSelected = _selectedTile && _selectedTile.col === realCol && _selectedTile.row === realRow;

                // タイルを描画
                g.poly(points, true).fill(color);

                // 枠線
                if (isSelected) {
                    g.poly(points, true).stroke({ width: 3, color: 0x00ff00 });
                } else if (isCenter) {
                    g.poly(points, true).stroke({ width: 3, color: 0xffd700 });
                } else if (facility) {
                    g.poly(points, true).stroke({ width: 2, color: 0xffffff, alpha: 0.5 });
                } else {
                    g.poly(points, true).stroke({ width: 1, color: 0x888888, alpha: 0.3 });
                }

                // 施設アイコン
                if (facility || isCenter) {
                    const icon = isCenter ? "🏰" : getFacilityIcon(facility);
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
