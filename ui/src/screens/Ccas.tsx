// Ccas.tsx — CCA directory home: the CCA list and one CCA's positions and holders.
// Ports the design's renderCcaList / renderCcaDetail / renderCcaModal onto the real
// DirectorySnapshot. Edits are pending changes validated in Rust (capacity, the
// one-CCA-per-semester rule, read-only residents) and published from Review.
// The design's position add/edit/delete controls are not ported: positions are
// read-only until the backend can change and publish them.
import { useEffect, useMemo, useState, type MouseEvent, type ReactNode } from "react";
import { Avatar, Badge, Button, Card, CcaIcon, Icon, TextInput, type BadgeColor } from "../components/index.ts";
import { Combo, type ComboOption } from "../components/Combo.tsx";
import type { ToastKind } from "../components/Toasts.tsx";
import {
  PERIODS,
  buildDirectoryIndex,
  capacityText,
  isEditable,
  isFull,
  kindLabel,
  periodLabel,
  typeLabel,
  type DirectoryIndex,
  type RemovedHolder,
} from "../lib/directory.ts";
import type {
  CommitmentPeriod,
  DirectoryAppointment,
  DirectoryCca,
  DirectoryPosition,
  DirectoryPositionType,
  DirectorySnapshot,
} from "../lib/types.ts";
import { ChipRow, EmptyState, PageTitle } from "./shared.tsx";

export interface CcasProps {
  directory: DirectorySnapshot;
  openCcaId: number | null;
  onOpenCca: (id: number | null) => void;
  onAdd: (userId: number, positionId: number, period: CommitmentPeriod) => Promise<boolean>;
  onRemove: (userId: number, positionId: number) => Promise<boolean>;
  onUpdatePeriod: (userId: number, positionId: number, period: CommitmentPeriod) => Promise<boolean>;
  toast: (kind: ToastKind, text: string) => void;
}

export function Ccas(props: CcasProps) {
  const { directory, openCcaId, onOpenCca } = props;
  const dx = useMemo(() => buildDirectoryIndex(directory), [directory]);
  const cca = openCcaId === null ? undefined : directory.ccas.find((c) => c.id === openCcaId);
  // Keyed so search, filters and expansion reset per CCA.
  if (cca) return <CcaDetail key={cca.id} {...props} cca={cca} dx={dx} onBack={() => onOpenCca(null)} />;
  return <CcaList directory={directory} dx={dx} onOpen={onOpenCca} />;
}

// ---- list ------------------------------------------------------------
function CcaList({ directory, dx, onOpen }: { directory: DirectorySnapshot; dx: DirectoryIndex; onOpen: (id: number) => void }) {
  const [kind, setKind] = useState<string>("all");
  const [search, setSearch] = useState("");
  const kinds = [...new Set(directory.ccas.map((c) => c.kind))];
  const q = search.trim().toLowerCase();
  const list = directory.ccas
    .filter((c) => kind === "all" || c.kind === kind)
    .filter((c) => !q || c.name.toLowerCase().includes(q));
  return (
    <div style={pageStyle(1120)}>
      <PageTitle title="CCAs" sub="Every CCA in the hall. Open one to manage its members." />
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 16, marginBottom: 16, flexWrap: "wrap" }}>
        <ChipRow label="Kind" options={[["all", "All"], ...kinds.map((k): [string, string] => [k, kindLabel(k)])]} value={kind} onChange={setKind} />
        <div style={{ width: 260 }}>
          <SearchField placeholder="Search CCAs…" value={search} onChange={setSearch} />
        </div>
      </div>
      {list.length === 0 ? (
        <EmptyState title="No matching CCAs" sub="Try a different kind or CCA name." />
      ) : (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))", gap: 12 }}>
          {list.map((c) => (
            <CcaCard key={c.id} cca={c} dx={dx} onClick={() => onOpen(c.id)} />
          ))}
        </div>
      )}
    </div>
  );
}

function vacantPositions(dx: DirectoryIndex, ccaId: number) {
  return (dx.positionsByCca.get(ccaId) ?? []).filter((p) => p.capacity !== 0 && !dx.holdersByPosition.get(p.id)?.length);
}

