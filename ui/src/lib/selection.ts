// selection.ts — the review checklist's read model. An operator can hold a
// whole position back from a commit; the same set then scopes the purge, so
// the held-back role's preferences survive into the next cycle. These are the
// pure parts: grouping the run's adds for the checklist, and counting what a
// scoped purge leaves behind.
import type { AssignmentView, Snapshot } from "./types";

export interface PositionGroup {
  positionId: number;
  name: string;
  adds: AssignmentView[];
}

export interface CcaGroup {
  ccaId: number;
  name: string;
  positions: PositionGroup[];
  addCount: number;
}

/// The run's adds nested CCA → position, both in snapshot order. Only
/// positions that gained a seat appear: a position with nothing to export has
/// nothing to hold back.
export function groupAdds(adds: AssignmentView[], snapshot: Snapshot): CcaGroup[] {
  const byPosition = new Map<number, AssignmentView[]>();
  for (const a of adds) {
    const list = byPosition.get(a.positionId);
    if (list) list.push(a);
    else byPosition.set(a.positionId, [a]);
  }

  const groups: CcaGroup[] = [];
  const byCca = new Map<number, CcaGroup>();
  for (const cca of snapshot.ccas) {
    const group: CcaGroup = { ccaId: cca.id, name: cca.name, positions: [], addCount: 0 };
    byCca.set(cca.id, group);
    groups.push(group);
  }
  for (const position of snapshot.positions) {
    const positionAdds = byPosition.get(position.id);
    const group = byCca.get(position.ccaId);
    if (!positionAdds || !group) continue;
    group.positions.push({ positionId: position.id, name: position.name, adds: positionAdds });
    group.addCount += positionAdds.length;
  }
  return groups.filter((g) => g.positions.length > 0);
}

/// The adds that will actually be exported, given the held-back positions.
export function includedAdds(adds: AssignmentView[], excluded: Set<number>): AssignmentView[] {
  return adds.filter((a) => !excluded.has(a.positionId));
}

/// Preference and chair-ranking rows a scoped purge leaves in the database:
/// applicant prefs pointing at a held-back position, plus those positions'
/// chair rankings. Mirrors purge's `position_id <> ALL(excluded)`.
export function heldBackRows(snapshot: Snapshot, excluded: Set<number>): number {
  const prefs = snapshot.applicants.reduce(
    (n, a) => n + a.prefs.filter((pid) => excluded.has(pid)).length,
    0,
  );
  return snapshot.positions
    .filter((p) => excluded.has(p.id))
    .reduce((n, p) => n + p.chairRank.length, prefs);
}
