const DEFAULT_API_ORIGIN = "http://127.0.0.1:3000";

export const USE_MOCK_STATE = import.meta.env.VITE_USE_MOCK_STATE === "true";
export const API_ORIGIN = import.meta.env.VITE_API_ORIGIN ?? DEFAULT_API_ORIGIN;
export const STATE_URL = `${API_ORIGIN}/api/state`;
export const WS_URL =
  import.meta.env.VITE_WS_URL ??
  `${API_ORIGIN.replace(/^http/, "ws")}/ws`;