function CcaCard({ cca, dx, onClick }: { cca: DirectoryCca; dx: DirectoryIndex; onClick: () => void }) {
  const n = dx.memberCount(cca.id);
  const vacant = vacantPositions(dx, cca.id);
  return (
    <button
      onClick={onClick}
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 12,
        textAlign: "left",
        font: "inherit",
        cursor: "pointer",
        padding: 16,
        borderRadius: 10,
        border: "1px solid var(--token-color-border-faint)",
        background: "var(--token-color-surface-primary)",
        boxShadow: "var(--token-surface-base-box-shadow)",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 11, width: "100%" }}>
        <CcaIcon kind={cca.kind} imageUrl={cca.imageUrl} size={36} />
        <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-start", gap: 4, minWidth: 0, flex: 1 }}>
          <span style={{ ...ellipsis, maxWidth: "100%", fontSize: 15, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>{cca.name}</span>
          <Badge color="neutral" text={kindLabel(cca.kind)} />
        </div>
        <Icon name="chevron-right" size={16} color="var(--token-color-foreground-faint)" />
      </div>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 6,
          width: "100%",
          paddingTop: 12,
          borderTop: "1px solid var(--token-color-border-faint)",
          fontSize: 12.5,
          color: "var(--token-color-foreground-faint)",
        }}
      >
        <Icon name="users" size={14} color="var(--token-color-foreground-faint)" />
        <b style={{ color: "var(--token-color-foreground-strong)" }}>{n}</b>
        {n === 1 ? " member" : " members"}
        <span style={{ flex: 1 }} />
        {vacant.length ? <VacantNote count={vacant.length} long title={"Vacant: " + vacant.map((p) => p.name).join(", ")} /> : null}
      </div>
    </button>
  );
}

function VacantNote({ count, long, title }: { count: number; long?: boolean; title?: string }) {
  const warn = "var(--token-color-foreground-warning-on-surface)";
  return (
    <span title={title} style={{ display: "inline-flex", alignItems: "center", gap: 5, fontWeight: 600, whiteSpace: "nowrap", color: warn }}>
      <Icon name="alert-triangle" size={13} color={warn} />
      {long ? `${count} vacant position${count === 1 ? "" : "s"}` : `${count} vacant`}
    </span>
  );
}

// ---- detail ----------------------------------------------------------
type Modal =
  | { mode: "add"; userId: number | null; positionId: number | null; period: CommitmentPeriod }
  | { mode: "member"; userId: number; positionId: number; period: CommitmentPeriod; confirm: boolean };

