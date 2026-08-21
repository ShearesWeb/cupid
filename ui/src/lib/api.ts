import { invoke } from "@tauri-apps/api/core";
import type { ExportReceipt, PurgeReceipt, Snapshot } from "./types";

/// Verify Supabase credentials and store them as the active connection.
/// Resolves to a display label for the target (never contains the password).
export const connect = (
  projectRef: string,
  password: string,
  region: string | null,
): Promise<string> => invoke("connect", { projectRef, password, region });

/// Display label of the active connection, or null when none is configured.
export const connectionInfo = (): Promise<string | null> => invoke("connection_info");

export const sync = (): Promise<Snapshot> => invoke("sync");

export const runMatching = (): Promise<Snapshot> => invoke("run_matching");

/// Probe SSH push access to the intranet repo. Resolves to a confirmation
/// line; rejects with operator guidance when the key or permission is missing.
export const checkAccess = (): Promise<string> => invoke("check_access");

/// Export the run's new allocations. `excluded` holds back whole positions:
/// their seats stay out of the merge request. Pass the same list to `purge`.
export const commit = (excluded: number[]): Promise<ExportReceipt> =>
  invoke("commit", { excluded });

export const archive = (): Promise<{ path: string; rows: number }> => invoke("archive");

/// Delete the cycle's preference rows and preallocations. Positions in
/// `excluded` keep theirs — pass the list the export was given.
export const purge = (excluded: number[]): Promise<PurgeReceipt> =>
  invoke("purge", { excluded });

export const addPreallocation = (
  applicantId: number,
  positionId: number,
  note: string | null,
): Promise<Snapshot> => invoke("add_preallocation", { applicantId, positionId, note });

export const removePreallocation = (applicantId: number, positionId: number): Promise<Snapshot> =>
  invoke("remove_preallocation", { applicantId, positionId });
