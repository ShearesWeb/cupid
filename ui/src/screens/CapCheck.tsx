// CapCheck.tsx — read-only report of residents flagged by the white paper's
// two caps: committee combinations, and committee seats + Sports/Cultural
// CCAs. Appeals are approved or withdrawn on the resident's profile; clicking
// a name here opens it, and clicking a seat opens its Edit member window. Rust decides
// every flag (directory::caps); this screen only lists them.
import { useMemo, useState, type ReactNode } from "react";
import { Avatar, CapTags, Card, Icon, TextInput } from "../components/index.ts";
import { buildDirectoryIndex, capCountText, capStatuses, describeRules, isEditable, kindLabel, type CapStatus } from "../lib/directory.ts";
import type { CapWindow, CommitteeCapEntry, DirectorySnapshot } from "../lib/types.ts";
import type { MemberRef } from "./Ccas.tsx";
import { ChipRow, EmptyState, PageTitle } from "./shared.tsx";

export interface CapCheckProps {
  directory: DirectorySnapshot;
  onSetWindow: (window: CapWindow) => Promise<boolean>;
  /** Opens the seat's CCA with that person's Edit member window. */
  onOpenMember: (member: MemberRef) => void;
  /** Opens the resident's profile, where cap appeals are approved or withdrawn. */
  onOpenProfile: (userId: number) => void;
}

type Filter = "all" | "committee" | "combined" | "duplicate" | "appealed";

const RANK: Record<CapStatus, number> = {
  over: 0,
  "over-appeal": 0,
  "over-combined": 0,
  "over-combined-appeal": 0,
  duplicate: 1,
  "appeal-used": 2,
  "combined-appeal-used": 2,
  appealed: 3,
  "combined-appealed": 3,
};
const rank = (e: CommitteeCapEntry) => Math.min(...capStatuses(e).map((s) => RANK[s]));

function matchesFilter(e: CommitteeCapEntry, filter: Filter) {
  switch (filter) {
    case "all":
      return true;
    case "committee":
      return e.overCap;
    case "combined":
      return e.overCombinedCap;
    case "duplicate":
      return e.duplicateCcas.length > 0;
    case "appealed":
      return e.appealed || e.combinedAppealed;
  }
}

