// Allocations.tsx — task-13: position/applicant allocation views.
// Ports reference/cca-console-design.html: renderAllocations (319-328), positionView (330-340),
// positionCard/capacityBar (393-424), applicantView/applicantRow (426-454).
// All displayed values are looked up from snapshot/idx — this screen never recomputes statuses;
// the Rust backend already orders seated lists existing -> allocated -> preallocated.
import { Card, Avatar, ChairCoverage, ChoiceCoverage, Icon, PositionTypeBadge, TextInput } from "../components/index.ts";
import { statusStyle } from "../components/statusStyle.ts";
import { pk, type Indexes } from "../lib/indexes.ts";
import type { ApplicantView, PositionType, PositionView, SeatView, Snapshot, Status } from "../lib/types.ts";
import { EmptyState, Legend, Pager } from "./shared.tsx";

type View = "position" | "applicant";
type TypeFilter = "all" | PositionType;

const TYPE_CHIPS: [TypeFilter, string][] = [
  ["all", "All"],
  ["main", "Main"],
  ["block", "Block"],
  ["sub", "Sub"],
];

export interface AllocationsProps {
  snapshot: Snapshot;
  idx: Indexes;
  view: View;
  search: string;
  typeFilter: TypeFilter;
  page: number;
  onSetView: (v: View) => void;
  onSetSearch: (v: string) => void;
  onSetTypeFilter: (v: TypeFilter) => void;
  onSetPage: (p: number) => void;
  onOpenDetail: (type: "applicant" | "position", id: number) => void;
  onOpenMatch: (aid: number, pid: number) => void;
  hasRun: boolean;
  running: boolean;
  onRun: () => void;
}

// `running`/`onRun` are accepted to match the shell's produced-props contract, but this screen
// has no run-prompt state of its own (position/applicant cards render existing seats even
// pre-run) — they're unused here on purpose.
export function Allocations(props: AllocationsProps) {
  const { snapshot, idx, view, search, typeFilter, page, onSetView, onSetSearch, onSetTypeFilter, onSetPage, onOpenDetail, onOpenMatch, hasRun } = props;
  return (
    <div style={{ padding: "24px 28px 48px", maxWidth: 1120, margin: "0 auto" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 16, marginBottom: 12, flexWrap: "wrap" }}>
        <SegmentToggle view={view} onSetView={onSetView} />
        <div style={{ width: 260 }}>
          <TextInput
            icon={<Icon name="search" size={16} color="var(--token-color-foreground-faint)" />}
            placeholder={view === "position" ? "Search CCA or position…" : "Search applicant name…"}
            value={search}
            onChange={(e) => onSetSearch(e.target.value)}
          />
        </div>
      </div>
      {view === "position" ? (
        <div style={{ display: "flex", flexWrap: "wrap", gap: 5, marginBottom: 12 }}>
          {TYPE_CHIPS.map(([value, label]) => (
            <TypeChip key={value} label={label} active={typeFilter === value} onClick={() => onSetTypeFilter(value)} />
          ))}
        </div>
      ) : null}
      <div style={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: "6px 16px", marginBottom: 16 }}>
        <Legend />
      </div>
      {view === "position" ? (
        <PositionView snapshot={snapshot} idx={idx} search={search} typeFilter={typeFilter} page={page} onSetPage={onSetPage} onOpenDetail={onOpenDetail} />
      ) : (
        <ApplicantView snapshot={snapshot} idx={idx} search={search} page={page} hasRun={hasRun} onSetPage={onSetPage} onOpenDetail={onOpenDetail} onOpenMatch={onOpenMatch} />
      )}
    </div>
  );
}

function SegmentToggle({ view, onSetView }: { view: View; onSetView: (v: View) => void }) {
  const seg = (v: View, label: string, icon: string) => (
    <button
      key={v}
      onClick={() => onSetView(v)}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 7,
        border: "none",
        cursor: "pointer",
        font: "inherit",
        fontSize: 12.5,
        fontWeight: 600,
        padding: "7px 13px",
        borderRadius: 7,
        background: view === v ? "var(--token-color-surface-primary)" : "transparent",
        color: view === v ? "var(--token-color-foreground-strong)" : "var(--token-color-foreground-faint)",
        boxShadow: view === v ? "var(--token-surface-base-box-shadow)" : "none",
      }}
    >
      <Icon name={icon} size={15} />
      {label}
    </button>
  );
  return (
    <div style={{ display: "flex", gap: 3, padding: 3, borderRadius: 9, background: "var(--token-color-surface-strong)" }}>
      {seg("position", "By position", "folder")}
      {seg("applicant", "By applicant", "users")}
    </div>
  );
}

function TypeChip({ label, active, onClick }: { label: string; active: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      style={{
        border: "1px solid " + (active ? "var(--token-color-border-action)" : "var(--token-color-border-faint)"),
        background: active ? "var(--token-color-surface-action)" : "transparent",
        color: active ? "var(--token-color-foreground-action)" : "var(--token-color-foreground-faint)",
        cursor: "pointer",
        font: "inherit",
        fontSize: 12,
        fontWeight: 600,
        padding: "4px 11px",
        borderRadius: 999,
      }}
    >
      {label}
    </button>
  );
}

