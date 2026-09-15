import { useState, type ReactNode } from "react";
import { Card, Icon } from "../components/index.ts";
import type {
  CommitmentPeriod,
  DirectoryAppointment,
  DirectoryCca,
  DirectoryPosition,
  DirectoryPositionType,
  DirectorySnapshot,
} from "../lib/types.ts";

export interface CcasProps {
  directory: DirectorySnapshot;
  onAdd: (userId: number, positionId: number, period: string) => Promise<boolean>;
  onRemove: (userId: number, positionId: number) => Promise<boolean>;
  onUpdatePeriod: (userId: number, positionId: number, period: string) => Promise<boolean>;
}

export function Ccas({ directory, onAdd, onRemove, onUpdatePeriod }: CcasProps) {
  const [activeKind, setActiveKind] = useState<string | null>(null);
  const [selectedCcaId, setSelectedCcaId] = useState<number | null>(null);
  const categories = [...new Set(directory.ccas.map((cca) => cca.kind))].sort();
  const visibleCcas = activeKind === null ? directory.ccas : directory.ccas.filter((cca) => cca.kind === activeKind);
  const selectedCca = directory.ccas.find((cca) => cca.id === selectedCcaId) ?? null;

  if (selectedCca) {
    return <CcaDetail cca={selectedCca} directory={directory} onBack={() => setSelectedCcaId(null)} onAdd={onAdd} onRemove={onRemove} onUpdatePeriod={onUpdatePeriod} />;
  }

  return (
    <div style={pageStyle}>
      <div style={{ marginBottom: 18 }}>
        <h1 style={headingStyle}>CCAs</h1>
        <p style={subheadingStyle}>Browse the complete CCA directory, including CCAs without positions.</p>
      </div>
      {directory.ccas.length ? (
        <div role="tablist" aria-label="CCA categories" style={tabsStyle}>
          <CcaTab label="All CCAs" active={activeKind === null} onClick={() => setActiveKind(null)} />
          {categories.map((category) => (
            <CcaTab key={category} label={categoryLabel(category)} active={activeKind === category} onClick={() => setActiveKind(category)} />
          ))}
        </div>
      ) : null}
      {visibleCcas.length ? (
        <div style={gridStyle}>
          {visibleCcas.map((cca) => {
            const positionCount = directory.positions.filter((position) => position.ccaId === cca.id).length;
            return <CcaCard key={cca.id} cca={cca} positionCount={positionCount} onClick={() => setSelectedCcaId(cca.id)} />;
          })}
        </div>
      ) : <Card padding="large"><div style={mutedStyle}>No CCAs available.</div></Card>}
    </div>
  );
}

function CcaCard({ cca, positionCount, onClick }: { cca: DirectoryCca; positionCount: number; onClick: () => void }) {
  return (
    <Card padding="none" style={{ border: "1px solid var(--token-color-border-faint)" }}>
      <button type="button" onClick={onClick} style={cardButtonStyle}>
        <span style={iconTileStyle}><Icon name="folder" size={16} /></span>
        <span style={{ flex: 1, minWidth: 0 }}>
          <span style={cardTitleStyle}>{cca.name}</span>
          <span style={cardMetaStyle}>{categoryLabel(cca.kind)} · {positionCount} position{positionCount === 1 ? "" : "s"}</span>
        </span>
        <Icon name="chevron-right" size={16} color="var(--token-color-foreground-faint)" />
      </button>
    </Card>
  );
}

function CcaDetail({ cca, directory, onBack, onAdd, onRemove, onUpdatePeriod }: { cca: DirectoryCca; directory: DirectorySnapshot; onBack: () => void; onAdd: CcasProps["onAdd"]; onRemove: CcasProps["onRemove"]; onUpdatePeriod: CcasProps["onUpdatePeriod"] }) {
  const positions = directory.positions.filter((position) => position.ccaId === cca.id).sort((a, b) => a.id - b.id);
  const appointmentsByPosition = new Map<number, DirectoryAppointment[]>();
  for (const appointment of directory.appointments) {
    const list = appointmentsByPosition.get(appointment.positionId);
    if (list) list.push(appointment);
    else appointmentsByPosition.set(appointment.positionId, [appointment]);
  }
  const userById = new Map(directory.users.map((user) => [user.id, user]));
  const positionById = new Map(directory.positions.map((position) => [position.id, position]));

  return (
    <div style={pageStyle}>
      <button type="button" onClick={onBack} style={backButtonStyle}><Icon name="chevron-left" size={15} /> Back to CCAs</button>
      <div style={{ display: "flex", alignItems: "flex-start", gap: 14, marginBottom: 18 }}>
        {cca.imageUrl ? (
          <img src={cca.imageUrl} alt="" style={ccaImageStyle} />
        ) : (
          <span style={largeIconTileStyle}><Icon name="folder" size={20} /></span>
        )}
        <div>
          <div style={eyebrowStyle}>{categoryLabel(cca.kind)} · {cca.tier} · {cca.type}</div>
          <h1 style={headingStyle}>{cca.name}</h1>
          <p style={subheadingStyle}>{cca.description ?? "No description available."}</p>
        </div>
      </div>
      <Section title={`Positions (${positions.length})`}>
        {positions.length ? positions.map((position) => (
          <PositionRow key={position.id} position={position} appointments={appointmentsByPosition.get(position.id) ?? []} users={directory.users} userById={userById} positionById={positionById} onAdd={onAdd} onRemove={onRemove} onUpdatePeriod={onUpdatePeriod} />
        )) : <div style={mutedStyle}>This CCA has no positions.</div>}
      </Section>
    </div>
  );
}