function CcaDetail({
  cca,
  dx,
  onBack,
  onAdd,
  onRemove,
  onUpdatePeriod,
  toast,
}: CcasProps & { cca: DirectoryCca; dx: DirectoryIndex; onBack: () => void }) {
  const [memSearch, setMemSearch] = useState("");
  const [memType, setMemType] = useState<"all" | DirectoryPositionType>("all");
  const [posOpen, setPosOpen] = useState<Record<number, boolean>>({});
  const [modal, setModal] = useState<Modal | null>(null);
  const [busy, setBusy] = useState(false);

  const positions = dx.positionsByCca.get(cca.id) ?? [];
  const holders = (pid: number) =>
    [...(dx.holdersByPosition.get(pid) ?? [])].sort((x, y) => nameOf(dx, x.userId).localeCompare(nameOf(dx, y.userId)));
  const vacant = vacantPositions(dx, cca.id);
  const members = dx.memberCount(cca.id);
  const types = [...new Set(positions.map((p) => p.positionType))];
  const typeCount = (t: DirectoryPositionType) =>
    positions.filter((p) => p.positionType === t).reduce((n, p) => n + (dx.holdersByPosition.get(p.id)?.length ?? 0), 0);
  const holderCount = positions.reduce((n, p) => n + (dx.holdersByPosition.get(p.id)?.length ?? 0), 0);

  const q = memSearch.trim().toLowerCase();
  const matches = (userId: number) => {
    const u = dx.userById.get(userId);
    return !q || (u ? u.name.toLowerCase().includes(q) || u.email.toLowerCase().includes(q) : false);
  };
  const isOpen = (pid: number, n: number) => !!q || (posOpen[pid] ?? n <= 8);
  const anyOpen = positions.some((p) => {
    const n = dx.holdersByPosition.get(p.id)?.length ?? 0;
    return n > 0 && isOpen(p.id, n);
  });
  const setAll = (open: boolean) => setPosOpen(Object.fromEntries(positions.map((p) => [p.id, open])));
  const editable = positions.filter(isEditable);

  const act = async (run: () => Promise<boolean>, success: string) => {
    setBusy(true);
    const ok = await run();
    setBusy(false);
    if (ok) {
      setModal(null);
      toast("success", success);
    }
  };

  const sections = types
    .filter((t) => memType === "all" || memType === t)
    .map((t) => {
      const cards = positions
        .filter((p) => p.positionType === t)
        .map((p) => {
          const hs = holders(p.id);
          const removed = dx.removedByPosition.get(p.id) ?? [];
          const rows = hs.filter((a) => matches(a.userId));
          const removedRows = removed.filter((r) => matches(r.userId));
          if (q && !rows.length && !removedRows.length) return null;
          const open = hs.length + removed.length > 0 && isOpen(p.id, hs.length);
          return (
            <PositionCard
              key={p.id}
              position={p}
              holders={hs}
              rows={rows}
              removedRows={removedRows}
              open={open}
              dx={dx}
              onToggle={() => setPosOpen((prev) => ({ ...prev, [p.id]: !open }))}
              onAdd={() => setModal({ mode: "add", userId: null, positionId: p.id, period: "full-year" })}
              onEdit={(a) => setModal({ mode: "member", userId: a.userId, positionId: a.positionId, period: a.commitmentPeriod, confirm: false })}
            />
          );
        })
        .filter((card) => card !== null);
      if (!cards.length) return null;
      return (
        <div key={t} style={{ marginBottom: 24 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 9 }}>
            <span style={{ fontSize: 10.5, fontWeight: 700, letterSpacing: "0.6px", textTransform: "uppercase", whiteSpace: "nowrap", color: "var(--cupid)" }}>
              {typeLabel(t)}
            </span>
            <span style={{ fontSize: 11.5, fontWeight: 600, color: "var(--token-color-foreground-faint)" }}>{typeCount(t)}</span>
          </div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(300px, 1fr))", gap: 12, alignItems: "start" }}>{cards}</div>
        </div>
      );
    })
    .filter((section) => section !== null);

  const meta = [
    `${members} member${members === 1 ? "" : "s"}`,
    `${positions.length} position${positions.length === 1 ? "" : "s"}`,
    cca.tier !== "none" ? `Tier ${cca.tier.slice("tier-".length)}` : null,
    cca.type !== "none" ? `Type ${cca.type.slice("type-".length).toUpperCase()}` : null,
  ].filter(Boolean);

  return (
    <div style={pageStyle(1040)}>
      <button onClick={onBack} style={backButtonStyle}>
        <Icon name="chevron-left" size={15} color="var(--token-color-foreground-strong)" />
        All CCAs
      </button>
      <div style={{ display: "flex", alignItems: "center", gap: 13, marginBottom: 20, flexWrap: "wrap" }}>
        <CcaIcon kind={cca.kind} imageUrl={cca.imageUrl} size={44} />
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 9 }}>
            <span style={{ fontSize: 21, fontWeight: 700, letterSpacing: "-0.4px", color: "var(--token-color-foreground-strong)" }}>{cca.name}</span>
            <Badge color="neutral" text={kindLabel(cca.kind)} />
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 10, fontSize: 12.5, color: "var(--token-color-foreground-faint)", marginTop: 3, flexWrap: "wrap" }}>
            <span style={{ whiteSpace: "nowrap" }}>{meta.join(" · ")}</span>
            {vacant.length ? <VacantNote count={vacant.length} /> : null}
          </div>
          {cca.description ? (
            <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)", marginTop: 6, maxWidth: 620 }}>{cca.description}</div>
          ) : null}
        </div>
        <Button
          color="primary"
          icon="plus"
          disabled={editable.length === 0}
          onClick={() => setModal({ mode: "add", userId: null, positionId: null, period: "full-year" })}
        >
          Add member
        </Button>
      </div>
      {positions.length === 0 ? (
        <EmptyState icon="tag" title="No positions yet" sub="This CCA has no positions in the directory." />
      ) : (
        <>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              gap: 16,
              marginBottom: 20,
              paddingBottom: 16,
              borderBottom: "1px solid var(--token-color-border-faint)",
              flexWrap: "wrap",
            }}
          >
            <ChipRow<"all" | DirectoryPositionType>
              label="Type"
              options={[["all", `All · ${holderCount}`], ...types.map((t): [DirectoryPositionType, string] => [t, `${typeLabel(t)} · ${typeCount(t)}`])]}
              value={memType}
              onChange={setMemType}
            />
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              {q ? null : (
                <button
                  onClick={() => setAll(!anyOpen)}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                    height: 34,
                    padding: "0 10px",
                    borderRadius: 7,
                    font: "inherit",
                    fontSize: 12.5,
                    fontWeight: 600,
                    cursor: "pointer",
                    whiteSpace: "nowrap",
                    border: "none",
                    background: "transparent",
                    color: "var(--token-color-foreground-primary)",
                  }}
                >
                  <Icon name={anyOpen ? "chevron-up" : "chevron-down"} size={14} color="var(--token-color-foreground-faint)" />
                  {anyOpen ? "Collapse all" : "Expand all"}
                </button>
              )}
              <div style={{ width: 240 }}>
                <SearchField placeholder="Search members…" value={memSearch} onChange={setMemSearch} />
              </div>
            </div>
          </div>
          {sections.length ? sections : <EmptyState title="No matching members" sub="Try a different name, email or type." />}
        </>
      )}
      {modal ? (
        <MemberModal
          cca={cca}
          dx={dx}
          positions={editable}
          modal={modal}
          busy={busy}
          onChange={(patch) => setModal((m) => (m ? ({ ...m, ...patch } as Modal) : m))}
          onClose={() => setModal(null)}
          onSaveAdd={(userId, positionId, period) =>
            void act(
              () => onAdd(userId, positionId, period),
              `${nameOf(dx, userId)} added as ${dx.positionById.get(positionId)?.name ?? "member"} (${periodLabel(period)}). Pending review.`,
            )
          }
          onSavePeriod={(userId, positionId, period) =>
            void act(() => onUpdatePeriod(userId, positionId, period), `${nameOf(dx, userId)}'s commitment set to ${periodLabel(period)}. Pending review.`)
          }
          onRemove={(userId, positionId) =>
            void act(() => onRemove(userId, positionId), `${nameOf(dx, userId)} removed from ${cca.name}. Pending review.`)
          }
        />
      ) : null}
    </div>
  );
}

