// Review.tsx — task-16: Review & commit screen.
// Ports reference/cca-console-design.html countPill (594-598), renderReview (599-631),
// commitStepper (632-654), commitCard (655-665). All displayed values are looked up
// from snapshot/idx — this screen never recomputes statuses; "seats filled" reads
// idx.seatsByPos directly (it already merges committed + run seats server-side).
import { useState, type ReactNode } from "react";
import { Button, Card, Icon, StatusPill, TextInput } from "../components/index.ts";
import { statusStyle } from "../components/statusStyle.ts";
import { errorMessage } from "../lib/format.ts";
import * as api from "../lib/api.ts";
import type { Indexes } from "../lib/indexes.ts";
import type { AssignmentView, Snapshot } from "../lib/types.ts";
import type { ToastKind } from "../components/Toasts.tsx";
import { RunPrompt, Section } from "./shared.tsx";

export interface CommitState {
  previewed: boolean;
  committed: boolean;
  archived: boolean;
  purged: boolean;
  committedCount: number;
  archiveRows: number;
}

export interface ReviewProps {
  snapshot: Snapshot;
  idx: Indexes;
  commitState: CommitState;
  purgeText: string;
  onCommitState: (s: CommitState | ((prev: CommitState) => CommitState)) => void;
  onPurgeText: (v: string) => void;
  onOpenMatch: (aid: number, pid: number) => void;
  toast: (kind: ToastKind, text: string) => void;
  running: boolean;
  onRun: () => void;
}

export function Review(props: ReviewProps) {
  const { snapshot, idx, commitState, purgeText, onCommitState, onPurgeText, onOpenMatch, toast, running, onRun } = props;
  const hasRun = snapshot.run !== null;

  if (!hasRun) {
    return (
      <div style={{ padding: "24px 28px 48px", maxWidth: 1120, margin: "0 auto" }}>
        <h1 style={{ margin: "0 0 4px", fontSize: 22, fontWeight: 700, letterSpacing: "-0.4px", color: "var(--token-color-foreground-strong)" }}>
          Review & commit
        </h1>
        <p style={{ margin: "0 0 18px", fontSize: 13, color: "var(--token-color-foreground-faint)" }}>
          Review the run, then commit, archive and purge.
        </p>
        <RunPrompt msg="Run matching first, there's nothing to review yet." running={running} onRun={onRun} />
      </div>
    );
  }

  const adds: AssignmentView[] = commitState.committed ? [] : [...idx.newAllocations, ...idx.appealAllocations];
  const addCount = commitState.committed ? commitState.committedCount : adds.length;
  const totalSeats = snapshot.positions.reduce((s, p) => s + p.capacity, 0);
  let filled = 0;
  idx.seatsByPos.forEach((seated) => {
    filled += seated.length;
  });

  const byCCA = new Map<number, AssignmentView[]>();
  for (const o of adds) {
    const pos = idx.posById.get(o.positionId);
    if (!pos) continue;
    const list = byCCA.get(pos.ccaId);
    if (list) list.push(o);
    else byCCA.set(pos.ccaId, [o]);
  }
  const changedCcas = snapshot.ccas.filter((c) => (byCCA.get(c.id)?.length ?? 0) > 0);

  return (
    <div style={{ padding: "24px 28px 48px", maxWidth: 1120, margin: "0 auto" }}>
      <h1 style={{ margin: "0 0 4px", fontSize: 22, fontWeight: 700, letterSpacing: "-0.4px", color: "var(--token-color-foreground-strong)" }}>
        Review & commit
      </h1>
      <p style={{ margin: "0 0 18px", fontSize: 13, color: "var(--token-color-foreground-faint)", maxWidth: 620 }}>
        These are the new allocations this run will write. Adds-only — existing appointments are never modified or deleted.
      </p>
      <div style={{ display: "flex", gap: 12, marginBottom: 20, flexWrap: "wrap" }}>
        <CountPill label="existing committed" value={snapshot.committed.length} color="var(--token-color-foreground-action)" />
        <CountPill label={commitState.committed ? "committed this run" : "to allocate"} value={`+${addCount}`} color="var(--token-color-foreground-success)" />
        <CountPill label="seats filled" value={`${filled} / ${totalSeats}`} />
      </div>
      <Section title="Changes to write">
        {changedCcas.length ? (
          changedCcas.map((cca) => (
            <DiffGroup key={cca.id} ccaName={cca.name} adds={byCCA.get(cca.id) ?? []} idx={idx} onOpenMatch={onOpenMatch} />
          ))
        ) : (
          <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)", padding: "4px 2px" }}>
            {commitState.committed ? "All additions have been committed." : "This run proposes no new allocations."}
          </div>
        )}
      </Section>
      <Section title="Finalize">
        <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)", marginTop: -4, marginBottom: 12 }}>
          Each step unlocks the next only on success — nothing auto-advances.
        </div>
      </Section>
      <FinalizeStepper
        snapshot={snapshot}
        commitState={commitState}
        purgeText={purgeText}
        onCommitState={onCommitState}
        onPurgeText={onPurgeText}
        toast={toast}
        addCount={addCount}
      />
    </div>
  );
}