export function CapCheck({ directory, onSetWindow, onOpenMember, onOpenProfile }: CapCheckProps) {
  const dx = useMemo(() => buildDirectoryIndex(directory), [directory]);
  const ccaById = useMemo(() => new Map(directory.ccas.map((c) => [c.id, c])), [directory]);
  // A Sports/Cultural chip names only the CCA; this finds the person's seat in it.
  const seatIn = useMemo(() => {
    const map = new Map<string, number>();
    for (const a of directory.appointments) {
      const position = dx.positionById.get(a.positionId);
      // Resident seats are read-only, so they have no Edit member window.
      if (!position || !isEditable(position)) continue;
      const key = `${a.userId}:${position.ccaId}`;
      if (!map.has(key)) map.set(key, a.positionId);
    }
    return (userId: number, ccaId: number) => map.get(`${userId}:${ccaId}`);
  }, [directory, dx]);
  const [filter, setFilter] = useState<Filter>("all");
  const [search, setSearch] = useState("");
  const { entries, window, unclassifiedCcas, rules } = directory.committeeCaps;
  const described = describeRules(rules);

  const nameOf = (id: number) => dx.userById.get(id)?.name ?? `User ${id}`;
  const q = search.trim().toLowerCase();
  const list = entries
    .filter((e) => matchesFilter(e, filter))
    .filter((e) => {
      const u = dx.userById.get(e.userId);
      return !q || (u ? u.name.toLowerCase().includes(q) || u.email.toLowerCase().includes(q) : false);
    })
    .sort((a, b) => rank(a) - rank(b) || nameOf(a.userId).localeCompare(nameOf(b.userId)));
  const count = (f: Filter) => entries.filter((e) => matchesFilter(e, f)).length;

  return (
    <div style={{ padding: "24px 28px 48px", maxWidth: 1040, margin: "0 auto" }}>
      <PageTitle
        title="CCA caps"
        sub={`${rules.academicYear ? `${rules.academicYear} caps. ` : ""}Committee: ${described.committee}. Sports/Culture: ${described.sportsCulture}. Total CCAs: ${described.total}. Change them on the Cap settings page. Flags never block an edit.`}
      />
      <div style={{ display: "flex", flexDirection: "column", gap: 10, marginBottom: 18, paddingBottom: 16, borderBottom: "1px solid var(--token-color-border-faint)" }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 16, flexWrap: "wrap" }}>
          <ChipRow<Filter>
            label="Show"
            options={[
              ["all", `All · ${entries.length}`],
              ["committee", `Over committee cap · ${count("committee")}`],
              ["combined", `Over CCA cap · ${count("combined")}`],
              ["duplicate", `Same committee · ${count("duplicate")}`],
              ["appealed", `Appealed · ${count("appealed")}`],
            ]}
            value={filter}
            onChange={setFilter}
          />
          <div style={{ width: 240 }}>
            <TextInput
              icon={<Icon name="search" size={16} color="var(--token-color-foreground-faint)" />}
              placeholder="Search residents…"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
        </div>
        <ChipRow<CapWindow>
          label="Count"
          options={[
            ["academic-year", "Whole academic year"],
            ["semester", "Each semester"],
          ]}
          value={window}
          onChange={(w) => void onSetWindow(w)}
        />
      </div>
      <Notice tone="info">To approve or withdraw an appeal, click the resident's name to open their profile.</Notice>
      {unclassifiedCcas.length ? (
        <Notice tone="warning">
          No Type A/B set (in the database or on the Cap settings page) for {unclassifiedCcas.map((id) => ccaById.get(id)?.name ?? `CCA ${id}`).join(", ")}. Their Sub-Comm seats count as Sub and the rest
          as Main until the type is set.
        </Notice>
      ) : null}
      {list.length === 0 ? (
        <EmptyState
          icon="check-circle"
          title={entries.length ? "No matching residents" : "Everyone is within the caps"}
          sub={entries.length ? "Try a different filter or name." : "No resident holds more seats than the white paper allows."}
        />
      ) : (
        <Card padding="none" style={{ padding: 0, overflow: "hidden" }}>
          {list.map((e, i) => {
            return (
              <CapRow
                key={e.userId}
                entry={e}
                first={i === 0}
                name={nameOf(e.userId)}
                email={dx.userById.get(e.userId)?.email ?? ""}
                ccaName={(id) => ccaById.get(id)?.name ?? `CCA ${id}`}
                ccaKind={(id) => kindLabel(ccaById.get(id)?.kind ?? "")}
                positionName={(id) => dx.positionById.get(id)?.name ?? `Position ${id}`}
                onOpenPerson={() => onOpenProfile(e.userId)}
                onOpenSeat={(positionId) => onOpenMember({ userId: e.userId, positionId })}
                seatIn={(ccaId) => seatIn(e.userId, ccaId)}
              />
            );
          })}
        </Card>
      )}
    </div>
  );
}

function Notice({ tone, children }: { tone: "info" | "warning"; children: ReactNode }) {
  const warning = tone === "warning";
  return (
    <div
      role="status"
      style={{
        display: "flex",
        gap: 10,
        padding: "10px 12px",
        marginBottom: 16,
        borderRadius: 8,
        fontSize: 12.5,
        background: warning ? "var(--token-color-surface-warning)" : "var(--token-color-surface-faint)",
        border: `1px solid ${warning ? "var(--token-color-border-warning)" : "var(--token-color-border-faint)"}`,
        color: "var(--token-color-foreground-strong)",
      }}
    >
      <span style={{ display: "flex", paddingTop: 2 }}>
        <Icon
          name={warning ? "alert-triangle" : "info"}
          size={14}
          color={warning ? "var(--token-color-foreground-warning-on-surface)" : "var(--token-color-foreground-faint)"}
        />
      </span>
      <span>{children}</span>
    </div>
  );
}

