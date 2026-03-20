/**
 * 開発用の仮マスデータ（DB/サーバーに接続しないとき用）
 * シード付き乱数でランダムな地形を生成し、1回スムージングしてマップっぽくする。
 */

import type { GameState, Territory } from "./game-state";
import { LEVEL_TERRAIN } from "./game-state";

const GRID_COLS = 48;
const GRID_ROWS = 48;
const HOME_COL = 24;
const HOME_ROW = 24;
const MOCK_SEED = 12345;

/** シード付き PRNG（mulberry32）。同じシードなら同じ並びになる */
function createRng(seed: number): () => number {
  return () => {
    seed = (seed + 0x6d2b79f5) >>> 0; // 32bit
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** ランダムな地形レベル（1〜6）のグリッドを生成し、1回スムージングしてまとまりを出す */
function buildLevelGrid(): number[][] {
  const rng = createRng(MOCK_SEED);
  const grid: number[][] = [];
  for (let row = 0; row < GRID_ROWS; row++) {
    grid[row] = [];
    for (let col = 0; col < GRID_COLS; col++) {
      grid[row][col] = Math.floor(rng() * 6) + 1;
    }
  }
  // 隣接マスと平均して 1〜6 に丸め（地形のまとまりができる）
  const next: number[][] = [];
  for (let row = 0; row < GRID_ROWS; row++) {
    next[row] = [];
    for (let col = 0; col < GRID_COLS; col++) {
      let sum = grid[row][col];
      let n = 1;
      if (row > 0) { sum += grid[row - 1][col]; n++; }
      if (row < GRID_ROWS - 1) { sum += grid[row + 1][col]; n++; }
      if (col > 0) { sum += grid[row][col - 1]; n++; }
      if (col < GRID_COLS - 1) { sum += grid[row][col + 1]; n++; }
      next[row][col] = Math.max(1, Math.min(6, Math.round(sum / n)));
    }
  }
  return next;
}

function buildMockTerritories(): Territory[] {
  const levelGrid = buildLevelGrid();
  const territories: Territory[] = [];
  for (let row = 0; row < GRID_ROWS; row++) {
    for (let col = 0; col < GRID_COLS; col++) {
      const id = `c_${col}_${row}`;
      const level = levelGrid[row][col];
      const name = LEVEL_TERRAIN[level] ?? "平原";
      const isHome = col === HOME_COL && row === HOME_ROW;
      territories.push({
        id,
        name,
        level,
        owner_id: isHome ? "player" : null,
        troops: isHome ? 10 : 2,
      });
    }
  }
  return territories;
}

/** 開発用の仮ゲーム状態（全マス入り）。DB未接続時に使用 */
export function getMockGameState(): GameState {
  return {
    turn: 1,
    phase: "idle",
    territories: buildMockTerritories(),
    log: [],
  };
}