const nameOf = (dx: DirectoryIndex, userId: number) => dx.userById.get(userId)?.name ?? `User ${userId}`;

function PositionCard({
  position,
  holders,
  rows,
  removedRows,
  open,
  dx,
  onToggle,
  onAdd,
  onEdit,
}: {
  position: DirectoryPosition;
  holders: DirectoryAppointment[];
  rows: DirectoryAppointment[];
  removedRows: RemovedHolder[];
  open: boolean;
  dx: DirectoryIndex;
  onToggle: () => void;
  onAdd: () => void;
  onEdit: (a: DirectoryAppointment) => void;
}) {
  const n = holders.length;
  const expandable = n + removedRows.length > 0;
  const closed = position.capacity === 0;
  const full = isFull(position, n);
  const editable = isEditable(position);
  const canAdd = editable && !full;
  const wide = n > 6;
  const reporting = position.reportingPositionId !== null ? dx.positionById.get(position.reportingPositionId) : undefined;
  return (
    <Card padding="none" style={{ padding: 0, overflow: "hidden", gridColumn: wide && open ? "1 / -1" : "auto" }}>
      <div style={{ display: "flex", alignItems: "center", gap: 4, padding: "6px 8px 6px 6px", minHeight: 50 }}>
        <button
          onClick={expandable ? onToggle : undefined}
          aria-expanded={open}
          style={{
            flex: 1,
            minWidth: 0,
            display: "flex",
            alignItems: "center",
            gap: 8,
            padding: "6px 8px",
            border: "none",
            background: "transparent",
            font: "inherit",
            textAlign: "left",
            cursor: expandable ? "pointer" : "default",
            borderRadius: 6,
          }}
        >
          <span style={{ display: "flex", width: 16, flexShrink: 0, opacity: expandable ? 1 : 0.35 }}>
            <Icon name={open ? "chevron-down" : "chevron-right"} size={15} color="var(--token-color-foreground-faint)" />
          </span>
          <span style={{ display: "flex", flexDirection: "column", minWidth: 0, flexShrink: 1, maxWidth: "60%" }}>
            <span style={{ ...ellipsis, fontSize: 14, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>{position.name}</span>
            {reporting ? <span style={{ ...ellipsis, fontSize: 11, color: "var(--token-color-foreground-faint)" }}>Reports to {reporting.name}</span> : null}
          </span>
          <span
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 6,
              fontSize: 12,
              fontWeight: 600,
              whiteSpace: "nowrap",
              flexShrink: 0,
              fontFamily: "var(--token-typography-font-stack-code)",
              color: n || closed ? "var(--token-color-foreground-faint)" : "var(--token-color-foreground-warning-on-surface)",
            }}
          >
            {closed ? <Badge color="neutral" text="Closed" /> : capacityText(position, n)}
            {closed ? null : full ? <Badge color="success" text="Full" /> : n ? null : <span style={{ fontFamily: "var(--token-typography-font-stack-display)" }}>Vacant</span>}
          </span>
          <span style={{ flex: 1 }} />
          {n && !open ? <AvatarStack userIds={holders.map((a) => a.userId)} dx={dx} /> : null}
        </button>
        {canAdd ? <IconButton icon="plus" title={`Add to ${position.name}`} onClick={onAdd} /> : null}
      </div>
      {!n && !removedRows.length ? (
        canAdd ? (
          <button
            onClick={onAdd}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 10,
              width: "100%",
              textAlign: "left",
              font: "inherit",
              cursor: "pointer",
              padding: "12px 16px",
              border: "none",
              borderTop: "1px dashed var(--token-color-border-strong)",
              background: "transparent",
            }}
          >
            <span
              style={{
                width: 28,
                height: 28,
                borderRadius: "50%",
                flexShrink: 0,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                background: "var(--cupid-soft)",
              }}
            >
              <Icon name="plus" size={14} color="var(--cupid-strong)" />
            </span>
            <span style={{ fontSize: 12.5, fontWeight: 600, color: "var(--token-color-foreground-strong)" }}>Assign someone to {position.name}</span>
          </button>
        ) : null
      ) : open ? (
        <div style={wide ? { display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))" } : undefined}>
          {rows.map((a) => (
            <MemberRow key={a.userId} appointment={a} dx={dx} onEdit={editable ? () => onEdit(a) : undefined} />
          ))}
          {removedRows.map((r) => (
            <RemovedRow key={`removed-${r.userId}`} holder={r} dx={dx} />
          ))}
        </div>
      ) : null}
    </Card>
  );
}

