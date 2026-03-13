import { FACILITIES, type FacilityId } from "./facilities";

const LEGACY_FACILITY_ALIASES: Record<string, FacilityId> = {
  training: "training_ground",
  workshop: "armory",
};

const FACILITY_COLORS: Partial<Record<FacilityId, number>> = {
  barracks: 0x8b4513,
  training_ground: 0xdc143c,
  armory: 0x696969,
  energy_well: 0x00ced1,
  crystal_mine: 0x9370db,
  lumber_mill: 0x8b4513,
  research_lab: 0x4682b4,
  magic_tower: 0x9932cc,
  skill_shrine: 0xff6347,
  warehouse: 0xdaa520,
  watchtower: 0x2e8b57,
  altar: 0xff1493,
  home_expansion: 0xffd700,
};

export function getCanonicalFacilityId(facilityId: string | null): FacilityId | null {
  if (!facilityId) return null;
  if (facilityId in FACILITIES) {
    return facilityId as FacilityId;
  }
  return LEGACY_FACILITY_ALIASES[facilityId] ?? null;
}

export function getFacilityVisual(facilityId: string | null): { color: number; icon: string } {
  const canonicalId = getCanonicalFacilityId(facilityId);
  if (!canonicalId) {
    return { color: 0x3a5a40, icon: "" };
  }

  const facility = FACILITIES[canonicalId];
  return {
    color: FACILITY_COLORS[canonicalId] ?? 0x556b2f,
    icon: facility?.icon ?? "🏗️",
  };
}
