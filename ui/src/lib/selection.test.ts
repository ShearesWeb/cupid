import { describe, expect, it } from "vitest";
import { groupAdds, heldBackRows } from "./selection";
import type { AssignmentView, Snapshot } from "./types";

const snap: Snapshot = {
  syncedAt: "2026-07-07T00:00:00Z",
  warnings: [],
  ccas: [
    { id: 1, name: "Chess" },
    { id: 2, name: "Media" },
  ],
  positions: [
    { id: 10, ccaId: 1, name: "Head", type: "main", capacity: 1, chairRank: [1, 2] },
    { id: 11, ccaId: 1, name: "Sub", type: "sub", capacity: 2, chairRank: [3] },
    { id: 20, ccaId: 2, name: "Chair", type: "main", capacity: 1, chairRank: [] },
  ],
  applicants: [
    { id: 1, name: "Ann", email: "ann@x", prefs: [10, 20] },
    { id: 2, name: "Ben", email: "ben@x", prefs: [10] },
    { id: 3, name: "Cid", email: "cid@x", prefs: [11] },
  ],
  committed: [],
  preallocations: [],
  quota: [],
  seats: [],
  outcomes: [],
  run: null,
};

const add = (applicantId: number, positionId: number): AssignmentView => ({
  applicantId,
  positionId,
  kind: "allocated",
  chairRank: null,
  prefRank: null,
});

describe("groupAdds", () => {
  it("nests adds under their position and cca in snapshot order", () => {
    const groups = groupAdds([add(3, 11), add(1, 20), add(2, 10)], snap);
    expect(groups.map((g) => g.name)).toEqual(["Chess", "Media"]);
    expect(groups[0].positions.map((p) => p.name)).toEqual(["Head", "Sub"]);
    expect(groups[0].positions[0].adds).toEqual([add(2, 10)]);
    expect(groups[1].positions[0].adds).toEqual([add(1, 20)]);
  });

  it("omits ccas and positions the run gave no adds", () => {
    const groups = groupAdds([add(2, 10)], snap);
    expect(groups.map((g) => g.name)).toEqual(["Chess"]);
    expect(groups[0].positions.map((p) => p.positionId)).toEqual([10]);
  });
});

describe("heldBackRows", () => {
  it("counts the prefs and chair rankings the purge will leave in place", () => {
    // Position 10 is held back: Ann and Ben both ranked it (2 pref rows), and
    // its chair ranked 2 candidates (2 ranking rows).
    expect(heldBackRows(snap, new Set([10]))).toBe(4);
  });

  it("is zero when nothing is held back", () => {
    expect(heldBackRows(snap, new Set())).toBe(0);
  });
});
