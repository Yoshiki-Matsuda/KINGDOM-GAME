/**
 * サーバー `random_level_grid` と同等の5フェーズ地形生成（オフライン/mock 用）
 */

const TERRAIN_BASE_WEIGHTS = [25, 35, 40] as const;
export const TERRAIN_LEVEL_RIVER = 4;
export const TERRAIN_LEVEL_ALPINE = 5;
export const TERRAIN_LEVEL_MOUNTAIN = 6;
export const TERRAIN_LEVEL_PERIL = 7;
export const TERRAIN_LEVEL_DEMON = 8;
export const TERRAIN_LEVEL_DEEP = 9;
const MOUNTAIN_SEED_CHANCE = [15, 1000] as const;
const MOUNTAIN_SPREAD_CHANCE = [45, 100] as const;
const RIVER_SEGMENT_CHANCE = [25, 1000] as const;
const DEEP_TERRAIN_CHANCE_9 = [12, 10000] as const;
const DEEP_TERRAIN_CHANCE_8 = [35, 10000] as const;
const DEEP_TERRAIN_CHANCE_7 = [100, 10000] as const;

export type TerrainRng = () => number;

/** mulberry32 — 同じシードなら同じ並び */
export function createTerrainRng(seed: number): TerrainRng {
  return () => {
    seed = (seed + 0x6d2b79f5) >>> 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function genRatio(rng: TerrainRng, num: number, den: number): boolean {
  return Math.floor(rng() * den) < num;
}

function pickBaseLevel(rng: TerrainRng): number {
  const total = TERRAIN_BASE_WEIGHTS.reduce((a, b) => a + b, 0);
  let roll = Math.floor(rng() * total);
  for (let i = 0; i < TERRAIN_BASE_WEIGHTS.length; i++) {
    roll -= TERRAIN_BASE_WEIGHTS[i];
    if (roll < 0) return i + 1;
  }
  return 3;
}

function fillBaseTerrain(cols: number, rows: number, rng: TerrainRng): number[][] {
  const grid: number[][] = [];
  for (let row = 0; row < rows; row++) {
    grid[row] = [];
    for (let col = 0; col < cols; col++) {
      grid[row][col] = pickBaseLevel(rng);
    }
  }
  return grid;
}

function clusterBaseTerrain(
  grid: number[][],
  cols: number,
  rows: number,
  rng: TerrainRng,
): number[][] {
  const next = grid.map((row) => row.slice());
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const level = grid[row][col];
      let same = 0;
      for (const [nr, nc] of neighbors4(row, col, rows, cols)) {
        if (grid[nr][nc] === level) same++;
      }
      if (same >= 2 || !genRatio(rng, 25, 100)) continue;
      const neighbors = neighbors4(row, col, rows, cols);
      if (neighbors.length === 0) continue;
      const [nr, nc] = neighbors[Math.floor(rng() * neighbors.length)];
      next[row][col] = grid[nr][nc];
    }
  }
  return next;
}

function neighbors4(row: number, col: number, rows: number, cols: number): [number, number][] {
  const out: [number, number][] = [];
  if (row > 0) out.push([row - 1, col]);
  if (row + 1 < rows) out.push([row + 1, col]);
  if (col > 0) out.push([row, col - 1]);
  if (col + 1 < cols) out.push([row, col + 1]);
  return out;
}

function spreadMountains(grid: number[][], cols: number, rows: number, rng: TerrainRng): void {
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const level = grid[row][col];
      if (level === 3 && genRatio(rng, MOUNTAIN_SEED_CHANCE[0], MOUNTAIN_SEED_CHANCE[1])) {
        grid[row][col] = TERRAIN_LEVEL_MOUNTAIN;
      }
    }
  }
  const toSpread: [number, number][] = [];
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      if (grid[row][col] !== TERRAIN_LEVEL_MOUNTAIN) continue;
      for (const [nr, nc] of neighbors4(row, col, rows, cols)) {
        const neighbor = grid[nr][nc];
        if (
          neighbor !== TERRAIN_LEVEL_MOUNTAIN &&
          neighbor !== TERRAIN_LEVEL_ALPINE &&
          neighbor !== TERRAIN_LEVEL_RIVER
        ) {
          toSpread.push([nr, nc]);
        }
      }
    }
  }
  for (const [row, col] of toSpread) {
    if (
      grid[row][col] !== TERRAIN_LEVEL_MOUNTAIN &&
      grid[row][col] !== TERRAIN_LEVEL_ALPINE &&
      grid[row][col] !== TERRAIN_LEVEL_RIVER &&
      genRatio(rng, MOUNTAIN_SPREAD_CHANCE[0], MOUNTAIN_SPREAD_CHANCE[1])
    ) {
      grid[row][col] = TERRAIN_LEVEL_ALPINE;
    }
  }
}

