// Allocations.tsx — position/applicant allocation views.
// Ports the design's renderAllocations, positionView/positionCard/capacityBar and
// applicantView/applicantRow. All displayed values are looked up from snapshot/idx —
// this screen never recomputes statuses; the Rust backend already orders seated
// lists existing -> allocated -> preallocated. Filtering and sorting are view-only.
import { Card, Avatar, CcaIcon, ChairCoverage, ChoiceCoverage, Icon, PositionTypeBadge, TextInput } from "../components/index.ts";
import { ccaKindIcon } from "../lib/directory.ts";
import { statusStyle } from "../components/statusStyle.ts";
import { pk, type Indexes } from "../lib/indexes.ts";
import type { AllocState, View } from "../lib/allocState.ts";
import type { ApplicantView, PositionType, PositionView, SeatView, Snapshot, Status } from "../lib/types.ts";
import { ChipRow, EmptyState, Legend, Pager } from "./shared.tsx";

export interface AllocationsProps {
  snapshot: Snapshot;
  idx: Indexes;
  state: AllocState;
  /** Merge a patch into the filter state; detail/match panes close alongside. */
  onPatch: (patch: Partial<AllocState>) => void;
  onOpenDetail: (type: "applicant" | "position", id: number) => void;
  onOpenMatch: (aid: number, pid: number) => void;
  hasRun: boolean;
}

interface AllocStats {
  unfilledSeats: number;
  unfilledPositions: number;
  seatCount: Map<number, number>;
  unallocated: Set<number>;
  quota: Set<number>;
  displaced: Set<number>;
  ranked: Map<number, number>;
}

function allocStats(snapshot: Snapshot, idx: Indexes): AllocStats {
  const seatCount = new Map<number, number>();
  let unfilledSeats = 0;
  let unfilledPositions = 0;
  for (const p of snapshot.positions) {
    const seated = idx.seatsByPos.get(p.id) ?? [];
    for (const s of seated) seatCount.set(s.applicantId, (seatCount.get(s.applicantId) ?? 0) + 1);
    unfilledSeats += Math.max(p.capacity - seated.length, 0);
    if (seated.length < p.capacity) unfilledPositions += 1;
  }
  const withStatus = (st: Status) => new Set(snapshot.outcomes.filter((o) => o.status === st).map((o) => o.applicantId));
  const ranked = new Map<number, number>();
  for (const a of snapshot.applicants) for (const pid of a.prefs) ranked.set(pid, (ranked.get(pid) ?? 0) + 1);
  return {
    unfilledSeats,
    unfilledPositions,
    seatCount,
    unallocated: new Set(snapshot.applicants.filter((a) => !seatCount.get(a.id)).map((a) => a.id)),
    quota: withStatus("quota"),
    displaced: withStatus("displaced"),
    ranked,
  };
}