function AvatarStack({ userIds, dx }: { userIds: number[]; dx: DirectoryIndex }) {
  return (
    <span style={{ display: "flex", alignItems: "center", minWidth: 0, overflow: "hidden", flexShrink: 1 }}>
      {userIds.slice(0, 3).map((id, i) => {
        const name = nameOf(dx, id);
        const initials = name
          .split(" ")
          .map((w) => w[0])
          .slice(0, 2)
          .join("");
        return (
          <span
            key={id}
            title={name}
            style={{
              width: 24,
              height: 24,
              borderRadius: "50%",
              marginLeft: i ? -7 : 0,
              flexShrink: 0,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: 9.5,
              fontWeight: 700,
              background: "var(--cupid-soft)",
              color: "var(--cupid-strong)",
              border: "2px solid var(--token-color-surface-primary)",
            }}
          >
            {initials}
          </span>
        );
      })}
      {userIds.length > 3 ? (
        <span style={{ marginLeft: 6, fontSize: 11.5, fontWeight: 600, whiteSpace: "nowrap", color: "var(--token-color-foreground-faint)" }}>
          +{userIds.length - 3}
        </span>
      ) : null}
    </span>
  );
}

// Pending-change marker on a holder row: the change is local until committed.
const CHANGE_TONES = {
  added: { label: "Added", tone: "success" },
  modified: { label: "Modified", tone: "warning" },
  removed: { label: "Removed", tone: "critical" },
} as const;

function ChangeTag({ kind }: { kind: keyof typeof CHANGE_TONES }) {
  const { label, tone } = CHANGE_TONES[kind];
  return (
    <span
      title="Pending until committed"
      style={{
        display: "inline-flex",
        alignItems: "center",
        height: 18,
        padding: "0 6px",
        borderRadius: 5,
        background: `var(--token-color-surface-${tone})`,
        color: `var(--token-color-foreground-${tone}-on-surface)`,
        border: `1px solid var(--token-color-border-${tone})`,
        fontSize: 10,
        fontWeight: 700,
        letterSpacing: 0.3,
        textTransform: "uppercase",
        whiteSpace: "nowrap",
      }}
    >
      {label}
    </span>
  );
}

function MemberRow({ appointment, dx, onEdit }: { appointment: DirectoryAppointment; dx: DirectoryIndex; onEdit?: () => void }) {
  const user = dx.userById.get(appointment.userId);
  const name = user?.name ?? `User ${appointment.userId}`;
  const fullYear = appointment.commitmentPeriod === "full-year";
  return (
    <div
      role={onEdit ? "button" : undefined}
      tabIndex={onEdit ? 0 : undefined}
      title={onEdit ? `Edit ${name}` : undefined}
      onClick={onEdit}
      onKeyDown={(e) => {
        if (onEdit && e.key === "Enter") onEdit();
      }}
      style={{ ...rowStyle, cursor: onEdit ? "pointer" : "default", paddingRight: onEdit ? 8 : 16 }}
    >
      <Avatar name={name} size="small" />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ ...ellipsis, fontSize: 13, fontWeight: 600, color: "var(--token-color-foreground-strong)" }}>{name}</div>
        <div style={{ ...ellipsis, fontSize: 11.5, color: "var(--token-color-foreground-faint)" }}>{user?.email ?? ""}</div>
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 6, flexShrink: 0 }}>
        {appointment.status !== "existing" ? <ChangeTag kind={appointment.status} /> : null}
        <span
          title="Commitment"
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: 5,
            height: 22,
            padding: "0 8px",
            borderRadius: 6,
            fontSize: 11.5,
            fontWeight: 600,
            whiteSpace: "nowrap",
            color: fullYear ? "var(--token-color-foreground-strong)" : "var(--token-color-foreground-faint)",
            background: "var(--token-color-surface-faint)",
            border: "1px solid var(--token-color-border-faint)",
          }}
        >
          <Icon name="calendar" size={12} color="var(--token-color-foreground-faint)" />
          {periodLabel(appointment.commitmentPeriod)}
        </span>
        {onEdit ? (
          <IconButton
            icon="edit"
            title={`Edit ${name}`}
            onClick={(e) => {
              e.stopPropagation();
              onEdit();
            }}
          />
        ) : null}
      </div>
    </div>
  );
}

