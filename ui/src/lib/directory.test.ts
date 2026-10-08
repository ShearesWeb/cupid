import { describe, expect, it } from "vitest";
import { buildDirectoryIndex, capacityText, isFull } from "./directory";
import type { DirectoryAppointment, DirectoryPosition, DirectorySnapshot } from "./types";

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
