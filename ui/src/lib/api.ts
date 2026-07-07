import { invoke } from "@tauri-apps/api/core";
import type { Snapshot } from "./types";

export const sync = (): Promise<Snapshot> => invoke("sync");

export const runMatching = (): Promise<Snapshot> => invoke("run_matching");

export const commit = (): Promise<number> => invoke("commit");

export const archive = (): Promise<{ path: string; rows: number }> => invoke("archive");

export const purge = (): Promise<number> => invoke("purge");