function RemovedRow({ holder, dx }: { holder: RemovedHolder; dx: DirectoryIndex }) {
  const user = dx.userById.get(holder.userId);
  const name = user?.name ?? `User ${holder.userId}`;
  return (
    <div style={{ ...rowStyle, paddingRight: 16, opacity: 0.7 }}>
      <Avatar name={name} size="small" />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ ...ellipsis, fontSize: 13, fontWeight: 600, color: "var(--token-color-foreground-faint)", textDecoration: "line-through" }}>{name}</div>
        <div style={{ ...ellipsis, fontSize: 11.5, color: "var(--token-color-foreground-faint)" }}>{user?.email ?? ""}</div>
      </div>
      <ChangeTag kind="removed" />
    </div>
  );
}

// ---- modal -----------------------------------------------------------
function MemberModal({
  cca,
  dx,
  positions,
  modal,
  busy,
  onChange,
  onClose,
  onSaveAdd,
  onSavePeriod,
  onRemove,
}: {
  cca: DirectoryCca;
  dx: DirectoryIndex;
  positions: DirectoryPosition[];
  modal: Modal;
  busy: boolean;
  onChange: (patch: Partial<Modal>) => void;
  onClose: () => void;
  onSaveAdd: (userId: number, positionId: number, period: CommitmentPeriod) => void;
  onSavePeriod: (userId: number, positionId: number, period: CommitmentPeriod) => void;
  onRemove: (userId: number, positionId: number) => void;
}) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [onClose]);

  const commitField = (
    <div>
      <FieldLabel>Commitment</FieldLabel>
      <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
        {PERIODS.map(([value, label]) => (
          <ChoiceChip key={value} label={label} on={modal.period === value} onClick={() => onChange({ period: value })} />
        ))}
      </div>
    </div>
  );

  let title: string;
  let body: ReactNode;
  let footerLeft: ReactNode = null;
  let save: ReactNode;

  if (modal.mode === "member") {
    const user = dx.userById.get(modal.userId);
    const position = dx.positionById.get(modal.positionId);
    const current = dx.holdersByPosition.get(modal.positionId)?.find((a) => a.userId === modal.userId);
    const changed = !!current && current.commitmentPeriod !== modal.period;
    title = "Edit member";
    body = (
      <>
        <div style={{ ...summaryStyle, gap: 12 }}>
          <Avatar name={user?.name ?? "?"} size="medium" />
          <div style={{ flex: 1, minWidth: 0 }}>
            <div style={{ fontSize: 14, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>{user?.name ?? `User ${modal.userId}`}</div>
            <div style={{ fontSize: 12, color: "var(--token-color-foreground-faint)" }}>{user?.email}</div>
          </div>
          {position ? (
            <span style={{ display: "flex", flexDirection: "column", alignItems: "flex-end", gap: 4 }}>
              <span style={{ fontSize: 12.5, fontWeight: 600, color: "var(--token-color-foreground-strong)" }}>{position.name}</span>
              <DirectoryTypeBadge type={position.positionType} />
            </span>
          ) : null}
        </div>
        {commitField}
      </>
    );
    footerLeft = modal.confirm ? (
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <CriticalButton label="Confirm remove" solid disabled={busy} onClick={() => onRemove(modal.userId, modal.positionId)} />
        <button
          onClick={() => onChange({ confirm: false })}
          style={{ border: "none", background: "transparent", font: "inherit", fontSize: 12.5, fontWeight: 600, cursor: "pointer", color: "var(--token-color-foreground-faint)" }}
        >
          Keep
        </button>
      </div>
    ) : (
      <CriticalButton label="Remove member" disabled={busy} onClick={() => onChange({ confirm: true })} />
    );
    save = (
      <Button color="primary" icon="check" busy={busy} disabled={!changed} onClick={() => onSavePeriod(modal.userId, modal.positionId, modal.period)}>
        Save changes
      </Button>
    );
  } else {
    const { userId, positionId, period } = modal;
    // One non-resident position per CCA: anyone already holding one here is
    // not offered again. Rust re-checks this, per semester, on save.
    const taken = new Set<number>();
    for (const p of positions) for (const a of dx.holdersByPosition.get(p.id) ?? []) taken.add(a.userId);
    const options = (query: string): ComboOption[] => {
      const n = query.trim().toLowerCase();
      const out: ComboOption[] = [];
      for (const u of dx.userById.values()) {
        if (taken.has(u.id)) continue;
        if (n && !u.name.toLowerCase().includes(n) && !u.email.toLowerCase().includes(n)) continue;
        out.push({ id: u.id, label: u.name, sub: u.email });
        if (out.length >= 6) break;
      }
      return out;
    };
    const selectedUser = userId !== null ? dx.userById.get(userId) : undefined;
    title = "Add member";
    body = (
      <>
        <div>
          <FieldLabel>Resident</FieldLabel>
          <div style={{ display: "flex" }}>
            <Combo
              placeholder="Search by name or email…"
              options={options}
              selected={selectedUser ? { id: selectedUser.id, label: selectedUser.name } : null}
              onSelect={(o) => onChange({ userId: o ? o.id : null })}
            />
          </div>
        </div>
        <div>
          <FieldLabel>Position</FieldLabel>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(2, minmax(0, 1fr))", gap: 8, maxHeight: 300, overflow: "auto", padding: 1 }}>
            {positions.map((p) => (
              <PositionTile key={p.id} position={p} held={dx.holdersByPosition.get(p.id)?.length ?? 0} on={positionId === p.id} onClick={() => onChange({ positionId: p.id })} />
            ))}
          </div>
        </div>
        {commitField}
      </>
    );
    save = (
      <Button
        color="primary"
        icon="plus"
        busy={busy}
        disabled={userId === null || positionId === null}
        onClick={() => {
          if (userId !== null && positionId !== null) onSaveAdd(userId, positionId, period);
        }}
      >
        Add member
      </Button>
    );
  }

  return (
    <div
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 60,
        background: "rgba(10,10,11,0.45)",
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "center",
        padding: "10vh 20px 20px",
        overflow: "auto",
      }}
    >
      <div
        role="dialog"
        aria-label={title}
        style={{
          width: "100%",
          maxWidth: 540,
          borderRadius: 12,
          background: "var(--token-color-surface-primary)",
          boxShadow: "var(--token-elevation-high-box-shadow)",
          border: "1px solid var(--token-color-border-faint)",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 10, padding: "16px 18px", borderBottom: "1px solid var(--token-color-border-faint)" }}>
          <div style={{ flex: 1 }}>
            <div style={{ fontSize: 16, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>{title}</div>
            <div style={{ fontSize: 12, color: "var(--token-color-foreground-faint)", marginTop: 2 }}>{cca.name}</div>
          </div>
          <IconButton icon="x" title="Close" onClick={onClose} />
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 18, padding: 18 }}>{body}</div>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 8,
            padding: "14px 18px",
            borderTop: "1px solid var(--token-color-border-faint)",
            background: "var(--token-color-surface-faint)",
            borderRadius: "0 0 12px 12px",
            flexWrap: "wrap",
          }}
        >
          {footerLeft}
          <span style={{ flex: 1 }} />
          <Button color="ghost" icon="x" onClick={onClose}>
            Cancel
          </Button>
          {save}
        </div>
      </div>
    </div>
  );
}