// ---- count pill (reference 594-598) ---------------------------------
export function CountPill({ label, value, color }: { label: string; value: string | number; color?: string }) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        padding: "12px 14px",
        borderRadius: 10,
        background: "var(--token-color-surface-faint)",
        border: "1px solid var(--token-color-border-faint)",
        minWidth: 110,
      }}
    >
      <span
        style={{
          fontSize: 24,
          fontWeight: 700,
          fontFamily: "var(--token-typography-font-stack-code)",
          color: color ?? "var(--token-color-foreground-strong)",
          lineHeight: 1,
        }}
      >
        {value}
      </span>
      <span style={{ fontSize: 11.5, color: "var(--token-color-foreground-faint)", marginTop: 5 }}>{label}</span>
    </div>
  );
}

// ---- diff (grouped by CCA) -------------------------------------------
function DiffGroup({
  ccaName,
  adds,
  idx,
  onOpenMatch,
}: {
  ccaName: string;
  adds: AssignmentView[];
  idx: Indexes;
  onOpenMatch: (aid: number, pid: number) => void;
}) {
  return (
    <Card padding="none" style={{ overflow: "hidden", marginBottom: 10 }}>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 9,
          padding: "9px 14px",
          background: "var(--token-color-surface-faint)",
          borderBottom: "1px solid var(--token-color-border-faint)",
          fontFamily: "var(--token-typography-font-stack-code)",
          fontSize: 12.5,
        }}
      >
        <Icon name="folder" size={14} color="var(--token-color-foreground-faint)" />
        <span style={{ fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>{ccaName}</span>
        <span style={{ color: "var(--token-color-foreground-success)", fontWeight: 700 }}>+{adds.length}</span>
      </div>
      {adds.map((o) => (
        <DiffRow key={`${o.applicantId}|${o.positionId}`} o={o} idx={idx} onOpenMatch={onOpenMatch} />
      ))}
    </Card>
  );
}

function DiffRow({
  o,
  idx,
  onOpenMatch,
}: {
  o: AssignmentView;
  idx: Indexes;
  onOpenMatch: (aid: number, pid: number) => void;
}) {
  const appeal = o.kind === "appealed";
  const st = statusStyle(appeal ? "appealed" : "allocated");
  const applicant = idx.appById.get(o.applicantId);
  const pos = idx.posById.get(o.positionId);
  return (
    <button
      onClick={() => onOpenMatch(o.applicantId, o.positionId)}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 10,
        width: "100%",
        padding: "7px 14px",
        border: "none",
        cursor: "pointer",
        textAlign: "left",
        font: "inherit",
        fontFamily: "var(--token-typography-font-stack-code)",
        fontSize: 12.5,
        background: st.bg,
        borderLeft: `3px solid ${st.dot}`,
      }}
    >
      <span style={{ color: st.fg, fontWeight: 700 }}>+</span>
      <span
        style={{
          flex: 1,
          fontWeight: 600,
          color: "var(--token-color-foreground-strong)",
          fontFamily: "var(--token-typography-font-stack-display)",
        }}
      >
        {(applicant?.name ?? "") + "  →  " + (pos?.name ?? "")}
      </span>
      {appeal ? (
        <span style={{ fontSize: 11, fontWeight: 700, color: st.fg, fontFamily: "var(--token-typography-font-stack-display)" }}>
          appeal · exempt
        </span>
      ) : null}
    </button>
  );
}

