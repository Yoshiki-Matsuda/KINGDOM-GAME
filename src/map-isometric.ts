export interface TileDimensions {
  width: number;
  height: number;
}

export function coordToScreen(
  col: number,
  row: number,
  dimensions: TileDimensions,
): { x: number; y: number } {
  return {
    x: (col - row) * (dimensions.width / 2),
    y: (col + row) * (dimensions.height / 2),
  };
}

export function screenToCoord(
  screenX: number,
  screenY: number,
  dimensions: TileDimensions,
): { col: number; row: number } {
  const halfWidth = dimensions.width / 2;
  const halfHeight = dimensions.height / 2;
  const col = (screenX / halfWidth + screenY / halfHeight) / 2;
  const row = (screenY / halfHeight - screenX / halfWidth) / 2;
  return { col: Math.round(col), row: Math.round(row) };
}

export function diamondPoints(
  centerX: number,
  centerY: number,
  dimensions: TileDimensions,
): number[] {
  const halfWidth = dimensions.width / 2;
  const halfHeight = dimensions.height / 2;
  return [
    centerX, centerY - halfHeight,
    centerX + halfWidth, centerY,
    centerX, centerY + halfHeight,
    centerX - halfWidth, centerY,
  ];
}
