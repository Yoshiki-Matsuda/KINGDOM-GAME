import { FACILITIES, type FacilityId } from "./facilities";

const LEGACY_FACILITY_ALIASES: Record<string, FacilityId> = {
  training: "training_tower",
  workshop: "fortress",
  training_ground: "training_tower",
  armory: "fortress",
  monster_well: "monster_barracks",
  crystal_mine: "ironworks",
  barracks: "stronghold",
  research_lab: "library",
  magic_tower: "battle_lab",
  skill_shrine: "battle_lab",
  watchtower: "guardian_shrine",
  altar: "guardian_shrine",
  home_expansion: "fortress",
};

const FACILITY_COLORS: Partial<Record<FacilityId, number>> = {
  field: 0x2a3a18,
  lumber_mill: 0x3a2510,
  ironworks: 0x2a2a2a,
  quarry: 0x3a3a38,
  warehouse: 0x3a3018,
  trading_post: 0x352a14,
  fortress: 0x3a1212,
  stronghold: 0x3a2010,
  training_tower: 0x4a1018,
  monster_barracks: 0x1a2a1a,
  battle_lab: 0x1a2a3a,
  beast_lab: 0x3a2210,
  demihuman_lab: 0x2a3018,
  spirit_lab: 0x103038,
  undead_lab: 0x1e1a30,
  giant_lab: 0x303030,
  demon_lab: 0x380808,
  dragon_lab: 0x2a1838,
  library: 0x1a2a3a,
  hero_statue: 0x3a3018,
  guardian_shrine: 0x183828,
  war_god_shrine: 0x3a1030,
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
    return { color: 0x1a1810, icon: "" };
  }

  const facility = FACILITIES[canonicalId];
  return {
    color: FACILITY_COLORS[canonicalId] ?? 0x1a1810,
    icon: facility?.icon ?? "🏗️",
  };
}
