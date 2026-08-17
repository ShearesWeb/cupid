// EventSidebar.tsx — task-15: per-pair event-history sidebar.
// Ports reference/cca-console-design.html renderMatchSidebar (341-392), miniStat (374-379),
// timelineRow (380-389), jumpLink (390-392).
import { Icon, StatusPill } from "../components/index.ts";
import { statusStyle } from "../components/statusStyle.ts";
import { pk, type Indexes } from "../lib/indexes.ts";
import type { EventView, Snapshot, Status } from "../lib/types.ts";

export interface EventSidebarProps {
  match: { aid: number; pid: number };
  snapshot: Snapshot;
  idx: Indexes;
  onClose: () => void;
  onOpenDetail: (type: "applicant" | "position", id: number) => void;
}

interface TimelineItem {
  icon: string;
  status: Status;
  title: string;
  detail: string;
}

function buildTimeline(
  events: EventView[],
  hasRun: boolean,
  existing: boolean,
  quotaFull: boolean,
  preallocated: boolean,
): TimelineItem[] {
  const items: TimelineItem[] = [];
  if (existing) {
    items.push({
      icon: "check-circle",
      status: "existing",
      title: "Existing appointment",
      detail: "Already committed in the system of record before this run.",
    });
  }
  for (const e of events) {
    if (e.kind === "accept") {
      // A preallocated pair's accept is the operator's doing, not the matcher's.
      items.push(
        preallocated
          ? { icon: "check-circle", status: "preallocated", title: "Preallocated", detail: e.detail }
          : { icon: "check-circle", status: "allocated", title: "Allocated", detail: e.detail },
      );
    } else if (e.kind === "displace") {
      items.push({ icon: "alert-triangle", status: "displaced", title: "Displaced", detail: e.detail });
    } else {
      // reject
      items.push({
        icon: "x-circle",
        status: quotaFull ? "quota" : "neutral",
        title: quotaFull ? "Quota full" : "Rejected",
        detail: e.detail,
      });
    }
  }
  if (items.length === 0) {
    items.push({
      icon: "info",
      status: "neutral",
      title: hasRun ? "No events" : "No run yet",
      detail: hasRun
        ? "This pairing produced no events in the run."
        : "Run matching to see the event history.",
    });
  }
  return items;
}