// ---- commit card frame (reference 655-665) ---------------------------
export function CommitCard({
  n,
  title,
  done,
  locked,
  danger,
  children,
}: {
  n: number;
  title: string;
  done: boolean;
  locked: boolean;
  danger?: boolean;
  children: ReactNode;
}) {
  return (
    <Card padding="none" style={{ overflow: "hidden", marginBottom: 12, opacity: locked ? 0.6 : 1 }}>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 12,
          padding: "13px 16px",
          borderBottom: "1px solid var(--token-color-border-faint)",
          background: danger && !locked && !done ? "var(--token-color-surface-critical)" : "transparent",
        }}
      >
        <div
          style={{
            width: 28,
            height: 28,
            flex: "0 0 28px",
            borderRadius: "50%",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontWeight: 700,
            fontSize: 13,
            background: done ? "var(--token-color-surface-success)" : danger ? "var(--token-color-surface-critical)" : "var(--token-color-surface-strong)",
            color: done ? "var(--token-color-foreground-success)" : danger ? "var(--token-color-foreground-critical-on-surface)" : "var(--token-color-foreground-faint)",
          }}
        >
          {done ? (
            <Icon name="check" size={16} color="var(--token-color-foreground-success)" />
          ) : danger ? (
            <Icon name="alert-triangle" size={15} color="var(--token-color-foreground-critical-on-surface)" />
          ) : (
            n
          )}
        </div>
        <div style={{ flex: 1 }}>
          <div style={{ fontSize: 10, fontWeight: 700, letterSpacing: "0.6px", textTransform: "uppercase", color: "var(--token-color-foreground-faint)" }}>
            Step {n}
          </div>
          <div
            style={{
              fontSize: 15,
              fontWeight: 700,
              color: danger && !locked && !done ? "var(--token-color-foreground-critical-on-surface)" : "var(--token-color-foreground-strong)",
            }}
          >
            {title}
          </div>
        </div>
        {done ? (
          <StatusPill status="allocated" label="Done" />
        ) : locked ? (
          <span style={{ display: "inline-flex", alignItems: "center", gap: 5, fontSize: 12, color: "var(--token-color-foreground-faint)" }}>
            <Icon name="lock" size={13} color="var(--token-color-foreground-faint)" />
            Locked
          </span>
        ) : null}
      </div>
      <div style={{ padding: 16 }}>
        {locked ? (
          <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)" }}>Complete the previous step to unlock.</div>
        ) : (
          children
        )}
      </div>
    </Card>
  );
}

