/**
 * 移動タイマー — 距離計算・出発・到着時送信
 */

import {
    ws, travelingUnits, setTravelingUnits,
    travelIntervalId, setTravelIntervalId,
    setAttackSourceId,
    formedUnitsList, setFormedUnitsList,
    render,
} from "../store";
import type { TravelingUnit } from "../store";
import { attackAction, deployAction } from "../shared/game-state";
import { BASE_TRAVEL_TIME_PER_TILE } from "./characters";

// Local parsing helper to avoid dependency on potentially missing map-view.ts
function parseTerritoryId(id: string): { col: number; row: number } | null {
    if (id.startsWith("c_")) {
        const parts = id.substring(2).split("_");
        if (parts.length === 2) {
            const col = parseInt(parts[0], 10);
            const row = parseInt(parts[1], 10);
            return { col, row };
        }
    }
    const parts = id.split("-");
    if (parts.length === 2) {
        const col = parseInt(parts[0], 10);
        const row = parseInt(parts[1], 10);
        return { col, row };
    }
    return null;
}

/** 本拠地(24,24)から領地までのマンハッタン距離（マス数） */
export function getDistanceFromHome(territoryId: string): number {
    const p = parseTerritoryId(territoryId);
    if (!p) return 0;
    return Math.abs(p.col - 24) + Math.abs(p.row - 24);
}

/** 2領地間のマンハッタン距離（マス数） */
export function getDistanceBetweenTerritories(fromId: string, toId: string): number {
    const pFrom = parseTerritoryId(fromId);
    const pTo = parseTerritoryId(toId);
    if (!pFrom || !pTo) return 0;
    return Math.abs(pFrom.col - pTo.col) + Math.abs(pFrom.row - pTo.row);
}

/** 移動時間（ミリ秒）。距離とユニット平均SPEEDから計算。SPEEDが高いほど短い */
export function getTravelTimeMs(distance: number, avgSpeed: number): number {
    if (distance <= 0 || avgSpeed <= 0) return 0;
    const refSpeed = 5;
    const secPerTile = BASE_TRAVEL_TIME_PER_TILE * (refSpeed / avgSpeed);
    return Math.max(0, Math.round(distance * secPerTile * 1000));
}

/** 到着したユニットのアクションをサーバーへ送信。攻撃の場合は同じ時間の帰還エントリを返す */
function sendTraveledAction(t: TravelingUnit): TravelingUnit | null {
    if (t.actionType === "return") return null;
    if (ws?.readyState !== WebSocket.OPEN) return null;
    if (t.actionType === "attack" && t.fromId != null) {
        ws.send(JSON.stringify(attackAction(
            t.fromId,
            t.targetId,
            t.count,
            t.energyPerBody,
            t.bodyNames,
            t.unitName,
            t.speedPerBody,
            t.skillsPerBody
        )));
        setAttackSourceId(null);
        const returnDurationMs = t.arrivalTime - t.departureTime;
        const now = Date.now();
        return {
            id: `travel-return-${t.unitId}-${now}`,
            unitId: t.unitId,
            unitName: t.unitName,
            actionType: "return",
            targetId: "",
            count: 0,
            energyPerBody: [],
            speedPerBody: [],
            bodyNames: [],
            skillsPerBody: [],
            departureTime: now,
            arrivalTime: now + returnDurationMs,
        };
    }
    if (t.actionType === "deploy") {
        ws.send(JSON.stringify(deployAction(t.targetId, t.count, t.energyPerBody, t.bodyNames)));
        setFormedUnitsList(formedUnitsList.filter((u) => u.id !== t.unitId));
    }
    return null;
}

/** 移動タイマーが未起動なら開始する */
export function startTravelIntervalIfNeeded(): void {
    if (travelIntervalId != null || travelingUnits.length === 0) return;
    const id = setInterval(() => {
        const now = Date.now();
        const remaining: TravelingUnit[] = [];
        for (const t of travelingUnits) {
            if (t.arrivalTime > now) {
                remaining.push(t);
            } else {
                if (t.actionType === "return") {
                    // 帰還完了はリストから削除するだけ
                } else {
                    const returnEntry = sendTraveledAction(t);
                    if (returnEntry) remaining.push(returnEntry);
                }
            }
        }
        setTravelingUnits(remaining);
        if (remaining.length === 0 && travelIntervalId != null) {
            clearInterval(travelIntervalId);
            setTravelIntervalId(null);
        }
        render();
    }, 500);
    setTravelIntervalId(id);
}
