// Review.tsx — task-16: Review & commit screen.
// Ports reference/cca-console-design.html countPill (594-598), renderReview (599-631),
// commitStepper (632-654), commitCard (655-665). All displayed values are looked up
// from snapshot/idx — this screen never recomputes statuses; "seats filled" reads
// idx.seatsByPos directly (it already merges committed + run seats server-side).
import { useState, type ReactNode } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Button, Card, Icon, StatusPill, TextInput } from "../components/index.ts";
import { statusStyle } from "../components/statusStyle.ts";
import { errorMessage } from "../lib/format.ts";
import * as api from "../lib/api.ts";
import type { Indexes } from "../lib/indexes.ts";
import { groupAdds, heldBackRows, includedAdds, type CcaGroup } from "../lib/selection.ts";
import type { AssignmentView, Snapshot } from "../lib/types.ts";
import type { ToastKind } from "../components/Toasts.tsx";
import { RunPrompt, Section } from "./shared.tsx";

export interface CommitState {
  previewed: boolean;
  /// Positions the operator held back this cycle. Scopes both the export and
  /// the purge, so a role that is not finalised keeps its preference data.
  excluded: number[];
  accessChecked: boolean;
  exported: boolean;
  archived: boolean;
  purged: boolean;
  exportedRows: number;
  branch: string | null;
  prUrl: string | null;
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
  onApplySnapshot: (snap: Snapshot) => void;
}