// ---- finalize stepper (reference 632-654) -----------------------------
function FinalizeStepper({
  snapshot,
  commitState,
  purgeText,
  onCommitState,
  onPurgeText,
  toast,
  addCount,
}: {
  snapshot: Snapshot;
  commitState: CommitState;
  purgeText: string;
  onCommitState: ReviewProps["onCommitState"];
  onPurgeText: ReviewProps["onPurgeText"];
  toast: ReviewProps["toast"];
  addCount: number;
}) {
  const [busyCommit, setBusyCommit] = useState(false);
  const [busyArchive, setBusyArchive] = useState(false);
  const [busyPurge, setBusyPurge] = useState(false);
  // Not part of commitState: the file path is purely informational for this card's
  // body and doesn't need to survive navigation away from the Review screen.
  const [archivePath, setArchivePath] = useState<string | null>(null);

  const prefRows = snapshot.applicants.reduce((s, a) => s + a.prefs.length, 0);
  const totalRows = prefRows + snapshot.positions.reduce((s, p) => s + p.chairRank.length, 0);
  const canPurge = purgeText.trim().toUpperCase() === "PURGE";

  const confirmPreview = () => onCommitState((prev) => ({ ...prev, previewed: true }));

  const doCommit = async () => {
    setBusyCommit(true);
    try {
      const n = await api.commit();
      onCommitState((prev) => ({ ...prev, committed: true, committedCount: n }));
      toast("success", `Committed ${n}. Sync to reload the system of record.`);
    } catch (e) {
      toast("error", errorMessage(e));
    } finally {
      setBusyCommit(false);
    }
  };

  const doArchive = async () => {
    setBusyArchive(true);
    try {
      const r = await api.archive();
      setArchivePath(r.path);
      onCommitState((prev) => ({ ...prev, archived: true, archiveRows: r.rows }));
    } catch (e) {
      // A cancelled save dialog is a no-op, not a failure — stay silent.
      if (errorMessage(e) !== "cancelled") toast("error", errorMessage(e));
    } finally {
      setBusyArchive(false);
    }
  };

  const doPurge = async () => {
    setBusyPurge(true);
    try {
      await api.purge();
      onCommitState((prev) => ({ ...prev, purged: true }));
    } catch (e) {
      // Expected until the backend `purge` command is implemented.
      toast("error", errorMessage(e));
    } finally {
      setBusyPurge(false);
    }
  };

  return (
    <>
      <CommitCard n={1} title="Confirm review" done={commitState.previewed} locked={false}>
        {commitState.previewed ? (
          <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)" }}>Reviewed.</div>
        ) : (
          <Button color="primary" onClick={confirmPreview}>
            I've reviewed the changes
          </Button>
        )}
      </CommitCard>

      <CommitCard n={2} title="Commit additions" done={commitState.committed} locked={!commitState.previewed}>
        {commitState.committed ? (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 7,
              fontSize: 13,
              fontWeight: 600,
              color: "var(--token-color-foreground-success-on-surface)",
            }}
          >
            <Icon name="check-circle" size={15} color="var(--token-color-foreground-success)" />
            {commitState.committedCount} additions committed.
          </div>
        ) : (
          <>
            <div style={{ fontSize: 13, color: "var(--token-color-foreground-primary)", marginBottom: 12 }}>
              Writes {addCount} new appointment{addCount === 1 ? "" : "s"}. Existing appointments are untouched.
            </div>
            <Button color="primary" busy={busyCommit} onClick={doCommit}>
              Commit {addCount} addition{addCount === 1 ? "" : "s"}
            </Button>
          </>
        )}
      </CommitCard>

      <CommitCard n={3} title="Archive a verified backup" done={commitState.archived} locked={!commitState.committed} danger>
        <div style={{ fontSize: 13, color: "var(--token-color-foreground-primary)", marginBottom: 12 }}>
          Export a complete backup of all preference & ranking data <strong>before</strong> purging.
        </div>
        <div style={{ display: "flex", gap: 10, marginBottom: 14, flexWrap: "wrap" }}>
          <CountPill label="applicants" value={snapshot.applicants.length} />
          <CountPill label="positions" value={snapshot.positions.length} />
          <CountPill label="preference rows" value={prefRows} />
        </div>
        {commitState.archived ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 7,
                fontSize: 13,
                fontWeight: 600,
                color: "var(--token-color-foreground-success-on-surface)",
              }}
            >
              <Icon name="check-circle" size={15} color="var(--token-color-foreground-success)" />
              Archive downloaded & verified · {commitState.archiveRows} rows.
            </div>
            {archivePath ? (
              <div style={{ fontSize: 11.5, color: "var(--token-color-foreground-faint)", fontFamily: "var(--token-typography-font-stack-code)" }}>
                {archivePath}
              </div>
            ) : null}
          </div>
        ) : (
          <Button color="ghost" icon="download" busy={busyArchive} onClick={doArchive}>
            Download verified archive
          </Button>
        )}
      </CommitCard>

      <CommitCard n={4} title="Purge preference data" done={commitState.purged} locked={!commitState.archived} danger>
        {commitState.purged ? (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 8,
              fontSize: 13.5,
              fontWeight: 700,
              color: "var(--token-color-foreground-critical-on-surface)",
            }}
          >
            <Icon name="check-circle" size={16} color="var(--token-color-foreground-critical-on-surface)" />
            {totalRows} preference rows permanently deleted. This cannot be undone.
          </div>
        ) : (
          <>
            <div
              style={{
                display: "flex",
                gap: 10,
                padding: 12,
                borderRadius: 8,
                background: "var(--token-color-surface-critical)",
                border: "1px solid var(--token-color-border-critical)",
                marginBottom: 14,
              }}
            >
              <Icon name="alert-triangle" size={18} color="var(--token-color-foreground-critical-on-surface)" />
              <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-critical-on-surface)", lineHeight: 1.5 }}>
                <strong>Permanently deletes all {totalRows} preference & ranking rows. </strong>
                Cannot be undone. Make sure your archive downloaded.
              </div>
            </div>
            <div style={{ maxWidth: 280, marginBottom: 14 }}>
              <TextInput label="Type PURGE to confirm" value={purgeText} onChange={(e) => onPurgeText(e.target.value)} placeholder="PURGE" />
            </div>
            <Button color="critical" disabled={!canPurge} busy={busyPurge} onClick={doPurge}>
              Purge preference data
            </Button>
          </>
        )}
      </CommitCard>
    </>
  );
}
