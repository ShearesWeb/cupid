import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { buildIndexes } from "../lib/indexes.ts";
import type { DirectorySnapshot, Snapshot } from "../lib/types.ts";
import { Review } from "./Review.tsx";

const snapshot: Snapshot = {
  syncedAt: "2026-08-01T00:00:00Z",
  warnings: [],
  ccas: [{ id: 1, name: "Chess", kind: "committee" }],
  positions: [{ id: 16, ccaId: 1, name: "Member", type: "main", capacity: 2, chairRank: [] }],
  applicants: [],
  committed: [],
  preallocations: [],
  quota: [],
  seats: [],
  outcomes: [],
  run: { assignments: [], events: [], unfilled: [] },
};

// Verbatim `directory_snapshot` payload for one added, one removed and one
// repointed appointment, as cupid-core serializes the change set.
const directory: DirectorySnapshot = {
  users: [
    { id: 1, name: "Ann", email: "ann@x" },
    { id: 2, name: "Bo", email: "bo@x" },
    { id: 3, name: "Cy", email: "cy@x" },
  ],
  ccas: [{ id: 1, name: "Chess", kind: "committee", tier: "none", type: "type-a", description: null, imageUrl: null }],
  positions: [{ id: 16, ccaId: 1, reportingPositionId: null, positionType: "member", name: "Member", description: null, capacity: null }],
  appointments: [],
  changes: [
    {
      kind: "add",
      appointment: {
        userId: 2,
        positionId: 16,
        commitmentPeriod: "semester-1",
        points: 0,
        teamStatus: "none",
        createdAt: null,
      },
    },
    { kind: "remove", userId: 1, positionId: 16 },
    { kind: "changePeriod", userId: 3, positionId: 16, from: "full-year", to: "semester-2" },
  ],
};

const noop = () => {};

function render() {
  return renderToStaticMarkup(
    <Review
      snapshot={snapshot}
      directory={directory}
      idx={buildIndexes(snapshot)}
      commitState={{
        previewed: false, excluded: [], accessChecked: false, exported: false,
        archived: false, purged: false, exportedRows: 0, branch: null, prUrl: null, archiveRows: 0,
      }}
      purgeText=""
      onCommitState={noop}
      onPurgeText={noop}
      onOpenMatch={noop}
      toast={noop}
      running={false}
      onRun={noop}
      onApplySnapshot={noop}
    />,
  );
}

describe("pending directory changes", () => {
  it("renders a row per change without dereferencing an absent appointment", () => {
    const html = render();
    expect(html).toContain("Ann");
    expect(html).toContain("Removed");
    expect(html).toContain("Bo");
    expect(html).toContain("Added");
    expect(html).toContain("Cy");
    expect(html).toContain("Full year -&gt; Semester 2");
    // Every change names its position, so none of them fell back to the id.
    expect(html).not.toContain("Position 16");
    expect(html).not.toContain("User 1");
  });
});