export function Review(props: ReviewProps) {
  const { snapshot, idx, commitState, purgeText, onCommitState, onPurgeText, onOpenMatch, toast, running, onRun, onApplySnapshot } = props;
  const hasRun = snapshot.run !== null;

  // After an export the stepper must stay visible even if the corpus is
  // resynced (run: null), so archive and purge can follow.
  if (!hasRun && !commitState.exported) {
    return (
      <div style={{ padding: "24px 28px 48px", maxWidth: 1120, margin: "0 auto" }}>
        <h1 style={{ margin: "0 0 4px", fontSize: 22, fontWeight: 700, letterSpacing: "-0.4px", color: "var(--token-color-foreground-strong)" }}>
          Review & commit
        </h1>
        <p style={{ margin: "0 0 18px", fontSize: 13, color: "var(--token-color-foreground-faint)" }}>
          Review the run, then export the merge request, archive and purge.
        </p>
        <RunPrompt msg="Run matching first, there's nothing to review yet." running={running} onRun={onRun} />
      </div>
    );
  }

  const adds: AssignmentView[] = commitState.exported ? [] : [...idx.newAllocations, ...idx.preallocatedAllocations];
  const excluded = new Set(commitState.excluded);
  const addCount = commitState.exported ? commitState.exportedRows : includedAdds(adds, excluded).length;
  const totalSeats = snapshot.positions.reduce((s, p) => s + p.capacity, 0);
  let filled = 0;
  idx.seatsByPos.forEach((seated) => {
    filled += seated.length;
  });

  const groups = groupAdds(adds, snapshot);
  const heldBackCount = adds.length - addCount;

  // The export and the purge must agree on what was held back, so the
  // checklist freezes the moment the branch is pushed.
  const togglePositions = (ids: number[], include: boolean) =>
    onCommitState((prev) => {
      const next = new Set(prev.excluded);
      for (const id of ids) {
        if (include) next.delete(id);
        else next.add(id);
      }
      return { ...prev, excluded: [...next] };
    });

  return (
    <div style={{ padding: "24px 28px 48px", maxWidth: 1120, margin: "0 auto" }}>
      <h1 style={{ margin: "0 0 4px", fontSize: 22, fontWeight: 700, letterSpacing: "-0.4px", color: "var(--token-color-foreground-strong)" }}>
        Review & commit
      </h1>
      <p style={{ margin: "0 0 18px", fontSize: 13, color: "var(--token-color-foreground-faint)", maxWidth: 620 }}>
        These are the new allocations this run will export. Adds-only — existing appointments are never modified or deleted, and
        nothing is written to the database: the export pushes CSV files as a merge request to the intranet repo.
      </p>
      <div style={{ display: "flex", gap: 12, marginBottom: 20, flexWrap: "wrap" }}>
        <CountPill label="existing committed" value={snapshot.committed.length} color="var(--token-color-foreground-action)" />
        <CountPill label={commitState.exported ? "exported this run" : "to allocate"} value={`+${addCount}`} color="var(--token-color-foreground-success)" />
        <CountPill label="seats filled" value={`${filled} / ${totalSeats}`} />
      </div>
      <Section title="Changes to export">
        {groups.length ? (
          <>
            <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)", marginTop: -4, marginBottom: 10 }}>
              Untick a position to hold it back: its seats stay out of the merge request and its preference data survives
              the purge, ready for a later cycle.
              {heldBackCount > 0 ? (
                <strong style={{ color: "var(--token-color-foreground-warning-on-surface)" }}>
                  {" "}
                  {heldBackCount} addition{heldBackCount === 1 ? "" : "s"} held back.
                </strong>
              ) : null}
            </div>
            {groups.map((group) => (
              <DiffGroup
                key={group.ccaId}
                group={group}
                idx={idx}
                excluded={excluded}
                locked={commitState.exported}
                onToggle={togglePositions}
                onOpenMatch={onOpenMatch}
              />
            ))}
          </>
        ) : (
          <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)", padding: "4px 2px" }}>
            {commitState.exported ? "All additions have been exported." : "This run proposes no new allocations."}
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
        onApplySnapshot={onApplySnapshot}
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

// ---- include/exclude checkbox ----------------------------------------
// Tri-state: a CCA whose positions are only partly held back shows a bar
// rather than a tick. No native input — `indeterminate` is a DOM property
// with no JSX attribute, and the visual is two tokens either way.
function Checkbox({
  state,
  disabled,
  label,
  onClick,
}: {
  state: "on" | "off" | "mixed";
  disabled: boolean;
  label: string;
  onClick: () => void;
}) {
  const filled = state !== "off";
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={state === "mixed" ? "mixed" : state === "on"}
      aria-label={label}
      disabled={disabled}
      onClick={onClick}
      style={{
        width: 15,
        height: 15,
        flex: "0 0 15px",
        padding: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        borderRadius: 4,
        border: `1px solid ${filled ? "var(--token-color-border-action)" : "var(--token-color-border-strong)"}`,
        background: filled ? "var(--token-color-foreground-action)" : "var(--token-color-surface-primary)",
        opacity: disabled ? 0.5 : 1,
        cursor: disabled ? "default" : "pointer",
      }}
    >
      {state === "on" ? <Icon name="check" size={11} color="#fff" /> : null}
      {state === "mixed" ? <span style={{ width: 7, height: 2, borderRadius: 1, background: "#fff" }} /> : null}
    </button>
  );
}

// ---- diff (grouped by CCA, then position) ----------------------------
function DiffGroup({
  group,
  idx,
  excluded,
  locked,
  onToggle,
  onOpenMatch,
}: {
  group: CcaGroup;
  idx: Indexes;
  excluded: Set<number>;
  locked: boolean;
  onToggle: (ids: number[], include: boolean) => void;
  onOpenMatch: (aid: number, pid: number) => void;
}) {
  const ids = group.positions.map((p) => p.positionId);
  const includedIds = ids.filter((id) => !excluded.has(id));
  const state = includedIds.length === 0 ? "off" : includedIds.length === ids.length ? "on" : "mixed";
  const includedCount = group.positions
    .filter((p) => !excluded.has(p.positionId))
    .reduce((n, p) => n + p.adds.length, 0);
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
        <Checkbox
          state={state}
          disabled={locked}
          label={`Include every ${group.name} position`}
          onClick={() => onToggle(ids, state !== "on")}
        />
        <Icon name="folder" size={14} color="var(--token-color-foreground-faint)" />
        <span style={{ fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>{group.name}</span>
        <span style={{ color: "var(--token-color-foreground-success)", fontWeight: 700 }}>+{includedCount}</span>
        {includedCount < group.addCount ? (
          <span style={{ color: "var(--token-color-foreground-faint)" }}>of +{group.addCount}</span>
        ) : null}
      </div>
      {group.positions.map((position) => (
        <DiffPosition
          key={position.positionId}
          position={position}
          idx={idx}
          held={excluded.has(position.positionId)}
          locked={locked}
          onToggle={onToggle}
          onOpenMatch={onOpenMatch}
        />
      ))}
    </Card>
  );
}

function DiffPosition({
  position,
  idx,
  held,
  locked,
  onToggle,
  onOpenMatch,
}: {
  position: CcaGroup["positions"][number];
  idx: Indexes;
  held: boolean;
  locked: boolean;
  onToggle: (ids: number[], include: boolean) => void;
  onOpenMatch: (aid: number, pid: number) => void;
}) {
  return (
    <>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 9,
          padding: "6px 14px 6px 28px",
          borderBottom: "1px solid var(--token-color-border-faint)",
          fontFamily: "var(--token-typography-font-stack-code)",
          fontSize: 12,
          color: "var(--token-color-foreground-faint)",
        }}
      >
        <Checkbox
          state={held ? "off" : "on"}
          disabled={locked}
          label={`Include ${position.name}`}
          onClick={() => onToggle([position.positionId], held)}
        />
        <span
          style={{
            fontWeight: 600,
            color: held ? "var(--token-color-foreground-faint)" : "var(--token-color-foreground-primary)",
            textDecoration: held ? "line-through" : "none",
          }}
        >
          {position.name}
        </span>
        <span style={{ color: held ? "var(--token-color-foreground-faint)" : "var(--token-color-foreground-success)", fontWeight: 700 }}>
          +{position.adds.length}
        </span>
        {held ? <span style={{ color: "var(--token-color-foreground-warning-on-surface)" }}>held back</span> : null}
      </div>
      {position.adds.map((o) => (
        <DiffRow key={`${o.applicantId}|${o.positionId}`} o={o} idx={idx} held={held} onOpenMatch={onOpenMatch} />
      ))}
    </>
  );
}

