// residentCaps.ts — one resident's cap counts for their profile, fetched from
// Rust (which decides every count and flag). Shared by the quota box and the
// CCA caps section so a profile asks once.
import { useEffect, useState } from "react";
import type { CommitteeCapEntry, DirectorySnapshot } from "./types.ts";

export interface ResidentCapsState {
  /** Null when they hold no counted CCA seat. */
  entry: CommitteeCapEntry | null;
  loading: boolean;
}

export function useResidentCaps(
  userId: number,
  directory: DirectorySnapshot,
  loadCaps: (userId: number) => Promise<CommitteeCapEntry | null>,
): ResidentCapsState {
  // The report's copy (flagged residents only) shows at once; the fetch adds
  // counts for everyone else.
  const reported = directory.committeeCaps.entries.find((e) => e.userId === userId) ?? null;
  const [fetched, setFetched] = useState<{ userId: number; entry: CommitteeCapEntry | null } | null>(null);

  // Refetch whenever the directory changes: a member edit or a saved appeal moves the counts.
  useEffect(() => {
    let live = true;
    loadCaps(userId)
      .then((entry) => {
        if (live) setFetched({ userId, entry });
      })
      .catch(() => {
        if (live) setFetched({ userId, entry: null });
      });
    return () => {
      live = false;
    };
  }, [userId, directory, loadCaps]);

  const current = fetched?.userId === userId ? fetched : null;
  return { entry: current ? (current.entry ?? reported) : reported, loading: !current };
}