function PositionTile({ position, held, on, onClick }: { position: DirectoryPosition; held: number; on: boolean; onClick: () => void }) {
  const closed = position.capacity === 0;
  const full = isFull(position, held);
  const text = closed ? "Closed" : full ? `Full · ${capacityText(position, held)}` : held ? `${capacityText(position, held)} filled` : `Vacant · ${capacityText(position, held)}`;
  return (
    <button
      disabled={full}
      onClick={onClick}
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "flex-start",
        gap: 4,
        padding: "9px 12px",
        borderRadius: 8,
        textAlign: "left",
        font: "inherit",
        cursor: full ? "not-allowed" : "pointer",
        opacity: full ? 0.55 : 1,
        border: "1px solid " + (on ? "var(--cupid)" : "var(--token-color-border-strong)"),
        background: on ? "var(--cupid-soft)" : "var(--token-color-surface-primary)",
        boxShadow: on ? "inset 0 0 0 1px var(--cupid)" : "none",
      }}
    >
      <span style={{ ...ellipsis, maxWidth: "100%", fontSize: 13, fontWeight: 700, color: on ? "var(--cupid-strong)" : "var(--token-color-foreground-strong)" }}>
        {position.name}
      </span>
      <span style={{ display: "flex", alignItems: "center", gap: 6 }}>
        <DirectoryTypeBadge type={position.positionType} />
        <span style={{ fontSize: 11.5, color: held ? "var(--token-color-foreground-faint)" : "var(--token-color-foreground-warning-on-surface)" }}>{text}</span>
      </span>
    </button>
  );
}