export function Allocations({ snapshot, idx, state, onPatch, onOpenDetail, onOpenMatch, hasRun }: AllocationsProps) {
  const stats = allocStats(snapshot, idx);
  const onApp = state.view === "applicant";
  const unfilledActive = !onApp && state.posFilter === "unfilled";
  const goUnfilled = () =>
    onPatch({
      view: "position",
      posFilter: unfilledActive ? "all" : "unfilled",
      appFilter: "all",
      typeFilter: "all",
      search: "",
      page: 0,
    });
  const crit = "var(--token-color-foreground-critical-on-surface)";
  return (
    <div style={{ padding: "24px 28px 48px", maxWidth: 1120, margin: "0 auto" }}>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(150px, 1fr))", gap: 10, marginBottom: 16 }}>
        <StatTile label="Unfilled seats" value={stats.unfilledSeats} color={crit} active={unfilledActive} onClick={goUnfilled} />
        <StatTile label="Unfilled positions" value={stats.unfilledPositions} color={crit} active={unfilledActive} onClick={goUnfilled} />
      </div>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 16, marginBottom: 12, flexWrap: "wrap" }}>
        <SegmentToggle view={state.view} onSetView={(view) => onPatch({ view, page: 0, search: "" })} />
        <div style={{ width: 260 }}>
          <TextInput
            icon={<Icon name="search" size={16} color="var(--token-color-foreground-faint)" />}
            placeholder={onApp ? "Search applicant name…" : "Search CCA or position…"}
            value={state.search}
            onChange={(e) => onPatch({ search: e.target.value, page: 0 })}
          />
        </div>
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 7, marginBottom: 12 }}>
        {onApp ? (
          <>
            <ChipRow
              label="Show"
              options={[["all", "All"], ["unallocated", "No allocation"], ["quota", "Quota-blocked"], ["displaced", "Displaced"]]}
              value={state.appFilter}
              onChange={(appFilter) => onPatch({ appFilter, page: 0 })}
            />
            <ChipRow
              label="Sort"
              options={[["name", "Name"], ["choices", "Fewest choices used"], ["unalloc", "Unallocated first"]]}
              value={state.appSort}
              onChange={(appSort) => onPatch({ appSort, page: 0 })}
            />
          </>
        ) : (
          <>
            <ChipRow
              label="Type"
              options={[["all", "All"], ["main", "Main"], ["block", "Block"], ["sub", "Sub"]]}
              value={state.typeFilter}
              onChange={(typeFilter) => onPatch({ typeFilter, page: 0 })}
            />
            <ChipRow
              label="Status"
              options={[["all", "All"], ["unfilled", "Unfilled"], ["oversub", "Over-subscribed"], ["prealloc", "Has preallocation"]]}
              value={state.posFilter}
              onChange={(posFilter) => onPatch({ posFilter, page: 0 })}
            />
          </>
        )}
      </div>
      <div style={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: "6px 16px", marginBottom: 16 }}>
        <Legend />
      </div>
      {onApp ? (
        <ApplicantList snapshot={snapshot} idx={idx} state={state} stats={stats} hasRun={hasRun} onPatch={onPatch} onOpenDetail={onOpenDetail} onOpenMatch={onOpenMatch} />
      ) : (
        <PositionList snapshot={snapshot} idx={idx} state={state} stats={stats} onPatch={onPatch} onOpenDetail={onOpenDetail} />
      )}
    </div>
  );
}

