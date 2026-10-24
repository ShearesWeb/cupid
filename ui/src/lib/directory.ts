// directory.ts — lookup maps over the DirectorySnapshot, mirroring what
// indexes.ts does for the allocation snapshot. View-only: validation and
// change tracking stay in Rust.
import type {
  CapRules,
  CommitteeCombo,
  CcaKind,
  CommitmentPeriod,
  CommitteeCapCount,
  CommitteeCapEntry,
  DirectoryAppointment,
  DirectoryPosition,
  DirectoryPositionType,
  DirectorySnapshot,
  DirectoryUser,
} from "./types.ts";

/** Position types ordered by significance; decides section order in a CCA. */
export const POSITION_TYPES: [DirectoryPositionType, string][] = [
  ["lead", "Lead"],
  ["vice", "Vice"],
  ["team-manager", "Team manager"],
  ["blockcomm", "Block comm"],
  ["maincomm", "Main comm"],
  ["subcomm", "Sub comm"],
  ["member", "Member"],
  ["resident", "Resident"],
];

export const PERIODS: [CommitmentPeriod, string][] = [
  ["full-year", "Full AY"],
  ["semester-1", "Sem 1"],
  ["semester-2", "Sem 2"],
  ["ex-shearite", "Ex-Shearite"],
];

const KIND_LABELS: Record<CcaKind, string> = {
  sports: "Sports",
  culture: "Culture",
  committee: "Committee",
  jcrc: "JCRC",
  adhoc: "Ad hoc",
  supplementary: "Supplementary",
};

const KIND_ICON: Record<string, string> = {
  sports: "basketball",
  culture: "music",
  committee: "users",
  jcrc: "users",
  adhoc: "star",
};

export function ccaKindIcon(kind: string | undefined): string {
  return (kind && KIND_ICON[kind]) || "hexagon";
}

export const kindLabel = (kind: string) => KIND_LABELS[kind as CcaKind] ?? kind;
export const typeLabel = (type: DirectoryPositionType) => POSITION_TYPES.find(([t]) => t === type)?.[1] ?? type;
export const typeRank = (type: DirectoryPositionType) => POSITION_TYPES.findIndex(([t]) => t === type);
export const periodLabel = (period: CommitmentPeriod) => PERIODS.find(([p]) => p === period)?.[1] ?? period;

/** Residents are read-only: Rust rejects the edit, so the UI never offers one. */
export const isEditable = (position: DirectoryPosition) => position.positionType !== "resident";

/** `null` capacity is unlimited; `0` is closed. */
export const isFull = (position: DirectoryPosition, held: number) => position.capacity !== null && held >= position.capacity;
export const capacityText = (position: DirectoryPosition, held: number) => `${held}/${position.capacity ?? "∞"}`;

/** "2 Main · 1 Sub · 4 Sports/Culture", prefixed with the semester in the per-semester window. */
export function capCountText(count: CommitteeCapCount): string {
  const seats = `${count.main} Main · ${count.sub} Sub · ${count.sportsCulture} Sports/Culture`;
  return count.semester === null ? seats : `Sem ${count.semester}: ${seats}`;
}

export type CapStatus =
  | "over"
  | "over-appeal"
  | "over-combined"
  | "over-combined-appeal"
  | "duplicate"
  | "appeal-used"
  | "appealed"
  | "combined-appeal-used"
  | "combined-appealed";

/** Every status that applies, most serious first. Rust decides each flag; this only names them. */
export function capStatuses(entry: CommitteeCapEntry): CapStatus[] {
  const out: CapStatus[] = [];
  if (entry.overCap) out.push(entry.appealed ? "over-appeal" : "over");
  if (entry.overCombinedCap) out.push(entry.combinedAppealed ? "over-combined-appeal" : "over-combined");
  if (entry.duplicateCcas.length) out.push("duplicate");
  if (entry.appealed && !entry.overCap) out.push(entry.counts.some((c) => c.exceedsStandard) ? "appeal-used" : "appealed");
  if (entry.combinedAppealed && !entry.overCombinedCap) {
    out.push(entry.counts.some((c) => c.exceedsSportsCulture || c.exceedsTotal) ? "combined-appeal-used" : "combined-appealed");
  }
  return out;
}

export const CAP_STATUS: Record<CapStatus, { label: string; tone: "critical" | "warning" | "success" | "neutral"; hint: string }> = {
  over: { label: "Over committee cap", tone: "critical", hint: "Holds more committee seats than this year's committee cap allows." },
  "over-appeal": { label: "Over appeal cap", tone: "critical", hint: "Committee appeal approved, but still over the committee cap with appeal." },
  "over-combined": { label: "Over CCA cap", tone: "critical", hint: "Over this year's Sports/Culture cap or total CCA cap." },
  "over-combined-appeal": {
    label: "Over CCA appeal cap",
    tone: "critical",
    hint: "CCA cap appeal approved, but still over the Sports/Culture or total cap with appeal.",
  },
  duplicate: { label: "Same committee twice", tone: "warning", hint: "Holds two seats in one committee in overlapping periods." },
  "appeal-used": { label: "Committee appeal used", tone: "success", hint: "Over the standard committee cap, within the appeal cap." },
  appealed: { label: "Committee appealed", tone: "neutral", hint: "Committee appeal recorded; currently within the standard cap." },
  "combined-appeal-used": { label: "CCA appeal used", tone: "success", hint: "Over the committee + Sports/Cultural cap, allowed by appeal." },
  "combined-appealed": { label: "CCA appealed", tone: "neutral", hint: "CCA cap appeal recorded; currently within the cap." },
};

