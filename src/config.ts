const DEFAULT_PVP_ORIGIN = "http://127.0.0.1:3000";
const DEFAULT_PVE_ORIGIN = "http://127.0.0.1:3001";

export type GameMode = "pvp" | "pve";

export const USE_MOCK_STATE = import.meta.env.VITE_USE_MOCK_STATE === "true";

/** npm run dev 時のみ。VITE_DEV_AUTO_LOGIN=false で無効化 */
export const DEV_AUTO_LOGIN =
  import.meta.env.DEV && import.meta.env.VITE_DEV_AUTO_LOGIN !== "false";
export const DEV_AUTO_LOGIN_USERNAME =
  import.meta.env.VITE_DEV_USERNAME ?? "offline_test";
export const DEV_AUTO_LOGIN_PASSWORD =
  import.meta.env.VITE_DEV_PASSWORD ?? "test12345";

export const PVP_API_ORIGIN =
  import.meta.env.VITE_PVP_API_ORIGIN ?? DEFAULT_PVP_ORIGIN;
export const PVE_API_ORIGIN =
  import.meta.env.VITE_PVE_API_ORIGIN ?? DEFAULT_PVE_ORIGIN;

/** 後方互換 */
export const API_ORIGIN = import.meta.env.VITE_API_ORIGIN ?? PVP_API_ORIGIN;

export function apiOriginForMode(mode: GameMode): string {
  return mode === "pve" ? PVE_API_ORIGIN : PVP_API_ORIGIN;
}

export function wsUrlForMode(mode: GameMode): string {
  const origin = apiOriginForMode(mode);
  return import.meta.env.VITE_WS_URL && mode === "pvp"
    ? import.meta.env.VITE_WS_URL
    : `${origin.replace(/^http/, "ws")}/ws`;
}

export function stateUrlForMode(mode: GameMode): string {
  return `${apiOriginForMode(mode)}/api/state`;
}

/** ログイン画面の PVP サーバー一覧（将来の複数サーバー用） */
export interface PvpServerEntry {
  id: string;
  label: string;
}

export const PVP_SERVERS: PvpServerEntry[] = [
  { id: "shared", label: "共有ワールド" },
];