function DiffRow({
  o,
  idx,
  held,
  onOpenMatch,
}: {
  o: AssignmentView;
  idx: Indexes;
  held: boolean;
  onOpenMatch: (aid: number, pid: number) => void;
}) {
  const preallocated = o.kind === "preallocated";
  const st = statusStyle(held ? "neutral" : preallocated ? "preallocated" : "allocated");
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
        opacity: held ? 0.55 : 1,
      }}
    >
      <span style={{ color: st.fg, fontWeight: 700 }}>{held ? "\u2013" : "+"}</span>
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
      {preallocated ? (
        <span style={{ fontSize: 11, fontWeight: 700, color: st.fg, fontFamily: "var(--token-typography-font-stack-display)" }}>
          preallocated
        </span>
      ) : null}
    </button>
  );
}

// ---- merge request link ----------------------------------------------
// The export's product: presented explicitly, never auto-opened. Opens in
// the system browser; the URL is also selectable and copyable.
function MergeRequestLink({ url, toast }: { url: string; toast: ReviewProps["toast"] }) {
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(url);
      toast("success", "Merge request URL copied.");
    } catch {
      toast("error", "Could not copy the URL — select it manually.");
    }
  };
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 10,
        padding: "10px 12px",
        borderRadius: 8,
        background: "var(--token-color-surface-faint)",
        border: "1px solid var(--token-color-border-faint)",
        flexWrap: "wrap",
      }}
    >
      <span
        style={{
          flex: 1,
          minWidth: 220,
          fontSize: 12,
          fontFamily: "var(--token-typography-font-stack-code)",
          color: "var(--token-color-foreground-strong)",
          wordBreak: "break-all",
          WebkitUserSelect: "all",
          userSelect: "all",
        }}
      >
        {url}
      </span>
      <Button color="primary" icon="external-link" onClick={() => void openUrl(url)}>
        Open merge request
      </Button>
      <Button color="ghost" icon="copy" onClick={() => void copy()}>
        Copy
      </Button>
    </div>
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
  onApplySnapshot,
  onPurgeText,
  toast,
  addCount,
}: {
  snapshot: Snapshot;
  commitState: CommitState;
  purgeText: string;
  onCommitState: ReviewProps["onCommitState"];
  onApplySnapshot: ReviewProps["onApplySnapshot"];
  onPurgeText: ReviewProps["onPurgeText"];
  toast: ReviewProps["toast"];
  addCount: number;
}) {
  const [busyCheck, setBusyCheck] = useState(false);
  const [busyExport, setBusyExport] = useState(false);
  const [busyArchive, setBusyArchive] = useState(false);
  const [busyPurge, setBusyPurge] = useState(false);
  // Not part of commitState: the file path is purely informational for this card's
  // body and doesn't need to survive navigation away from the Review screen.
  const [archivePath, setArchivePath] = useState<string | null>(null);

  const prefRows = snapshot.applicants.reduce((s, a) => s + a.prefs.length, 0);
  const totalRows = prefRows + snapshot.positions.reduce((s, p) => s + p.chairRank.length, 0);
  const heldRows = heldBackRows(snapshot, new Set(commitState.excluded));
  const purgeRows = totalRows - heldRows;
  const canPurge = purgeText.trim().toUpperCase() === "PURGE";

  const confirmPreview = () => onCommitState((prev) => ({ ...prev, previewed: true }));

  const doCheckAccess = async () => {
    setBusyCheck(true);
    try {
      const message = await api.checkAccess();
      onCommitState((prev) => ({ ...prev, accessChecked: true }));
      toast("success", message);
    } catch (e) {
      toast("error", errorMessage(e));
    } finally {
      setBusyCheck(false);
    }
  };

  const doExport = async () => {
    setBusyExport(true);
    try {
      const receipt = await api.commit(commitState.excluded);
      onCommitState((prev) => ({
        ...prev,
        exported: true,
        exportedRows: receipt.rows,
        branch: receipt.branch,
        prUrl: receipt.prUrl,
      }));
      toast("success", `Pushed ${receipt.rows} appointments on ${receipt.branch}.`);
    } catch (e) {
      toast("error", errorMessage(e));
    } finally {
      setBusyExport(false);
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
      const receipt = await api.purge(commitState.excluded);
      onApplySnapshot(receipt.snapshot);
      onCommitState((prev) => ({ ...prev, purged: true }));
      toast("success", `Purged ${receipt.deleted} rows.`);
    } catch (e) {
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

      <CommitCard n={2} title="Export & open merge request" done={commitState.exported} locked={!commitState.previewed}>
        {commitState.exported ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
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
              {commitState.exportedRows} additions pushed on {commitState.branch}.
            </div>
            {commitState.prUrl ? <MergeRequestLink url={commitState.prUrl} toast={toast} /> : null}
            <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)" }}>
              Open the merge request and get it merged — intranet's CI writes the appointments. They appear here after the
              next sync.
            </div>
          </div>
        ) : (
          <>
            <div style={{ fontSize: 13, color: "var(--token-color-foreground-primary)", marginBottom: 12 }}>
              Exports {addCount} new appointment{addCount === 1 ? "" : "s"} as CSV and pushes a branch to the intranet repo.
              Nothing is written to the database.
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 12, flexWrap: "wrap" }}>
              {commitState.accessChecked ? (
                <span
                  style={{
                    display: "inline-flex",
                    alignItems: "center",
                    gap: 7,
                    fontSize: 13,
                    fontWeight: 600,
                    color: "var(--token-color-foreground-success-on-surface)",
                  }}
                >
                  <Icon name="check-circle" size={15} color="var(--token-color-foreground-success)" />
                  Push access verified.
                </span>
              ) : (
                <>
                  <Button color="ghost" icon="lock" busy={busyCheck} onClick={doCheckAccess}>
                    Check SSH access
                  </Button>
                  <span style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)" }}>
                    Verifies this machine's SSH key can push to the intranet repo. Nothing is cloned or pushed yet.
                  </span>
                </>
              )}
            </div>
            <Button color="primary" busy={busyExport} disabled={!commitState.accessChecked} onClick={doExport}>
              Export {addCount} addition{addCount === 1 ? "" : "s"}
            </Button>
          </>
        )}
      </CommitCard>

      <CommitCard n={3} title="Archive a verified backup" done={commitState.archived} locked={!commitState.exported} danger>
        <div style={{ fontSize: 13, color: "var(--token-color-foreground-primary)", marginBottom: 12 }}>
          Export a complete backup of all preference, ranking and preallocation data <strong>before</strong> purging.
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
            {purgeRows} preference rows permanently deleted. This cannot be undone.
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
                <strong>
                  Permanently deletes {purgeRows} of {totalRows} preference & ranking rows from the database, plus the
                  local preallocations that go with them.{" "}
                </strong>
                Cannot be undone. Make sure your archive downloaded and the merge request is on its way.
                {heldRows > 0 ? (
                  <>
                    {" "}
                    The {heldRows} rows on held-back positions stay in place, along with their preallocations.
                  </>
                ) : null}
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