/** Labels for the two appeal toggles, keyed like the Rust `AppealKind`. */
export const APPEAL_COPY = {
  committee: {
    label: "Committee appeal approved",
    title: "The JCRC approved an appeal to exceed the committee cap: the committee cap with appeal applies.",
  },
  combined: {
    label: "CCA cap appeal approved",
    title: "The JCRC approved an appeal to exceed the Sports/Culture and total caps: their caps with appeal apply.",
  },
} as const;

/** One combination as the white paper writes it: {main: 1, sub: 2} → "1M2S". */
export function comboText({ main, sub }: CommitteeCombo): string {
  if (main && sub) return `${main}M${sub}S`;
  if (main) return `${main}M`;
  return sub ? `${sub}S` : "none";
}

/** A combination list, e.g. "2M, 1M1S or 3S". */
export function committeeCombos(combos: CommitteeCombo[]): string {
  const texts = combos.map(comboText);
  if (texts.length <= 1) return texts[0] ?? "nothing";
  return `${texts.slice(0, -1).join(", ")} or ${texts[texts.length - 1]}`;
}

/** One line per cap, e.g. for the CCA caps page description. */
export function describeRules(rules: CapRules): { committee: string; sportsCulture: string; total: string } {
  return {
    committee: `${committeeCombos(rules.committeeCombos)} (with appeal: ${committeeCombos(rules.committeeCombosAppealed)})`,
    sportsCulture: `${rules.sportsCultureMax} (with CCA cap appeal: ${rules.sportsCultureMaxAppealed})`,
    total: `${rules.totalMax} (with CCA cap appeal: ${rules.totalMaxAppealed}; with both appeals: ${rules.totalMaxBothAppeals})`,
  };
}

/** A committee's type this year: the Cap settings choice, else the database's. */
export function effectiveType(rules: CapRules, cca: { id: number; type: "none" | "type-a" | "type-b" }): "none" | "type-a" | "type-b" {
  const set: "type-a" | "type-b" | undefined = rules.ccaTypes[String(cca.id)];
  return set ?? cca.type;
}

/** A pending removal: the row has left `appointments` but stays reviewable. */
export interface RemovedHolder {
  userId: number;
  positionId: number;
}

export interface DirectoryIndex {
  userById: Map<number, DirectoryUser>;
  positionById: Map<number, DirectoryPosition>;
  /** Per CCA, sorted by type significance then id. */
  positionsByCca: Map<number, DirectoryPosition[]>;
  holdersByPosition: Map<number, DirectoryAppointment[]>;
  removedByPosition: Map<number, RemovedHolder[]>;
  /** Distinct people holding any position in the CCA. */
  memberCount: (ccaId: number) => number;
  /** Flagged committee-cap entries only. */
  capByUser: Map<number, CommitteeCapEntry>;
  /** This year's caps, for wording warnings and limits. */
  capRules: CapRules;
}

function push<K, V>(map: Map<K, V[]>, key: K, value: V) {
  const list = map.get(key);
  if (list) list.push(value);
  else map.set(key, [value]);
}

export function buildDirectoryIndex(directory: DirectorySnapshot): DirectoryIndex {
  const userById = new Map(directory.users.map((u) => [u.id, u]));
  const positionById = new Map(directory.positions.map((p) => [p.id, p]));
  const positionsByCca = new Map<number, DirectoryPosition[]>();
  for (const p of directory.positions) push(positionsByCca, p.ccaId, p);
  for (const list of positionsByCca.values()) {
    list.sort((a, b) => typeRank(a.positionType) - typeRank(b.positionType) || a.id - b.id);
  }
  const holdersByPosition = new Map<number, DirectoryAppointment[]>();
  for (const a of directory.appointments) push(holdersByPosition, a.positionId, a);
  const removedByPosition = new Map<number, RemovedHolder[]>();
  for (const c of directory.changes) {
    if (c.kind === "remove") push(removedByPosition, c.positionId, { userId: c.userId, positionId: c.positionId });
  }
  const members = new Map<number, Set<number>>();
  for (const a of directory.appointments) {
    const ccaId = positionById.get(a.positionId)?.ccaId;
    if (ccaId === undefined) continue;
    const set = members.get(ccaId) ?? new Set<number>();
    set.add(a.userId);
    members.set(ccaId, set);
  }
  return {
    userById,
    positionById,
    positionsByCca,
    holdersByPosition,
    removedByPosition,
    memberCount: (ccaId) => members.get(ccaId)?.size ?? 0,
    capByUser: new Map(directory.committeeCaps.entries.map((e) => [e.userId, e])),
    capRules: directory.committeeCaps.rules,
  };
}
