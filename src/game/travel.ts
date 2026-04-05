/**
 * 移動タイマー — 距離計算・出発・到着時送信
 */

import {
    ws, travelingUnits, setTravelingUnits,
    travelIntervalId, setTravelIntervalId,
    setAttackSourceId,
    formedUnitsList, setFormedUnitsList,
    gameState,
    render,
} from "../store";
import type { TravelingUnit } from "../store";
import { attackAction, deployAction } from "../shared/game-state";
import { BASE_TRAVEL_TIME_PER_TILE } from "./characters";
import { getAdjacentAttackSource } from "./combat";

/** 本拠地(24,24)から領地までのマンハッタン距離（マス数） */
export { getDistanceBetweenTerritories, getDistanceFromHome } from "./territories";

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
    if (t.actionType === "attack") {
        const fromId = t.fromId ?? getAdjacentAttackSource(gameState, t.targetId) ?? null;
        if (fromId == null || fromId === "") {
            console.warn("[kingdom] 攻撃元領地を決められませんでした。隣接する自領を確認してください。", t);
            return null;
        }
        ws.send(JSON.stringify(attackAction(
            fromId,
            t.targetId,
            t.count,
            t.monstersPerBody,
            t.bodyNames,
            t.unitName,
            t.speedPerBody,
            t.skillsPerBody,
            t.statsPerBody,
            t.ownedCardIndices
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
            monstersPerBody: [],
            speedPerBody: [],
            bodyNames: [],
            skillsPerBody: [],
            statsPerBody: [],
            departureTime: now,
            arrivalTime: now + returnDurationMs,
        };
    }
    if (t.actionType === "deploy") {
        ws.send(JSON.stringify(deployAction(t.targetId, t.count, t.monstersPerBody, t.bodyNames)));
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
