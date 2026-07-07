// types.ts — exact mirror of the Rust snapshot JSON
export type Status = "existing" | "allocated" | "appealed" | "displaced" | "quota" | "noreturn" | "neutral";
export type PositionType = "block" | "main" | "sub";

export interface CcaView {
  id: number;
  name: string;
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

export interface AppealView {
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
  appealed: number;
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
  kind: "allocated" | "appealed";
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

export interface Snapshot {
  syncedAt: string;
  warnings: string[];
  ccas: CcaView[];
  positions: PositionView[];
  applicants: ApplicantView[];
  committed: PairView[];
  appeals: AppealView[];
  quota: QuotaView[];
  seats: SeatsView[];
  outcomes: OutcomeView[];
  run: RunView | null;
}

export interface CommitReceipt {
  inserted: number;
  snapshot: Snapshot;
}

export interface PurgeReceipt {
  deleted: number;
  snapshot: Snapshot;
}