function PositionRow({ position, appointments, users, userById, positionById, onAdd, onRemove, onUpdatePeriod }: { position: DirectoryPosition; appointments: DirectoryAppointment[]; users: { id: number; name: string; email: string }[]; userById: Map<number, { name: string; email: string }>; positionById: Map<number, DirectoryPosition>; onAdd: CcasProps["onAdd"]; onRemove: CcasProps["onRemove"]; onUpdatePeriod: CcasProps["onUpdatePeriod"] }) {
  const reporting = position.reportingPositionId ? positionById.get(position.reportingPositionId) : null;
  const [selectedPeriod, setSelectedPeriod] = useState<CommitmentPeriod>("full-year");
  const editable = position.positionType !== "resident";
  const heldUserIds = new Set(appointments.map((appointment) => appointment.userId));
  const candidates = users.filter((user) => !heldUserIds.has(user.id));
  const [selectedUserId, setSelectedUserId] = useState<number | null>(null);
  const [memberQuery, setMemberQuery] = useState("");

  const addSelectedMember = async () => {
    if (selectedUserId === null) return;
    if (await onAdd(selectedUserId, position.id, selectedPeriod)) {
      setSelectedUserId(null);
      setMemberQuery("");
    }
  };
  return (
    <div style={positionStyle}>
      <div>
        <div style={cardTitleStyle}>{position.name}</div>
        <div style={positionMetaStyle}>
          <span>{positionTypeLabel(position.positionType)}</span>
          <span>{capacityLabel(position.capacity)}</span>
          {reporting ? <span>Reports to {reporting.name}</span> : null}
        </div>
      </div>
      {position.description ? <div style={{ marginTop: 7, fontSize: 12, color: "var(--token-color-foreground-faint)" }}>{position.description}</div> : null}
      <div style={{ marginTop: 10 }}>
        {appointments.length ? appointments.map((appointment) => {
          const user = userById.get(appointment.userId);
          return (
            <div key={`${appointment.userId}-${appointment.positionId}`} style={holderRowStyle}>
              <Icon name={position.positionType === "member" || position.positionType === "resident" ? "users" : "check-circle"} size={15} color="var(--token-color-foreground-faint)" />
              <span style={{ flex: 1, minWidth: 0 }}>
                <span style={holderNameStyle}>{user?.name ?? `User ${appointment.userId}`}</span>
                <span style={holderEmailStyle}>{user?.email ?? ""}</span>
              </span>
              {editable ? (
                <select value={appointment.commitmentPeriod} onChange={(event) => void onUpdatePeriod(appointment.userId, appointment.positionId, event.target.value)} style={periodSelectStyle}>
                  {periods.map((period) => <option key={period} value={period}>{periodLabel(period)}</option>)}
                </select>
              ) : <span style={periodStyle}>{periodLabel(appointment.commitmentPeriod)}</span>}
              {editable ? <button type="button" onClick={() => void onRemove(appointment.userId, appointment.positionId)} style={removeButtonStyle}>Remove</button> : null}
            </div>
          );
        }) : <div style={mutedStyle}>No current holders.</div>}
      </div>
      {editable ? (
        <div style={editRowStyle}>
          <MemberPicker
            candidates={candidates}
            query={memberQuery}
            selectedUserId={selectedUserId}
            onQueryChange={(query) => {
              setMemberQuery(query);
              if (selectedUserId !== null && !query) setSelectedUserId(null);
            }}
            onSelect={(user) => {
              setSelectedUserId(user.id);
              setMemberQuery(user.name);
            }}
          />
          <select value={selectedPeriod} onChange={(event) => setSelectedPeriod(event.target.value as CommitmentPeriod)} style={periodSelectStyle} disabled={!candidates.length}>
            {periods.map((period) => <option key={period} value={period}>{periodLabel(period)}</option>)}
          </select>
          <button type="button" onClick={() => void addSelectedMember()} style={addButtonStyle} disabled={selectedUserId === null}>Add holder</button>
        </div>
      ) : null}
    </div>
  );
}