// ---- atoms -----------------------------------------------------------
const TYPE_BADGE: Partial<Record<DirectoryPositionType, BadgeColor>> = { lead: "highlight", vice: "action", "team-manager": "success" };

function DirectoryTypeBadge({ type }: { type: DirectoryPositionType }) {
  return <Badge color={TYPE_BADGE[type] ?? "neutral"} text={typeLabel(type)} />;
}

function SearchField({ placeholder, value, onChange }: { placeholder: string; value: string; onChange: (v: string) => void }) {
  return (
    <TextInput
      icon={<Icon name="search" size={16} color="var(--token-color-foreground-faint)" />}
      placeholder={placeholder}
      value={value}
      onChange={(e) => onChange(e.target.value)}
    />
  );
}

function IconButton({ icon, title, onClick }: { icon: string; title: string; onClick: (e: MouseEvent) => void }) {
  return (
    <button
      title={title}
      aria-label={title}
      onClick={onClick}
      style={{
        width: 30,
        height: 30,
        flexShrink: 0,
        borderRadius: 6,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        cursor: "pointer",
        border: "1px solid transparent",
        background: "transparent",
      }}
    >
      <Icon name={icon} size={15} color="var(--token-color-foreground-faint)" />
    </button>
  );
}

function FieldLabel({ children }: { children: ReactNode }) {
  return <div style={{ fontSize: 11.5, fontWeight: 700, color: "var(--token-color-foreground-strong)", marginBottom: 7 }}>{children}</div>;
}

function ChoiceChip({ label, on, onClick }: { label: string; on: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      style={{
        height: 30,
        padding: "0 12px",
        borderRadius: 7,
        font: "inherit",
        fontSize: 12.5,
        fontWeight: 600,
        cursor: "pointer",
        whiteSpace: "nowrap",
        border: "1px solid " + (on ? "var(--cupid-line)" : "var(--token-color-border-strong)"),
        background: on ? "var(--cupid-soft)" : "var(--token-color-surface-primary)",
        color: on ? "var(--cupid-strong)" : "var(--token-color-foreground-strong)",
      }}
    >
      {label}
    </button>
  );
}

function CriticalButton({ label, onClick, disabled, solid }: { label: string; onClick: () => void; disabled?: boolean; solid?: boolean }) {
  const fg = solid ? "#fff" : "var(--token-color-foreground-critical-on-surface)";
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 7,
        height: 34,
        padding: "0 12px",
        borderRadius: 7,
        font: "inherit",
        fontSize: 12.5,
        fontWeight: 600,
        whiteSpace: "nowrap",
        cursor: disabled ? "not-allowed" : "pointer",
        opacity: disabled ? 0.5 : 1,
        border: "1px solid " + (solid ? "var(--token-color-foreground-critical)" : "var(--token-color-border-critical)"),
        background: solid ? "var(--token-color-foreground-critical)" : "var(--token-color-surface-critical)",
        color: fg,
      }}
    >
      <Icon name="trash" size={14} color={fg} />
      {label}
    </button>
  );
}

const pageStyle = (maxWidth: number) => ({ padding: "24px 28px 48px", maxWidth, margin: "0 auto" }) as const;
const ellipsis = { whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" } as const;
const rowStyle = {
  display: "flex",
  alignItems: "center",
  gap: 12,
  minHeight: 52,
  padding: "8px 8px 8px 16px",
  borderTop: "1px solid var(--token-color-border-faint)",
} as const;
const summaryStyle = {
  display: "flex",
  alignItems: "center",
  gap: 8,
  padding: "10px 12px",
  borderRadius: 8,
  background: "var(--token-color-surface-faint)",
  border: "1px solid var(--token-color-border-faint)",
} as const;
const backButtonStyle = {
  display: "flex",
  whiteSpace: "nowrap",
  flexShrink: 0,
  alignItems: "center",
  gap: 6,
  height: 32,
  padding: "0 12px 0 9px",
  borderRadius: 7,
  border: "1px solid var(--token-color-border-strong)",
  background: "var(--token-color-surface-primary)",
  cursor: "pointer",
  font: "inherit",
  fontSize: 12.5,
  fontWeight: 600,
  color: "var(--token-color-foreground-strong)",
  marginBottom: 16,
} as const;
