// directory.ts — lookup maps over the DirectorySnapshot, mirroring what
// indexes.ts does for the allocation snapshot. View-only: validation and
// change tracking stay in Rust.
import type {
  CcaKind,
  CommitmentPeriod,
  DirectoryAppointmentView,
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

/** Mirrors `can_manage_appointments`: Rust rejects edits to members and residents, so the UI never offers one. */
export const isEditable = (position: DirectoryPosition) => position.positionType !== "member" && position.positionType !== "resident";

/** `null` capacity is unlimited; `0` is closed. */
export const isFull = (position: DirectoryPosition, held: number) => position.capacity !== null && held >= position.capacity;
export const capacityText = (position: DirectoryPosition, held: number) => `${held}/${position.capacity ?? "∞"}`;

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
  holdersByPosition: Map<number, DirectoryAppointmentView[]>;
  removedByPosition: Map<number, RemovedHolder[]>;
  /** Distinct people holding any position in the CCA. */
  memberCount: (ccaId: number) => number;
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
  const holdersByPosition = new Map<number, DirectoryAppointmentView[]>();
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
  };
}