// ---- position view ---------------------------------------------------
function PositionView({
  snapshot,
  idx,
  search,
  typeFilter,
  page,
  onSetPage,
  onOpenDetail,
}: {
  snapshot: Snapshot;
  idx: Indexes;
  search: string;
  typeFilter: TypeFilter;
  page: number;
  onSetPage: (p: number) => void;
  onOpenDetail: (type: "applicant" | "position", id: number) => void;
}) {
  const q = search.trim().toLowerCase();
  const positions = snapshot.positions
    .filter((p) => typeFilter === "all" || p.type === typeFilter)
    .filter(
      (p) => !q || p.name.toLowerCase().includes(q) || (idx.ccaById.get(p.ccaId)?.name ?? "").toLowerCase().includes(q),
    );
  if (positions.length === 0) {
    return <EmptyState title="No matching positions" sub="Try a different type, CCA or position name." />;
  }
  const per = 9;
  const pages = Math.ceil(positions.length / per);
  const clamped = Math.min(page, pages - 1);
  const shown = positions.slice(clamped * per, clamped * per + per);
  return (
    <div>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(330px, 1fr))", gap: 12 }}>
        {shown.map((p) => (
          <PositionCard key={p.id} pos={p} idx={idx} onOpenDetail={onOpenDetail} />
        ))}
      </div>
      <Pager total={positions.length} perPage={per} page={page} onSetPage={onSetPage} />
    </div>
  );
}

function PositionCard({
  pos,
  idx,
  onOpenDetail,
}: {
  pos: PositionView;
  idx: Indexes;
  onOpenDetail: (type: "applicant" | "position", id: number) => void;
}) {
  const cca = idx.ccaById.get(pos.ccaId);
  const seated = idx.seatsByPos.get(pos.id) ?? [];
  const filled = seated.length;
  const full = filled >= pos.capacity;
  return (
    <Card padding="none" style={{ padding: 0, overflow: "hidden" }}>
      <button
        onClick={() => onOpenDetail("position", pos.id)}
        style={{
          width: "100%",
          textAlign: "left",
          border: "none",
          background: "transparent",
          cursor: "pointer",
          padding: "13px 15px 11px",
          borderBottom: "1px solid var(--token-color-border-faint)",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: 10,
        }}
      >
        <div style={{ minWidth: 0 }}>
          <div style={{ fontSize: 10.5, color: "var(--token-color-foreground-faint)", fontWeight: 600 }}>{cca?.name}</div>
          <div style={{ display: "flex", alignItems: "center", gap: 7, marginTop: 1 }}>
            <span
              style={{
                fontSize: 14.5,
                fontWeight: 700,
                color: "var(--token-color-foreground-strong)",
                whiteSpace: "nowrap",
                overflow: "hidden",
                textOverflow: "ellipsis",
              }}
            >
              {pos.name}
            </span>
            <PositionTypeBadge type={pos.type} />
          </div>
        </div>
        <span
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            fontSize: 14,
            fontWeight: 700,
            fontFamily: "var(--token-typography-font-stack-code)",
            color: full ? "var(--token-color-foreground-faint)" : "var(--token-color-foreground-critical-on-surface)",
          }}
        >
          {filled}/{pos.capacity}
          <Icon name="chevron-right" size={14} color="var(--token-color-foreground-faint)" />
        </span>
      </button>
      <CapacityBar capacity={pos.capacity} seated={seated} />
      <div style={{ padding: "9px 15px 12px" }}>
        <ChairCoverage ranked={pos.chairRank.length} openSeats={Math.max(pos.capacity - filled, 0)} />
      </div>
    </Card>
  );
}

// One segment per seat, coloured by who holds it and grey for the vacancies.
function CapacityBar({ capacity, seated }: { capacity: number; seated: SeatView[] }) {
  const segs = seated.map((s) => statusStyle(s.status).dot);
  while (segs.length < capacity) segs.push("var(--token-color-surface-strong)");
  return (
    <div style={{ display: "flex", gap: 3, padding: "10px 15px 0" }}>
      {segs.map((c, i) => (
        <span key={i} style={{ flex: 1, height: 5, borderRadius: 3, background: c }} />
      ))}
    </div>
  );
}