export function EventSidebar({ match, snapshot, idx, onClose, onOpenDetail }: EventSidebarProps) {
  const { aid, pid } = match;
  const app = idx.appById.get(aid);
  const pos = idx.posById.get(pid);
  if (!app || !pos) return null;
  const cca = idx.ccaById.get(pos.ccaId);

  const key = pk(aid, pid);
  const outcome = idx.outcomeByPair.get(key);
  const events = idx.eventsByPair.get(key) ?? [];
  const existing = idx.committedSet.has(key);
  const hasRun = snapshot.run !== null;
  const cr = idx.chairRankOf(pid, aid);
  const pr = idx.prefRankOf(aid, pid);
  const items = buildTimeline(events, hasRun, existing, outcome?.status === "quota", outcome?.status === "preallocated");

  return (
    <aside
      style={{
        width: 364,
        flex: "0 0 364px",
        background: "var(--token-color-surface-primary)",
        borderLeft: "1px solid var(--token-color-border-primary)",
        display: "flex",
        flexDirection: "column",
        boxShadow: "var(--token-elevation-high-box-shadow)",
        zIndex: 20,
      }}
    >
      <div
        style={{
          padding: "13px 16px",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          borderBottom: "1px solid var(--token-color-border-faint)",
          flexShrink: 0,
        }}
      >
        <span
          style={{
            fontSize: 10.5,
            fontWeight: 700,
            letterSpacing: "0.7px",
            textTransform: "uppercase",
            color: "var(--token-color-foreground-faint)",
          }}
        >
          Event history
        </span>
        <button
          onClick={onClose}
          style={{ border: "none", background: "transparent", cursor: "pointer", padding: 4, display: "flex" }}
        >
          <Icon name="x" size={18} color="var(--token-color-foreground-faint)" />
        </button>
      </div>
      <div style={{ flex: 1, overflow: "auto", padding: 16 }}>
        <div style={{ marginBottom: 18 }}>
          <div
            style={{
              fontSize: 15,
              fontWeight: 700,
              color: "var(--token-color-foreground-strong)",
              letterSpacing: "-0.2px",
            }}
          >
            {app.name}
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 7, marginTop: 4, flexWrap: "wrap" }}>
            <Icon name="arrow-right" size={13} color="var(--token-color-foreground-faint)" />
            <span style={{ fontSize: 13, color: "var(--token-color-foreground-primary)", fontWeight: 600 }}>
              {`${cca?.name ?? ""} · ${pos.name}`}
            </span>
            {outcome ? <StatusPill status={outcome.status} label={outcome.label} /> : null}
          </div>
          <div style={{ display: "flex", gap: 8, marginTop: 12 }}>
            <MiniStat label="Chair rank" value={cr == null ? "—" : String(cr)} />
            <MiniStat label="Their preference" value={pr == null ? "—" : `#${pr}`} />
          </div>
        </div>
        <div
          style={{
            fontSize: 10.5,
            fontWeight: 700,
            letterSpacing: "0.6px",
            textTransform: "uppercase",
            color: "#DB2A63",
            marginBottom: 12,
          }}
        >
          Timeline
        </div>
        <div style={{ display: "flex", flexDirection: "column" }}>
          {items.map((it, i) => (
            <TimelineRow key={i} item={it} last={i === items.length - 1} />
          ))}
        </div>
        <div
          style={{
            display: "flex",
            gap: 8,
            marginTop: 18,
            paddingTop: 14,
            borderTop: "1px solid var(--token-color-border-faint)",
          }}
        >
          <JumpLink
            label="View applicant"
            onClick={() => {
              onClose();
              onOpenDetail("applicant", aid);
            }}
          />
          <JumpLink
            label="View position"
            onClick={() => {
              onClose();
              onOpenDetail("position", pid);
            }}
          />
        </div>
      </div>
    </aside>
  );
}

function MiniStat({ label, value }: { label: string; value: string }) {
  return (
    <div
      style={{
        flex: 1,
        padding: "8px 11px",
        borderRadius: 8,
        background: "var(--token-color-surface-faint)",
        border: "1px solid var(--token-color-border-faint)",
      }}
    >
      <div
        style={{
          fontSize: 14,
          fontWeight: 700,
          fontFamily: "var(--token-typography-font-stack-code)",
          color: "var(--token-color-foreground-strong)",
        }}
      >
        {value}
      </div>
      <div style={{ fontSize: 10.5, color: "var(--token-color-foreground-faint)", marginTop: 2 }}>{label}</div>
    </div>
  );
}

function TimelineRow({ item, last }: { item: TimelineItem; last: boolean }) {
  const st = statusStyle(item.status);
  return (
    <div style={{ display: "flex", gap: 11 }}>
      <div style={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
        <div
          style={{
            width: 26,
            height: 26,
            borderRadius: "50%",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: st.bg,
            border: `1px solid ${st.bd}`,
            flexShrink: 0,
            zIndex: 1,
          }}
        >
          <Icon name={item.icon} size={14} color={st.dot} />
        </div>
        {last ? null : (
          <div style={{ width: 2, flex: 1, background: "var(--token-color-border-faint)", minHeight: 14 }} />
        )}
      </div>
      <div style={{ paddingBottom: last ? 0 : 16, flex: 1 }}>
        <div style={{ fontSize: 13, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>
          {item.title}
        </div>
        <div style={{ fontSize: 12, color: "var(--token-color-foreground-faint)", marginTop: 2, lineHeight: 1.45 }}>
          {item.detail}
        </div>
      </div>
    </div>
  );
}

function JumpLink({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      style={{
        flex: 1,
        height: 32,
        borderRadius: 7,
        border: "1px solid var(--token-color-border-strong)",
        background: "var(--token-color-surface-primary)",
        cursor: "pointer",
        font: "inherit",
        fontSize: 12,
        fontWeight: 600,
        color: "var(--token-color-foreground-strong)",
      }}
    >
      {label}
    </button>
  );
}
