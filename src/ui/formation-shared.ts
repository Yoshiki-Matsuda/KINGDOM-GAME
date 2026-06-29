/**
 * 編成画面 — 共有状態
 * 循環インポートを避けるため、3ファイル間で共有するstateをオブジェクトで一元管理
 */

export type FormationView = "hub" | "units" | "monsters";
export type CharPickerMode = "assign" | "browse";

export const shared = {
  formationEl: null as HTMLDivElement | null,
  characterPickerEl: null as HTMLDivElement | null,
  hubEl: null as HTMLDivElement | null,
  formationModalEl: null as HTMLDivElement | null,
  cardDetailEl: null as HTMLDivElement | null,
  statAllocEl: null as HTMLDivElement | null,
  formationView: "hub" as FormationView,
  charPickerMode: "assign" as CharPickerMode,
  editingUnitId: null as string | null,
  editingSlotIndex: null as 0 | 1 | 2 | null,
};

export function setMapPointerBlocked(blocked: boolean): void {
  const mapContainer = document.querySelector<HTMLElement>(".map-container");
  if (mapContainer) mapContainer.style.pointerEvents = blocked ? "none" : "";
}