// ---- applicant view ---------------------------------------------------
function ApplicantView({
  snapshot,
  idx,
  search,
  page,
  hasRun,
  onSetPage,
  onOpenDetail,
  onOpenMatch,
}: {
  snapshot: Snapshot;
  idx: Indexes;
  search: string;
  page: number;
  hasRun: boolean;
  onSetPage: (p: number) => void;
  onOpenDetail: (type: "applicant" | "position", id: number) => void;
  onOpenMatch: (aid: number, pid: number) => void;
}) {
  const q = search.trim().toLowerCase();
  const apps = snapshot.applicants.filter((a) => !q || a.name.toLowerCase().includes(q) || a.email.toLowerCase().includes(q));
  if (apps.length === 0) {
    return <EmptyState title="No matching applicants" sub="Try a different name." />;
  }
  const per = 8;
  const pages = Math.ceil(apps.length / per);
  const clamped = Math.min(page, pages - 1);
  const shown = apps.slice(clamped * per, clamped * per + per);
  return (
    <div>
      <Card padding="none" style={{ overflow: "hidden" }}>
        {shown.map((a, i) => (
          <ApplicantRow key={a.id} a={a} idx={idx} snapshot={snapshot} hasRun={hasRun} index={i} onOpenDetail={onOpenDetail} onOpenMatch={onOpenMatch} />
        ))}
      </Card>
      <Pager total={apps.length} perPage={per} page={page} onSetPage={onSetPage} />
    </div>
  );
}

function ApplicantRow({
  a,
  idx,
  snapshot,
  hasRun,
  index,
  onOpenDetail,
  onOpenMatch,
}: {
  a: ApplicantView;
  idx: Indexes;
  snapshot: Snapshot;
  hasRun: boolean;
  index: number;
  onOpenDetail: (type: "applicant" | "position", id: number) => void;
  onOpenMatch: (aid: number, pid: number) => void;
}) {
  const seats: { pid: number; status: Status }[] = [];
  for (const c of snapshot.committed) {
    if (c.applicantId === a.id) seats.push({ pid: c.positionId, status: "existing" });
  }
  if (hasRun) {
    for (const o of idx.newAllocations) {
      if (o.applicantId === a.id && !idx.committedSet.has(pk(o.applicantId, o.positionId))) {
        seats.push({ pid: o.positionId, status: "allocated" });
      }
    }
    for (const o of idx.preallocatedAllocations) {
      if (o.applicantId === a.id && !idx.committedSet.has(pk(o.applicantId, o.positionId))) {
        seats.push({ pid: o.positionId, status: "preallocated" });
      }
    }
  } else {
    // Pre-run the preallocation is the claim on the seat, exactly as the
    // position cards show it.
    for (const p of snapshot.preallocations) {
      if (p.applicantId === a.id && !idx.committedSet.has(pk(a.id, p.positionId))) {
        seats.push({ pid: p.positionId, status: "preallocated" });
      }
    }
  }
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 12,
        width: "100%",
        padding: "11px 15px",
        borderTop: index ? "1px solid var(--token-color-border-faint)" : "none",
      }}
    >
      <button
        onClick={() => onOpenDetail("applicant", a.id)}
        style={{ display: "flex", alignItems: "center", gap: 12, border: "none", background: "transparent", cursor: "pointer", textAlign: "left", font: "inherit", padding: 0 }}
      >
        <Avatar name={a.name} size="small" />
        <div style={{ width: 190, minWidth: 0 }}>
          <div
            style={{
              fontSize: 13.5,
              fontWeight: 700,
              color: "var(--token-color-foreground-strong)",
              whiteSpace: "nowrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
            }}
          >
            {a.name}
          </div>
          <div
            style={{
              fontSize: 11.5,
              color: "var(--token-color-foreground-faint)",
              whiteSpace: "nowrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
            }}
          >
            {a.email}
          </div>
        </div>
      </button>
      <div style={{ flex: 1, display: "flex", flexWrap: "wrap", gap: 6 }}>
        {seats.length ? (
          seats.map((s, k) => {
            const st = statusStyle(s.status);
            const pos = idx.posById.get(s.pid);
            const cca = pos ? idx.ccaById.get(pos.ccaId) : undefined;
            return (
              <button
                key={k}
                onClick={() => onOpenMatch(a.id, s.pid)}
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  gap: 6,
                  padding: "3px 9px",
                  borderRadius: 7,
                  background: st.bg,
                  border: "1px solid " + st.bd,
                  color: st.fg,
                  fontSize: 12,
                  fontWeight: 600,
                  cursor: "pointer",
                  font: "inherit",
                }}
              >
                <span style={{ width: 6, height: 6, borderRadius: "50%", background: st.dot }} />
                {(cca?.name ?? "") + " · " + (pos?.name ?? "")}
              </button>
            );
          })
        ) : (
          <span style={{ fontSize: 12, color: "var(--token-color-foreground-faint)", fontStyle: "italic" }}>
            {hasRun ? "No allocation" : "—"}
          </span>
        )}
      </div>
      <div style={{ width: 200, flexShrink: 0 }}>
        <ChoiceCoverage ranked={a.prefs.length} />
      </div>
      <button onClick={() => onOpenDetail("applicant", a.id)} style={{ border: "none", background: "transparent", cursor: "pointer", padding: 2, display: "flex" }}>
        <Icon name="chevron-right" size={16} color="var(--token-color-foreground-faint)" />
      </button>
    </div>
  );
}
