import type { GameMode } from "../config";
import { apiOriginForMode } from "../config";

const PROBE_TIMEOUT_MS = 1000;

let cachedModes: GameMode[] | null = null;
let cacheExpiresAt = 0;
const CACHE_TTL_MS = 5000;

/** /health でサーバー応答を確認 */
export async function isServerReachable(mode: GameMode): Promise<boolean> {
  try {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), PROBE_TIMEOUT_MS);
    const response = await fetch(`${apiOriginForMode(mode)}/health`, {
      signal: controller.signal,
    });
    clearTimeout(timer);
    return response.ok;
  } catch {
    return false;
  }
}

/** 起動中のサーバーモード一覧（pvp → pve の順） */
export async function discoverAvailableModes(force = false): Promise<GameMode[]> {
  const now = Date.now();
  if (!force && cachedModes && now < cacheExpiresAt) {
    return cachedModes;
  }

  const candidates: GameMode[] = ["pvp", "pve"];
  const checks = await Promise.all(
    candidates.map(async (mode) => ({ mode, up: await isServerReachable(mode) })),
  );
  cachedModes = checks.filter((c) => c.up).map((c) => c.mode);
  cacheExpiresAt = now + CACHE_TTL_MS;
  return cachedModes;
}

/**
 * 接続先モードを決定する。
 * 保存済みモードのサーバーが起動していればそれを優先し、
 * 未起動なら起動中のもう一方へフォールバックする。
 */
export async function resolveInitialGameMode(stored: GameMode): Promise<GameMode> {
  const available = await discoverAvailableModes();
  if (available.length === 0) return stored;
  if (available.includes(stored)) return stored;
  return available[0];
}

/** 指定モードが利用可能か。利用不可なら起動中の代替モードを返す */
export async function preferReachableMode(preferred: GameMode): Promise<GameMode> {
  return resolveInitialGameMode(preferred);
}
