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
  status: "existing" | "added" | "modified";
}

export type DirectoryAppointmentChange =
  | { kind: "add"; appointment: DirectoryAppointment }
  | { kind: "remove"; userId: number; positionId: number }
  | { kind: "changePeriod"; userId: number; positionId: number; from: CommitmentPeriod; to: CommitmentPeriod };

export type CapWindow = "academic-year" | "semester";

export interface CommitteeSeat {
  ccaId: number;
  positionId: number;
  level: "main" | "sub";
}

/** Seat counts for the academic year (`semester: null`) or one semester. */
export interface CommitteeCapCount {
  semester: number | null;
  main: number;
  sub: number;
  /** Distinct Sports and Cultural CCAs. */
  sportsCulture: number;
  exceedsStandard: boolean;
  exceedsAppeal: boolean;
  /** Over the Sports/Culture cap without appeal. */
  exceedsSportsCulture: boolean;
  /** The Sports/Culture limit that applies, given their appeals. */
  sportsCultureLimit: number;
  /** Over the total cap without appeal. */
  exceedsTotal: boolean;
  /** The total limit that applies, given their appeals. */
  totalLimit: number;
}

export interface CommitteeCapEntry {
  userId: number;
  appealed: boolean;
  /** Breaks the committee cap that applies: the appeal cap if appealed, else the standard one. */
  overCap: boolean;
  combinedAppealed: boolean;
  /** Breaks the Sports/Culture or total cap that applies to them. */
  overCombinedCap: boolean;
  counts: CommitteeCapCount[];
  duplicateCcas: number[];
  seats: CommitteeSeat[];
  sportsCultureCcas: number[];
}

export type AppealKind = "committee" | "combined";

/** One staged appeal decision; a batch is saved in one write. */
export interface AppealChange {
  userId: number;
  kind: AppealKind;
  appealed: boolean;
}

/** One allowed committee holding: at most `main` Main and `sub` Sub seats together. */
export interface CommitteeCombo {
  main: number;
  sub: number;
}

/** This year's caps and committee types: the Cap settings page. */
export interface CapRules {
  academicYear: string;
  /** Allowed committee holdings; a holding fits inside any one of them. */
  committeeCombos: CommitteeCombo[];
  committeeCombosAppealed: CommitteeCombo[];
  sportsCultureMax: number;
  sportsCultureMaxAppealed: number;
  totalMax: number;
  /** With the CCA cap appeal. */
  totalMaxAppealed: number;
  /** With both appeals. */
  totalMaxBothAppeals: number;
  /** Type set on the Cap settings page, by CCA ID; overrides the database. */
  ccaTypes: Record<string, "type-a" | "type-b">;
}

export interface CommitteeCapReport {
  window: CapWindow;
  rules: CapRules;
  /** Flagged residents only: over either cap, duplicate seat, or appealed. */
  entries: CommitteeCapEntry[];
  unclassifiedCcas: number[];
}

export interface DirectorySnapshot {
  users: DirectoryUser[];
  ccas: DirectoryCca[];
  positions: DirectoryPosition[];
  appointments: DirectoryAppointment[];
  changes: DirectoryAppointmentChange[];
  committeeCaps: CommitteeCapReport;
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
