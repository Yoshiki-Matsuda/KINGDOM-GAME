import { TERRAIN_LEVEL_RIVER } from "../shared/terrain-gen";

/** 川タイル（Lv4）のグリッド軸。アイソメ描画では col 軸と row 軸でテクスチャを90度ずらす */
export type RiverAxis = "col" | "row";

export { TERRAIN_LEVEL_RIVER as RIVER_TERRAIN_LEVEL };

export function isRiverLevel(level: number): boolean {
  return level === TERRAIN_LEVEL_RIVER;
}

/** 隣接する川マスから、直線川の向き（col=横一列 / row=縦一列）を決める */
export function pickRiverAxis(
  col: number,
  row: number,
  levelAt: (c: number, r: number) => number | undefined,
): RiverAxis {
  const isRiver = (c: number, r: number) => isRiverLevel(levelAt(c, r) ?? 0);

  const hasColNeighbor = isRiver(col - 1, row) || isRiver(col + 1, row);
  const hasRowNeighbor = isRiver(col, row - 1) || isRiver(col, row + 1);

  if (hasRowNeighbor && !hasColNeighbor) return "row";
  if (hasColNeighbor && !hasRowNeighbor) return "col";

  // 端マスや両隣接（理論上は生成されない）: 2マス先まで見て軸を推定
  const onColLine =
    isRiver(col + 1, row) || isRiver(col + 2, row) ||
    isRiver(col - 1, row) || isRiver(col - 2, row);
  const onRowLine =
    isRiver(col, row + 1) || isRiver(col, row + 2) ||
    isRiver(col, row - 1) || isRiver(col, row - 2);

  if (onRowLine && !onColLine) return "row";
  return "col";
}
