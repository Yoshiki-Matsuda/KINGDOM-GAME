/**
 * 開発用の仮マスデータ（DB/サーバーに接続しないとき用）
 * サーバーと同じ5フェーズ地形生成 + MOCK_SEED で再現可能。
 */

import type { GameState, Territory } from "./game-state";
import {
  DEFAULT_PLAYER_ID,
  DEFAULT_RESOURCES,
  DEFAULT_WORLD_CONFIG,
  LEVEL_TERRAIN,
} from "./game-state";
import { buildTerrainLevelGrid } from "./terrain-gen";

export const MOCK_SEED = 12345;

const HOME_COL = DEFAULT_WORLD_CONFIG.home_col;
const HOME_ROW = DEFAULT_WORLD_CONFIG.home_row;
const GRID_COLS = DEFAULT_WORLD_CONFIG.cols;
const GRID_ROWS = DEFAULT_WORLD_CONFIG.rows;

function buildMockTerritories(): Territory[] {
  const levelGrid = buildTerrainLevelGrid(GRID_COLS, GRID_ROWS, MOCK_SEED);
  const territories: Territory[] = [];
  for (let row = 0; row < GRID_ROWS; row++) {
    for (let col = 0; col < GRID_COLS; col++) {
      const id = `c_${col}_${row}`;
      const isHome = col === HOME_COL && row === HOME_ROW;
      const level = isHome ? 1 : levelGrid[row][col];
      const name = LEVEL_TERRAIN[level] ?? "平原";
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
    world: { ...DEFAULT_WORLD_CONFIG, terrain_seed: MOCK_SEED },
    territories: buildMockTerritories(),
    log: [],
    players: {
      [DEFAULT_PLAYER_ID]: {
        player_id: DEFAULT_PLAYER_ID,
        home_territory_id: `c_${HOME_COL}_${HOME_ROW}`,
        inventory: [],
        facilities: [],
        owned_cards: [],
        allied_player_ids: [],
        resources: DEFAULT_RESOURCES,
      },
    },
  };
}
