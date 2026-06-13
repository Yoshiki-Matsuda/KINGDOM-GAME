import type { BuiltFacility, InventoryItem } from "./shared/game-state";
import type { FormedUnit, TravelingUnit } from "./store";
import {
  formedUnitsList,
  gameState,
  getLocalPlayerId,
  setGameState,
  setTravelingUnits,
  travelingUnits,
} from "./store";
import { commitFormedUnits } from "./game/formed-units-persist";

export function appendFormedUnit(unit: FormedUnit): void {
  commitFormedUnits([...formedUnitsList, unit]);
}

export function removeFormedUnit(unitId: string): void {
  commitFormedUnits(formedUnitsList.filter((unit) => unit.id !== unitId));
}

export function appendTravelingUnit(unit: TravelingUnit): void {
  setTravelingUnits([...travelingUnits, unit]);
}

export function replaceLocalPlayerState(update: {
  inventory?: InventoryItem[];
  facilities?: BuiltFacility[];
}): void {
  const playerId = getLocalPlayerId();
  const player = gameState.players[playerId];
  const nextPlayers = {
    ...gameState.players,
    ...(player
      ? {
          [playerId]: {
            ...player,
            ...(update.inventory !== undefined ? { inventory: update.inventory } : {}),
            ...(update.facilities !== undefined ? { facilities: update.facilities } : {}),
          },
        }
      : {}),
  };

  setGameState({
    ...gameState,
    players: nextPlayers,
  });
}