function StatTile({ label, value, color, active, onClick }: { label: string; value: number; color: string; active: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      style={{
        textAlign: "left",
        font: "inherit",
        cursor: "pointer",
        padding: "11px 14px",
        borderRadius: 10,
        border: "1px solid " + (active ? "var(--cupid-line)" : "var(--token-color-border-faint)"),
        background: active ? "var(--cupid-soft)" : "var(--token-color-surface-primary)",
        boxShadow: "var(--token-surface-base-box-shadow)",
      }}
    >
      <div style={{ fontSize: 22, fontWeight: 700, letterSpacing: "-0.5px", color, lineHeight: 1.1 }}>{value}</div>
      <div style={{ fontSize: 12, color: "var(--token-color-foreground-faint)", marginTop: 3 }}>{label}</div>
    </button>
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

// ---- position view ---------------------------------------------------
function PositionList({
  snapshot,
  idx,
  state,
  stats,
  onPatch,
  onOpenDetail,
}: {
  snapshot: Snapshot;
  idx: Indexes;
  state: AllocState;
  stats: AllocStats;
  onPatch: (patch: Partial<AllocState>) => void;
  onOpenDetail: (type: "applicant" | "position", id: number) => void;
}) {
  const q = state.search.trim().toLowerCase();
  const preallocated = new Set(snapshot.preallocations.map((p) => p.positionId));
  const statusMatch = (p: PositionView) => {
    switch (state.posFilter) {
      case "unfilled":
        return (idx.seatsByPos.get(p.id)?.length ?? 0) < p.capacity;
      case "oversub":
        return (stats.ranked.get(p.id) ?? 0) > p.capacity;
      case "prealloc":
        return preallocated.has(p.id);
      default:
        return true;
    }
  };
  const positions = snapshot.positions
    .filter((p) => state.typeFilter === "all" || p.type === state.typeFilter)
    .filter(statusMatch)
    .filter((p) => !q || p.name.toLowerCase().includes(q) || (idx.ccaById.get(p.ccaId)?.name ?? "").toLowerCase().includes(q));
  if (positions.length === 0) {
    return <EmptyState title="No matching positions" sub="Try a different filter, CCA or position name." />;
  }
  const per = 9;
  const pages = Math.ceil(positions.length / per);
  const clamped = Math.min(state.page, pages - 1);
  const shown = positions.slice(clamped * per, clamped * per + per);
  return (
    <div>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(330px, 1fr))", gap: 12 }}>
        {shown.map((p) => (
          <PositionCard key={p.id} pos={p} idx={idx} onOpenDetail={onOpenDetail} />
        ))}
      </div>
      <Pager total={positions.length} perPage={per} page={state.page} onSetPage={(page) => onPatch({ page })} />
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
        <div style={{ minWidth: 0, display: "flex", alignItems: "center", gap: 10 }}>
          <CcaIcon kind={cca?.kind} size={30} />
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
function ApplicantList({
  snapshot,
  idx,
  state,
  stats,
  hasRun,
  onPatch,
  onOpenDetail,
  onOpenMatch,
}: {
  snapshot: Snapshot;
  idx: Indexes;
  state: AllocState;
  stats: AllocStats;
  hasRun: boolean;
  onPatch: (patch: Partial<AllocState>) => void;
  onOpenDetail: (type: "applicant" | "position", id: number) => void;
  onOpenMatch: (aid: number, pid: number) => void;
}) {
  const q = state.search.trim().toLowerCase();
  const only = { all: null, unallocated: stats.unallocated, quota: stats.quota, displaced: stats.displaced }[state.appFilter];
  const byName = (x: ApplicantView, y: ApplicantView) => x.name.localeCompare(y.name);
  const seats = (a: ApplicantView) => stats.seatCount.get(a.id) ?? 0;
  const sorter =
    state.appSort === "choices"
      ? (x: ApplicantView, y: ApplicantView) => x.prefs.length - y.prefs.length || byName(x, y)
      : state.appSort === "unalloc"
        ? (x: ApplicantView, y: ApplicantView) => seats(x) - seats(y) || byName(x, y)
        : byName;
  const apps = snapshot.applicants
    .filter((a) => !q || a.name.toLowerCase().includes(q) || a.email.toLowerCase().includes(q))
    .filter((a) => !only || only.has(a.id))
    .sort(sorter);
  if (apps.length === 0) {
    return <EmptyState title="No matching applicants" sub="Try a different filter or name." />;
  }
  const per = 8;
  const pages = Math.ceil(apps.length / per);
  const clamped = Math.min(state.page, pages - 1);
  const shown = apps.slice(clamped * per, clamped * per + per);
  return (
    <div>
      <Card padding="none" style={{ overflow: "hidden" }}>
        {shown.map((a, i) => (
          <ApplicantRow key={a.id} a={a} idx={idx} snapshot={snapshot} hasRun={hasRun} index={i} onOpenDetail={onOpenDetail} onOpenMatch={onOpenMatch} />
        ))}
      </Card>
      <Pager total={apps.length} perPage={per} page={state.page} onSetPage={(page) => onPatch({ page })} />
    </div>
  );
}

// Headroom left under the quota rule (main + block <= 2, sub <= 3, and at
// most one sub alongside a main), phrased for the applicant list.
function quotaLine(idx: Indexes, aid: number) {
  const qa = idx.quotaByApp.get(aid);
  if (!qa) return null;
  const mainBlock = Math.max(2 - qa.main - qa.block, 0);
  const sub = Math.max((qa.main >= 1 ? 1 : 3) - qa.sub, 0);
  const text = qa.over ? "Over quota" : mainBlock + sub === 0 ? "Quota full" : `Room: ${mainBlock} main/block · ${sub} sub`;
  return (
    <div
      style={{
        fontSize: 11,
        whiteSpace: "nowrap",
        color: qa.over ? "var(--token-color-foreground-critical-on-surface)" : "var(--token-color-foreground-faint)",
        fontWeight: qa.over ? 700 : 500,
      }}
    >
      {text}
    </div>
  );
}

const TYPE_ORDER: Record<PositionType, number> = { main: 0, sub: 1, block: 2 };

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
  const rank = (pid: number) => {
    const pos = idx.posById.get(pid);
    return pos ? TYPE_ORDER[pos.type] : 3;
  };
  seats.sort((x, y) => rank(x.pid) - rank(y.pid));
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
          {quotaLine(idx, a.id)}
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
                <Icon name={ccaKindIcon(cca?.kind)} size={13} color={st.dot} />
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