function MemberPicker({ candidates, query, selectedUserId, onQueryChange, onSelect }: { candidates: { id: number; name: string; email: string }[]; query: string; selectedUserId: number | null; onQueryChange: (query: string) => void; onSelect: (user: { id: number; name: string; email: string }) => void }) {
  const [open, setOpen] = useState(false);
  const normalizedQuery = query.trim().toLowerCase();
  const matches = candidates
    .filter((user) => !normalizedQuery || `${user.name} ${user.email}`.toLowerCase().includes(normalizedQuery))
    .slice(0, 8);
  const selected = candidates.find((user) => user.id === selectedUserId);

  return (
    <div style={pickerStyle}>
      <div style={pickerInputWrapStyle}>
        <Icon name="search" size={14} color="var(--token-color-foreground-faint)" />
        <input
          value={query}
          placeholder={selected ? selected.name : "Search members by name or email"}
          onChange={(event) => onQueryChange(event.target.value)}
          onFocus={() => setOpen(true)}
          onBlur={() => window.setTimeout(() => setOpen(false), 120)}
          style={pickerInputStyle}
          aria-label="Search members"
          disabled={!candidates.length}
        />
      </div>
      {open && candidates.length ? (
        <div style={pickerMenuStyle} role="listbox">
          {matches.length ? matches.map((user) => (
            <button
              key={user.id}
              type="button"
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => { onSelect(user); setOpen(false); }}
              style={pickerOptionStyle}
            >
              <span style={pickerNameStyle}>{user.name}</span>
              <span style={pickerEmailStyle}>{user.email}</span>
            </button>
          )) : <div style={pickerEmptyStyle}>No matching members</div>}
        </div>
      ) : null}
    </div>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return <Card padding="large"><h2 style={sectionTitleStyle}>{title}</h2>{children}</Card>;
}

function CcaTab({ label, active, onClick }: { label: string; active: boolean; onClick: () => void }) {
  return <button type="button" role="tab" aria-selected={active} onClick={onClick} style={{ ...tabStyle, borderColor: active ? "var(--token-color-border-action)" : "var(--token-color-border-faint)", background: active ? "var(--token-color-surface-action)" : "var(--token-color-surface-primary)", color: active ? "var(--token-color-foreground-action)" : "var(--token-color-foreground-faint)" }}>{label}</button>;
}

function categoryLabel(category: string): string { return category === "jcrc" ? "JCRC" : category.charAt(0).toUpperCase() + category.slice(1); }
function positionTypeLabel(type: DirectoryPositionType): string { return type.replace("comm", "").replace("-", " ").replace(/\b\w/g, (letter) => letter.toUpperCase()); }
function capacityLabel(capacity: number | null): string { return capacity === null ? "Unlimited" : capacity === 0 ? "Closed" : `${capacity} seat${capacity === 1 ? "" : "s"}`; }
function periodLabel(period: CommitmentPeriod): string { return period.replace("semester-", "Semester ").replace("full-year", "Full year").replace("ex-shearite", "Ex-Shearite"); }
const periods: CommitmentPeriod[] = ["semester-1", "semester-2", "full-year", "ex-shearite"];

