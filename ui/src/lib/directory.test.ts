import { describe, expect, it } from "vitest";
import { buildDirectoryIndex, capStatuses, capacityText, committeeCombos, describeRules, effectiveType, isFull } from "./directory";
import type { CapRules, CommitteeCapEntry, DirectoryAppointment, DirectoryPosition, DirectorySnapshot } from "./types";

const DEFAULT_RULES: CapRules = {
  academicYear: "AY26/27",
  committeeCombos: [
    { main: 2, sub: 0 },
    { main: 1, sub: 1 },
    { main: 0, sub: 3 },
  ],
  committeeCombosAppealed: [
    { main: 2, sub: 1 },
    { main: 1, sub: 2 },
    { main: 0, sub: 4 },
  ],
  sportsCultureMax: 5,
  sportsCultureMaxAppealed: 6,
  totalMax: 6,
  totalMaxAppealed: 7,
  totalMaxBothAppeals: 8,
  ccaTypes: {},
};

const position = (id: number, positionType: DirectoryPosition["positionType"], capacity: number | null): DirectoryPosition => ({
  id,
  ccaId: 1,
  reportingPositionId: null,
  positionType,
  name: `P${id}`,
  description: null,
  capacity,
});

const holder = (userId: number, positionId: number, status: DirectoryAppointment["status"] = "existing"): DirectoryAppointment => ({
  userId,
  positionId,
  commitmentPeriod: "full-year",
  points: 0,
  teamStatus: "none",
  createdAt: null,
  status,
});

const directory: DirectorySnapshot = {
  users: [
    { id: 1, name: "Ann", email: "ann@x" },
    { id: 2, name: "Ben", email: "ben@x" },
    { id: 3, name: "Cid", email: "cid@x" },
  ],
  ccas: [
    { id: 1, name: "Choir", kind: "culture", tier: "none", type: "none", description: null, imageUrl: null },
    { id: 2, name: "Empty", kind: "sports", tier: "none", type: "none", description: null, imageUrl: null },
  ],
  positions: [position(30, "member", null), position(10, "resident", 5), position(20, "lead", 1)],
  appointments: [holder(1, 20), holder(1, 10), holder(2, 30, "added")],
  changes: [{ kind: "remove", userId: 3, positionId: 30 }],
  committeeCaps: { window: "academic-year", rules: DEFAULT_RULES, entries: [], unclassifiedCcas: [] },
};

describe("buildDirectoryIndex", () => {
  const dx = buildDirectoryIndex(directory);

  it("orders a CCA's positions by type significance", () => {
    expect(dx.positionsByCca.get(1)?.map((p) => p.id)).toEqual([20, 30, 10]);
  });

  it("counts each person once per CCA, whatever they hold", () => {
    expect(dx.memberCount(1)).toBe(2);
    expect(dx.memberCount(2)).toBe(0);
  });

  it("keeps pending removals reviewable by position", () => {
    expect(dx.removedByPosition.get(30)).toEqual([{ userId: 3, positionId: 30 }]);
    expect(dx.holdersByPosition.get(30)?.map((a) => a.userId)).toEqual([2]);
  });
});

describe("capacity", () => {
  it("treats null as unlimited and zero as closed", () => {
    expect(isFull(position(1, "member", null), 99)).toBe(false);
    expect(isFull(position(1, "member", 0), 0)).toBe(true);
    expect(capacityText(position(1, "member", null), 3)).toBe("3/∞");
    expect(capacityText(position(1, "lead", 1), 1)).toBe("1/1");
  });
});

describe("capStatuses", () => {
  const entry = (patch: Partial<CommitteeCapEntry>, exceedsStandard = false): CommitteeCapEntry => ({
    userId: 1,
    appealed: false,
    overCap: false,
    combinedAppealed: false,
    overCombinedCap: false,
    counts: [
      {
        semester: null,
        main: 1,
        sub: 2,
        sportsCulture: 3,
        exceedsStandard,
        exceedsAppeal: false,
        exceedsSportsCulture: false,
        sportsCultureLimit: 5,
        exceedsTotal: false,
        totalLimit: 6,
      },
    ],
    duplicateCcas: [],
    seats: [],
    sportsCultureCcas: [],
    ...patch,
  });

  it("names each flag Rust raised, most serious first", () => {
    expect(capStatuses(entry({ overCap: true }, true))).toEqual(["over"]);
    expect(capStatuses(entry({ overCap: true, appealed: true }, true))).toEqual(["over-appeal"]);
    expect(capStatuses(entry({ appealed: true }, true))).toEqual(["appeal-used"]);
    expect(capStatuses(entry({ appealed: true }))).toEqual(["appealed"]);
    expect(capStatuses(entry({ overCap: true, duplicateCcas: [4] }, true))).toEqual(["over", "duplicate"]);
    expect(capStatuses(entry({ overCap: true, overCombinedCap: true }, true))).toEqual(["over", "over-combined"]);
    expect(capStatuses(entry({ combinedAppealed: true }))).toEqual(["combined-appealed"]);
    expect(capStatuses(entry({ combinedAppealed: true, overCombinedCap: true }))).toEqual(["over-combined-appeal"]);
  });

  it("is indexed by user", () => {
    const flagged = entry({ userId: 2, overCap: true }, true);
    const dx = buildDirectoryIndex({ ...directory, committeeCaps: { ...directory.committeeCaps, entries: [flagged] } });
    expect(dx.capByUser.get(2)).toBe(flagged);
    expect(dx.capByUser.has(1)).toBe(false);
  });
});

describe("cap rule wording", () => {
  it("writes combinations the way the white paper does", () => {
    expect(committeeCombos(DEFAULT_RULES.committeeCombos)).toBe("2M, 1M1S or 3S");
    expect(committeeCombos(DEFAULT_RULES.committeeCombosAppealed)).toBe("2M1S, 1M2S or 4S");
    expect(committeeCombos([{ main: 0, sub: 2 }])).toBe("2S");
    expect(committeeCombos([])).toBe("nothing");
  });

  it("describes every cap from the rules", () => {
    const d = describeRules(DEFAULT_RULES);
    expect(d.committee).toBe("2M, 1M1S or 3S (with appeal: 2M1S, 1M2S or 4S)");
    expect(d.total).toBe("6 (with CCA cap appeal: 7; with both appeals: 8)");
  });

  it("prefers this year's committee type over the database's", () => {
    const rules = { ...DEFAULT_RULES, ccaTypes: { "4": "type-b" as const } };
    expect(effectiveType(rules, { id: 4, type: "type-a" })).toBe("type-b");
    expect(effectiveType(rules, { id: 5, type: "none" })).toBe("none");
  });
});
