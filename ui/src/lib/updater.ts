// updater.ts — one stable channel; the release endpoint lives in tauri.conf.json.
import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

// Dev builds carry the unbumped tauri.conf version, so every published release
// looks newer. Skip the check there rather than nag on every `tauri dev`.
export const updatesSupported = !import.meta.env.DEV;

export interface PendingUpdate {
  version: string;
  current: string;
  notes: string | null;
  handle: Update;
}

export function appVersion(): Promise<string> {
  return getVersion();
}

export async function checkForUpdate(): Promise<PendingUpdate | null> {
  if (!updatesSupported) return null;
  const update = await check();
  if (!update) return null;
  return {
    version: update.version,
    current: update.currentVersion,
    notes: update.body ?? null,
    handle: update,
  };
}

// Downloads, installs, then restarts into the new build. The Windows installer
// replaces the running binary, so relaunch() may never return there.
export async function installUpdate(pending: PendingUpdate): Promise<void> {
  await pending.handle.downloadAndInstall();
  await relaunch();
}

// Releases the download handle; failure is inert (the modal is closing anyway).
export async function dismissUpdate(pending: PendingUpdate): Promise<void> {
  try {
    await pending.handle.close();
  } catch {
    /* nothing to release */
  }
}
