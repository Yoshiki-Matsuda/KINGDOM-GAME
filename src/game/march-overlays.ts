import type { TravelingDestinationOverlay } from "../map-view";
import {
  aiFactionName,
  getMapVisibleMarches,
  isAiOwnerId,
  type GameState,
} from "../shared/game-state";

function marchOwnerLabel(state: GameState, ownerId: string): string {
  if (isAiOwnerId(ownerId)) {
    return aiFactionName(state, ownerId) ?? ownerId;
  }
  return ownerId;
}

function ownerFlagColor(state: GameState, ownerId: string, localPlayerId: string): number {
  if (ownerId === localPlayerId) return 0xff4444;
  if (isAiOwnerId(ownerId)) {
    const suffix = ownerId.replace(/^ai_/, "");
    const faction = state.ai_factions?.find((f) => f.faction_id === suffix);
    return faction?.color ?? 0xcc6644;
  }
  return 0xff8800;
}

function ownerLineColor(state: GameState, ownerId: string, localPlayerId: string): number {
  if (ownerId === localPlayerId) return 0xff8888;
  if (isAiOwnerId(ownerId)) {
    const suffix = ownerId.replace(/^ai_/, "");
    const faction = state.ai_factions?.find((f) => f.faction_id === suffix);
    return faction?.color ?? 0xcc8866;
  }
  return 0xffaa66;
}

function ownerExploreMarkerColor(state: GameState, ownerId: string, localPlayerId: string): number {
  if (ownerId === localPlayerId) return 0x5dade2;
  if (isAiOwnerId(ownerId)) {
    const suffix = ownerId.replace(/^ai_/, "");
    const faction = state.ai_factions?.find((f) => f.faction_id === suffix);
    const base = faction?.color ?? 0x66aa88;
    return ((base & 0xfefefe) >> 1) | 0x66ccaa;
  }
  return 0x88ccaa;
}

function ownerExploreLineColor(state: GameState, ownerId: string, localPlayerId: string): number {
  if (ownerId === localPlayerId) return 0x88d4f0;
  if (isAiOwnerId(ownerId)) {
    const suffix = ownerId.replace(/^ai_/, "");
    const faction = state.ai_factions?.find((f) => f.faction_id === suffix);
    return faction?.color ?? 0x88bbaa;
  }
  return 0xaaddcc;
}

/** マップ上の旗・点線オーバーレイ用データ（自分・他プレイヤー・AI の遠征） */
export function buildMarchMapOverlays(
  state: GameState,
  localPlayerId: string,
  now: number = Date.now(),
): TravelingDestinationOverlay[] {
  const marches = getMapVisibleMarches(state, now);

  return marches.map((m) => {
    const isExplore = m.kind === "explore";
    const isAttack = m.kind === "attack";
    return {
      overlayKey: m.march_id,
      targetId: m.to_territory_id,
      arrivesAt: m.arrives_at,
      secLeft: (m.arrives_at - now) / 1000,
      unitNames: [m.unit_name ?? (m.owner_id === localPlayerId ? "遠征隊" : marchOwnerLabel(state, m.owner_id))],
      lineFromId:
        isAttack || isExplore
          ? m.home_territory_id
          : undefined,
      ownerId: m.owner_id,
      flagColor: isExplore
        ? ownerExploreMarkerColor(state, m.owner_id, localPlayerId)
        : ownerFlagColor(state, m.owner_id, localPlayerId),
      lineColor: isExplore
        ? ownerExploreLineColor(state, m.owner_id, localPlayerId)
        : ownerLineColor(state, m.owner_id, localPlayerId),
      marchKind: isExplore ? "explore" : isAttack ? "attack" : undefined,
    };
  });
}
