import { describe, expect, it } from "vitest";
import { buildIndexes, pk } from "./indexes";
import type { Snapshot } from "./types";

const snap: Snapshot = {
  syncedAt: "2026-07-07T00:00:00Z",
  warnings: [],
  ccas: [{ id: 1, name: "Chess" }],
  positions: [
    { id: 10, ccaId: 1, name: "Head", type: "main", capacity: 1, chairRank: [1, 2] },
    { id: 11, ccaId: 1, name: "Sub", type: "sub", capacity: 2, chairRank: [3] },
  ],
  applicants: [
    { id: 1, name: "Ann", email: "ann@x", prefs: [10] },
    { id: 2, name: "Ben", email: "ben@x", prefs: [10] },
    { id: 3, name: "Cid", email: "cid@x", prefs: [11] },
  ],
  committed: [{ applicantId: 3, positionId: 11 }],
  preallocations: [{ applicantId: 3, positionId: 11, note: null }],
  quota: [
    { applicantId: 1, main: 1, block: 0, sub: 0,
      canAddMain: true, canAddBlock: true, canAddSub: true, over: false },
  ],
  seats: [
    { positionId: 10, seated: [{ applicantId: 1, status: "allocated" }] },
    { positionId: 11, seated: [{ applicantId: 3, status: "existing" }] },
  ],
  outcomes: [
    { applicantId: 1, positionId: 10, status: "allocated", label: "Allocated", detail: "Newly allocated by this run." },
    { applicantId: 2, positionId: 10, status: "displaced", label: "Displaced", detail: "Displaced by Ann (chair-rank 1)." },
  ],
  run: {
    assignments: [
      { applicantId: 1, positionId: 10, kind: "allocated", chairRank: 1, prefRank: 1 },
      { applicantId: 3, positionId: 11, kind: "preallocated", chairRank: 1, prefRank: 1 },
    ],
    events: [
      { applicantId: 2, positionId: 10, seq: 0, kind: "accept", byApplicantId: null, detail: "Allocated at chair-rank 2 (their preference #1)." },
      { applicantId: 2, positionId: 10, seq: 1, kind: "displace", byApplicantId: 1, detail: "Displaced by Ann (chair-rank 1)." },
    ],
    unfilled: [{ positionId: 11, open: 1 }],
  },
};

describe("buildIndexes", () => {
  const idx = buildIndexes(snap);
  it("maps entities by id", () => {
    expect(idx.posById.get(10)!.name).toBe("Head");
    expect(idx.ccaById.get(1)!.name).toBe("Chess");
  });
  it("ranks are 1-based and null when absent", () => {
    expect(idx.chairRankOf(10, 1)).toBe(1);
    expect(idx.chairRankOf(10, 99)).toBeNull();
    expect(idx.prefRankOf(1, 10)).toBe(1);
  });
  it("pairs index outcomes and events", () => {
    expect(idx.outcomeByPair.get(pk(1, 10))!.status).toBe("allocated");
    expect(idx.eventsByPair.get(pk(2, 10))!.length).toBe(2);
  });
  it("splits assignments by kind", () => {
    expect(idx.newAllocations.length).toBe(1);
    expect(idx.preallocatedAllocations.length).toBe(1);
  });
});
