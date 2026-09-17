// types.ts — exact mirror of the Rust snapshot JSON
export type Status = "existing" | "allocated" | "preallocated" | "displaced" | "quota" | "noreturn" | "neutral";
export type PositionType = "block" | "main" | "sub";

export type DirectoryPositionType =
  | "lead"
  | "vice"
  | "blockcomm"
  | "maincomm"
  | "subcomm"
  | "team-manager"
  | "member"
  | "resident";

export type CcaKind = "sports" | "committee" | "culture" | "jcrc" | "adhoc" | "supplementary";

export type CommitmentPeriod = "semester-1" | "semester-2" | "full-year" | "ex-shearite";

export interface DirectoryUser {
  id: number;
  name: string;
  email: string;
}

export interface DirectoryCca {
  id: number;
  name: string;
  kind: CcaKind;
  tier: "none" | "tier-1" | "tier-2";
  type: "none" | "type-a" | "type-b";
  description: string | null;
  imageUrl: string | null;
}

export interface DirectoryPosition {
  id: number;
  ccaId: number;
  reportingPositionId: number | null;
  positionType: DirectoryPositionType;
  name: string;
  description: string | null;
  capacity: number | null;
}

export interface DirectoryAppointment {
  userId: number;
  positionId: number;
  commitmentPeriod: CommitmentPeriod;
  points: number;
  teamStatus: "none" | "shortlisted" | "reserve" | "main-team" | "varsity";
  createdAt: string | null;
}

// Only the snapshot's holdings carry a status; a change's own appointment is
// the bare record, so the two must not share one type.
export interface DirectoryAppointmentView extends DirectoryAppointment {
  status: "existing" | "added" | "modified";
}

export type DirectoryAppointmentChange =
  | { kind: "add"; appointment: DirectoryAppointment }
  | { kind: "remove"; userId: number; positionId: number }
  | { kind: "changePeriod"; userId: number; positionId: number; from: CommitmentPeriod; to: CommitmentPeriod };

export interface DirectorySnapshot {
  users: DirectoryUser[];
  ccas: DirectoryCca[];
  positions: DirectoryPosition[];
  appointments: DirectoryAppointmentView[];
  changes: DirectoryAppointmentChange[];
}

export interface CcaView {
  id: number;
  name: string;
  kind: string;
}

export interface PositionView {
  id: number;
  ccaId: number;
  name: string;
  type: PositionType;
  capacity: number;
  chairRank: number[];
}

export interface ApplicantView {
  id: number;
  name: string;
  email: string;
  prefs: number[];
}

export interface PreallocationView {
  applicantId: number;
  positionId: number;
  note: string | null;
}

export interface PairView {
  applicantId: number;
  positionId: number;
}

export interface QuotaView {
  applicantId: number;
  main: number;
  block: number;
  sub: number;
  canAddMain: boolean;
  canAddBlock: boolean;
  canAddSub: boolean;
  over: boolean;
}

export interface SeatView {
  applicantId: number;
  status: Status;
}

export interface SeatsView {
  positionId: number;
  seated: SeatView[];
}

export interface OutcomeView {
  applicantId: number;
  positionId: number;
  status: Status;
  label: string;
  detail: string;
}

export interface AssignmentView {
  applicantId: number;
  positionId: number;
  kind: "allocated" | "preallocated";
  chairRank: number | null;
  prefRank: number | null;
}

export interface EventView {
  applicantId: number;
  positionId: number;
  seq: number;
  kind: "accept" | "reject" | "displace";
  byApplicantId: number | null;
  detail: string;
}

export interface UnfilledView {
  positionId: number;
  open: number;
}

export interface RunView {
  assignments: AssignmentView[];
  events: EventView[];
  unfilled: UnfilledView[];
}

export interface AllocationSnapshot {
  syncedAt: string;
  warnings: string[];
  ccas: CcaView[];
  positions: PositionView[];
  applicants: ApplicantView[];
  committed: PairView[];
  preallocations: PreallocationView[];
  quota: QuotaView[];
  seats: SeatsView[];
  outcomes: OutcomeView[];
  run: RunView | null;
}

/** @deprecated Use AllocationSnapshot. Kept temporarily for existing screens. */
export type Snapshot = AllocationSnapshot;

export interface ExportReceipt {
  rows: number;
  files: string[];
  branch: string;
  prUrl: string;
}

export interface PurgeReceipt {
  deleted: number;
  snapshot: AllocationSnapshot;
}