const pageStyle = { padding: "24px 28px 48px", maxWidth: 1120, margin: "0 auto" } as const;
const headingStyle = { margin: "0 0 4px", fontSize: 22, fontWeight: 700, letterSpacing: "-0.4px", color: "var(--token-color-foreground-strong)" } as const;
const subheadingStyle = { margin: 0, fontSize: 13, color: "var(--token-color-foreground-faint)" } as const;
const mutedStyle = { color: "var(--token-color-foreground-faint)", fontSize: 12.5 } as const;
const tabsStyle = { display: "flex", gap: 7, flexWrap: "wrap", marginBottom: 18 } as const;
const gridStyle = { display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(270px, 1fr))", gap: 12 } as const;
const cardButtonStyle = { display: "flex", alignItems: "center", gap: 12, width: "100%", minHeight: 82, padding: "14px 16px", border: "none", background: "transparent", cursor: "pointer", font: "inherit", textAlign: "left" } as const;
const iconTileStyle = { display: "flex", alignItems: "center", justifyContent: "center", width: 32, height: 32, flexShrink: 0, borderRadius: 8, background: "var(--token-color-surface-strong)", color: "var(--token-color-foreground-faint)" } as const;
const largeIconTileStyle = { ...iconTileStyle, width: 42, height: 42 } as const;
const ccaImageStyle = { width: 42, height: 42, flexShrink: 0, borderRadius: 8, objectFit: "cover" } as const;
const eyebrowStyle = { marginBottom: 5, fontSize: 10.5, fontWeight: 700, letterSpacing: "0.6px", textTransform: "uppercase", color: "var(--token-color-foreground-faint)" } as const;
const tabStyle = { cursor: "pointer", font: "inherit", fontSize: 12, fontWeight: 600, padding: "6px 12px", borderRadius: 7, border: "1px solid" } as const;
const backButtonStyle = { display: "flex", alignItems: "center", gap: 6, marginBottom: 16, padding: "6px 10px 6px 7px", border: "1px solid var(--token-color-border-strong)", borderRadius: 7, background: "var(--token-color-surface-primary)", color: "var(--token-color-foreground-strong)", cursor: "pointer", font: "inherit", fontSize: 12.5, fontWeight: 600 } as const;
const positionStyle = { padding: "14px 0", borderTop: "1px solid var(--token-color-border-faint)" } as const;
const positionMetaStyle = { display: "flex", gap: 8, flexWrap: "wrap", marginTop: 5, fontSize: 11.5, color: "var(--token-color-foreground-faint)" } as const;
const cardTitleStyle = { display: "block", fontSize: 14, fontWeight: 700, color: "var(--token-color-foreground-strong)", overflowWrap: "anywhere" } as const;
const cardMetaStyle = { display: "block", marginTop: 4, fontSize: 11.5, color: "var(--token-color-foreground-faint)" } as const;
const holderRowStyle = { display: "flex", alignItems: "center", gap: 9, padding: "8px 0", borderTop: "1px solid var(--token-color-border-faint)" } as const;
const holderNameStyle = { display: "block", fontSize: 12.5, fontWeight: 600, color: "var(--token-color-foreground-primary)" } as const;
const holderEmailStyle = { display: "block", marginTop: 2, fontSize: 11, color: "var(--token-color-foreground-faint)" } as const;
const periodStyle = { flexShrink: 0, padding: "4px 8px", borderRadius: 6, background: "var(--token-color-surface-faint)", color: "var(--token-color-foreground-faint)", fontSize: 11, fontWeight: 600 } as const;
const periodSelectStyle = { flexShrink: 0, maxWidth: 150, padding: "4px 6px", borderRadius: 6, border: "1px solid var(--token-color-border-faint)", background: "var(--token-color-surface-primary)", color: "var(--token-color-foreground-primary)", font: "inherit", fontSize: 11 } as const;
const editRowStyle = { display: "flex", gap: 7, flexWrap: "wrap", marginTop: 9, paddingTop: 9, borderTop: "1px dashed var(--token-color-border-faint)" } as const;
const pickerStyle = { position: "relative", flex: "1 1 260px", minWidth: 220, maxWidth: 360 } as const;
const pickerInputWrapStyle = { display: "flex", alignItems: "center", gap: 7, height: 29, padding: "0 8px", border: "1px solid var(--token-color-border-faint)", borderRadius: 6, background: "var(--token-color-surface-primary)" } as const;
const pickerInputStyle = { width: "100%", minWidth: 0, border: "none", outline: "none", background: "transparent", color: "var(--token-color-foreground-primary)", font: "inherit", fontSize: 11 } as const;
const pickerMenuStyle = { position: "absolute", zIndex: 5, top: 34, left: 0, right: 0, maxHeight: 220, overflowY: "auto", padding: 4, border: "1px solid var(--token-color-border-faint)", borderRadius: 7, background: "var(--token-color-surface-primary)", boxShadow: "0 8px 20px rgba(0, 0, 0, 0.12)" } as const;
const pickerOptionStyle = { display: "flex", flexDirection: "column", alignItems: "flex-start", width: "100%", gap: 2, padding: "7px 8px", border: "none", borderRadius: 5, background: "transparent", color: "var(--token-color-foreground-primary)", cursor: "pointer", font: "inherit", textAlign: "left" } as const;
const pickerNameStyle = { fontSize: 11.5, fontWeight: 700 } as const;
const pickerEmailStyle = { fontSize: 10.5, color: "var(--token-color-foreground-faint)" } as const;
const pickerEmptyStyle = { padding: "9px 8px", color: "var(--token-color-foreground-faint)", fontSize: 11 } as const;
const addButtonStyle = { padding: "5px 9px", border: "1px solid var(--token-color-border-action)", borderRadius: 6, background: "var(--token-color-surface-action)", color: "var(--token-color-foreground-action)", cursor: "pointer", font: "inherit", fontSize: 11, fontWeight: 700 } as const;
const removeButtonStyle = { padding: "4px 7px", border: "1px solid var(--token-color-border-faint)", borderRadius: 6, background: "transparent", color: "var(--token-color-foreground-faint)", cursor: "pointer", font: "inherit", fontSize: 10.5 } as const;
const sectionTitleStyle = { margin: "0 0 6px", fontSize: 14, color: "var(--token-color-foreground-strong)" } as const;