function riverSegmentCells(startCol: number, startRow: number, dc: number, dr: number): [number, number][] {
  return [
    [startCol, startRow],
    [startCol + dc, startRow + dr],
    [startCol + dc * 2, startRow + dr * 2],
  ];
}

function canPlaceRiverSegment(
  grid: number[][],
  cells: [number, number][],
  rows: number,
  cols: number,
): boolean {
  for (const [c, r] of cells) {
    if (
      grid[r][c] === TERRAIN_LEVEL_MOUNTAIN ||
      grid[r][c] === TERRAIN_LEVEL_ALPINE ||
      grid[r][c] === TERRAIN_LEVEL_RIVER
    ) return false;
    for (const [nr, nc] of neighbors4(r, c, rows, cols)) {
      if (grid[nr][nc] === TERRAIN_LEVEL_RIVER) return false;
    }
  }
  return true;
}

function shuffleInPlace<T>(arr: T[], rng: TerrainRng): void {
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(rng() * (i + 1));
    [arr[i], arr[j]] = [arr[j], arr[i]];
  }
}

function placeRiverSegments(grid: number[][], cols: number, rows: number, rng: TerrainRng): void {
  const order: [number, number][] = [];
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      order.push([col, row]);
    }
  }
  shuffleInPlace(order, rng);

  for (const [startCol, startRow] of order) {
    if (!genRatio(rng, RIVER_SEGMENT_CHANCE[0], RIVER_SEGMENT_CHANCE[1])) continue;
    const horizontal = rng() < 0.5;
    const options: [number, number][] = [];
    if (horizontal) {
      if (startCol + 2 < cols) options.push([1, 0]);
      if (startCol >= 2) options.push([-1, 0]);
    } else {
      if (startRow + 2 < rows) options.push([0, 1]);
      if (startRow >= 2) options.push([0, -1]);
    }
    if (options.length === 0) continue;
    const [dc, dr] = options[Math.floor(rng() * options.length)];
    const cells = riverSegmentCells(startCol, startRow, dc, dr);
    if (!canPlaceRiverSegment(grid, cells, rows, cols)) continue;
    for (const [c, r] of cells) {
      grid[r][c] = TERRAIN_LEVEL_RIVER;
    }
  }
}

function scatterDeepTerrain(grid: number[][], cols: number, rows: number, rng: TerrainRng): void {
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const level = grid[row][col];
      if (
        level === TERRAIN_LEVEL_RIVER ||
        level === TERRAIN_LEVEL_MOUNTAIN ||
        level === TERRAIN_LEVEL_ALPINE ||
        level >= TERRAIN_LEVEL_PERIL
      ) continue;
      if (genRatio(rng, DEEP_TERRAIN_CHANCE_9[0], DEEP_TERRAIN_CHANCE_9[1])) {
        grid[row][col] = TERRAIN_LEVEL_DEEP;
      } else if (genRatio(rng, DEEP_TERRAIN_CHANCE_8[0], DEEP_TERRAIN_CHANCE_8[1])) {
        grid[row][col] = TERRAIN_LEVEL_DEMON;
      } else if (genRatio(rng, DEEP_TERRAIN_CHANCE_7[0], DEEP_TERRAIN_CHANCE_7[1])) {
        grid[row][col] = TERRAIN_LEVEL_PERIL;
      }
    }
  }
}

/** シード付き地形レベルグリッド（Lv1-9） */
export function buildTerrainLevelGrid(cols: number, rows: number, seed: number): number[][] {
  const rng = createTerrainRng(seed >>> 0);
  let grid = fillBaseTerrain(cols, rows, rng);
  grid = clusterBaseTerrain(grid, cols, rows, rng);
  spreadMountains(grid, cols, rows, rng);
  placeRiverSegments(grid, cols, rows, rng);
  scatterDeepTerrain(grid, cols, rows, rng);
  return grid;
}