function CapRow({
  entry,
  first,
  name,
  email,
  ccaName,
  ccaKind,
  positionName,
  onOpenPerson,
  onOpenSeat,
  seatIn,
}: {
  entry: CommitteeCapEntry;
  first: boolean;
  name: string;
  email: string;
  ccaName: (id: number) => string;
  ccaKind: (id: number) => string;
  positionName: (id: number) => string;
  onOpenPerson: () => void;
  onOpenSeat: (positionId: number) => void;
  seatIn: (ccaId: number) => number | undefined;
}) {
  const duplicate = new Set(entry.duplicateCcas);
  return (
    <div style={{ display: "flex", gap: 12, padding: "12px 16px", borderTop: first ? "none" : "1px solid var(--token-color-border-faint)" }}>
      <Avatar name={name} size="small" />
      <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 7 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
          <button
            onClick={onOpenPerson}
            title={`Open ${name}'s profile`}
            style={{
              border: "none",
              background: "transparent",
              padding: 0,
              font: "inherit",
              fontSize: 13,
              fontWeight: 600,
              cursor: "pointer",
              color: "var(--token-color-foreground-action)",
            }}
          >
            {name}
          </button>
          <span style={{ fontSize: 11.5, color: "var(--token-color-foreground-faint)" }}>{email}</span>
          <CapTags entry={entry} />
        </div>
        <div style={{ fontSize: 12, fontWeight: 600, color: "var(--token-color-foreground-strong)", fontFamily: "var(--token-typography-font-stack-code)" }}>
          {entry.counts.map(capCountText).join("   ")}
        </div>
        {entry.seats.length || entry.sportsCultureCcas.length ? (
          <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
            {entry.seats.map((s) => (
              <SeatChip
                key={`seat-${s.positionId}`}
                mark={s.level === "main" ? "M" : "S"}
                strong={s.level === "main"}
                cca={ccaName(s.ccaId)}
                detail={positionName(s.positionId)}
                warn={duplicate.has(s.ccaId)}
                onClick={() => onOpenSeat(s.positionId)}
              />
            ))}
            {entry.sportsCultureCcas.map((id) => {
              const positionId = seatIn(id);
              return (
                <SeatChip
                  key={`sc-${id}`}
                  mark="SC"
                  cca={ccaName(id)}
                  detail={ccaKind(id)}
                  onClick={positionId === undefined ? undefined : () => onOpenSeat(positionId)}
                />
              );
            })}
          </div>
        ) : (
          <span style={{ fontSize: 11.5, color: "var(--token-color-foreground-faint)" }}>No counted seats.</span>
        )}
      </div>
    </div>
  );
}

function SeatChip({
  mark,
  strong,
  cca,
  detail,
  warn,
  onClick,
}: {
  mark: string;
  strong?: boolean;
  cca: string;
  detail: string;
  warn?: boolean;
  onClick: (() => void) | undefined;
}) {
  return (
    <button
      onClick={onClick}
      disabled={!onClick}
      title={onClick ? `Edit this seat in ${cca}` : undefined}
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 6,
        height: 24,
        padding: "0 8px",
        borderRadius: 6,
        font: "inherit",
        fontSize: 11.5,
        cursor: onClick ? "pointer" : "default",
        background: "var(--token-color-surface-faint)",
        border: `1px solid ${warn ? "var(--token-color-border-warning)" : "var(--token-color-border-faint)"}`,
        color: "var(--token-color-foreground-strong)",
      }}
    >
      <b style={{ fontSize: 10, color: strong ? "var(--cupid-strong)" : "var(--token-color-foreground-faint)" }}>{mark}</b>
      <span style={{ fontWeight: 600 }}>{cca}</span>
      <span style={{ color: "var(--token-color-foreground-faint)" }}>{detail}</span>
    </button>
  );
}
