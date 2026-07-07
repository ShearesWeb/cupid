import type {
  ApplicantView,
  AssignmentView,
  CcaView,
  EventView,
  OutcomeView,
  PositionView,
  QuotaView,
  SeatView,
  Snapshot,
} from "./types";

export const pk = (aid: number, pid: number) => `${aid}|${pid}`;

export interface Indexes {
  posById: Map<number, PositionView>;
  appById: Map<number, ApplicantView>;
  ccaById: Map<number, CcaView>;
  committedSet: Set<string>; // pk
  outcomeByPair: Map<string, OutcomeView>;
  eventsByPair: Map<string, EventView[]>; // seq-ordered (input order)
  quotaByApp: Map<number, QuotaView>;
  seatsByPos: Map<number, SeatView[]>;
  chairRankOf: (pid: number, aid: number) => number | null; // 1-based
  prefRankOf: (aid: number, pid: number) => number | null; // 1-based
  chairRankIndex: Map<string, number>; // pk -> 1-based
  prefRankIndex: Map<string, number>; // pk -> 1-based
  newAllocations: AssignmentView[]; // kind === "allocated"
  appealAllocations: AssignmentView[]; // kind === "appealed"
}

export function buildIndexes(s: Snapshot): Indexes {
  const posById = new Map<number, PositionView>();
  for (const p of s.positions) posById.set(p.id, p);

  const appById = new Map<number, ApplicantView>();
  for (const a of s.applicants) appById.set(a.id, a);

  const ccaById = new Map<number, CcaView>();
  for (const c of s.ccas) ccaById.set(c.id, c);

  const committedSet = new Set<string>();
  for (const pair of s.committed) committedSet.add(pk(pair.applicantId, pair.positionId));

  const outcomeByPair = new Map<string, OutcomeView>();
  for (const o of s.outcomes) outcomeByPair.set(pk(o.applicantId, o.positionId), o);

  const quotaByApp = new Map<number, QuotaView>();
  for (const q of s.quota) quotaByApp.set(q.applicantId, q);

  const seatsByPos = new Map<number, SeatView[]>();
  for (const sv of s.seats) seatsByPos.set(sv.positionId, sv.seated);

  const chairRankIndex = new Map<string, number>();
  for (const p of s.positions) {
    p.chairRank.forEach((aid, i) => {
      chairRankIndex.set(pk(aid, p.id), i + 1);
    });
  }

  const prefRankIndex = new Map<string, number>();
  for (const a of s.applicants) {
    a.prefs.forEach((pid, i) => {
      prefRankIndex.set(pk(a.id, pid), i + 1);
    });
  }

  const eventsByPair = new Map<string, EventView[]>();
  const assignments = s.run?.assignments ?? [];
  const events = s.run?.events ?? [];
  for (const e of events) {
    const key = pk(e.applicantId, e.positionId);
    const list = eventsByPair.get(key);
    if (list) {
      list.push(e);
    } else {
      eventsByPair.set(key, [e]);
    }
  }

  const newAllocations = assignments.filter((a) => a.kind === "allocated");
  const appealAllocations = assignments.filter((a) => a.kind === "appealed");

  const chairRankOf = (pid: number, aid: number): number | null =>
    chairRankIndex.get(pk(aid, pid)) ?? null;

  const prefRankOf = (aid: number, pid: number): number | null =>
    prefRankIndex.get(pk(aid, pid)) ?? null;

  return {
    posById,
    appById,
    ccaById,
    committedSet,
    outcomeByPair,
    eventsByPair,
    quotaByApp,
    seatsByPos,
    chairRankOf,
    prefRankOf,
    chairRankIndex,
    prefRankIndex,
    newAllocations,
    appealAllocations,
  };
}
