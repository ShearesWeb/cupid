// CapSettings.tsx — this academic year's CCA caps and committee types. Edits
// are a draft until Save changes; Rust validates and saves them to the local
// settings file, and every cap check, tag and warning then uses them. The
// page never decides a cap itself: it only edits CapRules.
//
// Committee types are a drag-and-drop board (Type A / Type B columns, with a
// "Not set" tray); every card also has a move button for keyboard use.
// Committee caps are lists of allowed Main + Sub combinations, written the way
// the white paper lists them.
import { useState, type DragEvent, type ReactNode } from "react";
import { Button, Card, Icon, TextInput } from "../components/index.ts";
import { committeeCombos, effectiveType } from "../lib/directory.ts";
import type { CapRules, CommitteeCombo, DirectoryCca, DirectorySnapshot } from "../lib/types.ts";
import { PageTitle } from "./shared.tsx";

export interface CapSettingsProps {
  directory: DirectorySnapshot;
  onSave: (rules: CapRules) => Promise<boolean>;
  toast: (kind: "success" | "error", text: string) => void;
}

/** The AY26/27 white paper, offered as a starting point. Committee types are kept. */
const WHITE_PAPER: Omit<CapRules, "academicYear" | "ccaTypes"> = {
  committeeCombos: [
    { main: 2, sub: 0 },
    { main: 1, sub: 1 },
    { main: 0, sub: 3 },
  ],
  committeeCombosAppealed: [
    { main: 2, sub: 1 },
    { main: 1, sub: 2 },
    { main: 0, sub: 4 },
  ],
  sportsCultureMax: 5,
  sportsCultureMaxAppealed: 6,
  totalMax: 6,
  totalMaxAppealed: 7,
  totalMaxBothAppeals: 8,
};

const MAX_COMBOS = 10;
const MAX_VALUE = 50;
const clamp = (n: number) => Math.max(0, Math.min(MAX_VALUE, Number.isFinite(n) ? Math.trunc(n) : 0));

type Column = "type-a" | "type-b";

export function CapSettings({ directory, onSave, toast }: CapSettingsProps) {
  const saved = directory.committeeCaps.rules;
  const [draft, setDraft] = useState<CapRules>(saved);
  const [saving, setSaving] = useState(false);
  const dirty = JSON.stringify(draft) !== JSON.stringify(saved);
  const patch = (p: Partial<CapRules>) => setDraft((d) => ({ ...d, ...p }));

  const save = async () => {
    setSaving(true);
    if (await onSave(draft)) toast("success", `Caps saved${draft.academicYear ? ` for ${draft.academicYear}` : ""}. Every check now uses them.`);
    setSaving(false);
  };

  return (
    <div style={{ padding: "24px 28px 48px", maxWidth: 960, margin: "0 auto" }}>
      <PageTitle
        title="Cap settings"
        sub="This academic year's CCA caps and committee types. Every cap check, tag and warning uses them. They apply to every resident and are saved on this computer only."
      />

      {dirty ? (
        <div style={saveBarStyle}>
          <Icon name="edit" size={14} color="var(--cupid-strong)" />
          <span style={{ flex: 1, fontSize: 12.5, fontWeight: 600, color: "var(--token-color-foreground-strong)" }}>Unsaved changes</span>
          <Button color="ghost" icon="x" disabled={saving} onClick={() => setDraft(saved)}>
            Discard
          </Button>
          <Button color="primary" icon="check" busy={saving} onClick={() => void save()}>
            Save changes
          </Button>
        </div>
      ) : null}

      <Block title="Academic year" sub="A label for these caps, shown on the CCA caps page.">
        <div style={{ display: "flex", alignItems: "center", gap: 10, flexWrap: "wrap" }}>
          <div style={{ width: 200 }}>
            <TextInput placeholder="e.g. AY27/28" value={draft.academicYear} onChange={(e) => patch({ academicYear: e.target.value.slice(0, 40) })} />
          </div>
          <Button color="ghost" icon="layers" onClick={() => patch(WHITE_PAPER)}>
            Use AY26/27 white paper caps
          </Button>
        </div>
      </Block>

      <Block
        title="Committee types"
        sub="Drag each committee into Type A or Type B. Type A Sub-Comm seats count as Sub and everything else as Main; every Type B seat counts as Main."
      >
        <TypeBoard committees={directory.ccas.filter((c) => c.kind === "committee")} draft={draft} onChange={(ccaTypes) => patch({ ccaTypes })} />
      </Block>

      <Block
        title="Committee caps"
        sub="List the Main + Sub holdings a resident may have. Anything that fits inside one of them is allowed: with 2M listed, 1M is fine too."
      >
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap: 16 }}>
          <ComboList title="Without appeal" combos={draft.committeeCombos} onChange={(c) => patch({ committeeCombos: c })} />
          <ComboList title="With committee appeal" combos={draft.committeeCombosAppealed} onChange={(c) => patch({ committeeCombosAppealed: c })} />
        </div>
      </Block>

      <Block title="Sports & Culture cap" sub="The most Sports and Cultural CCAs a resident may hold. Each CCA counts once; team managers count.">
        <div style={{ display: "flex", gap: 16, flexWrap: "wrap" }}>
          <NumberField label="Without appeal" value={draft.sportsCultureMax} onChange={(n) => patch({ sportsCultureMax: n })} />
          <NumberField label="With CCA cap appeal" value={draft.sportsCultureMaxAppealed} onChange={(n) => patch({ sportsCultureMaxAppealed: n })} />
        </div>
      </Block>

      <Block title="Total CCAs" sub="The most CCAs one resident may hold: committee seats plus Sports/Cultural CCAs.">
        <div style={{ display: "flex", gap: 16, flexWrap: "wrap" }}>
          <NumberField label="Without appeal" value={draft.totalMax} onChange={(n) => patch({ totalMax: n })} />
          <NumberField label="With CCA cap appeal" value={draft.totalMaxAppealed} onChange={(n) => patch({ totalMaxAppealed: n })} />
          <NumberField label="With both appeals" value={draft.totalMaxBothAppeals} onChange={(n) => patch({ totalMaxBothAppeals: n })} />
        </div>
      </Block>
    </div>
  );
}

// ---- committee types: drag and drop -----------------------------------
function TypeBoard({ committees, draft, onChange }: { committees: DirectoryCca[]; draft: CapRules; onChange: (t: CapRules["ccaTypes"]) => void }) {
  const [search, setSearch] = useState("");
  const [over, setOver] = useState<Column | null>(null);
  const [dragging, setDragging] = useState<number | null>(null);

  const sorted = [...committees].sort((a, b) => a.name.localeCompare(b.name));
  const q = search.trim().toLowerCase();
  const matches = (c: DirectoryCca) => !q || c.name.toLowerCase().includes(q);
  const typeOf = (c: DirectoryCca) => effectiveType(draft, c);
  const setType = (c: DirectoryCca, type: Column | null) => {
    const next = { ...draft.ccaTypes };
    // Choosing the database's own type is the same as no setting.
    if (type === null || type === c.type) delete next[String(c.id)];
    else next[String(c.id)] = type;
    onChange(next);
  };

  const drop = (column: Column) => (e: DragEvent) => {
    e.preventDefault();
    setOver(null);
    const cca = committees.find((c) => c.id === Number(e.dataTransfer.getData("text/plain")));
    if (cca) setType(cca, column);
  };
  const unset = sorted.filter((c) => typeOf(c) === "none");
  const card = (c: DirectoryCca) => (
    <TypeCard
      key={c.id}
      cca={c}
      type={typeOf(c)}
      changed={String(c.id) in draft.ccaTypes}
      dimmed={!matches(c)}
      dragging={dragging === c.id}
      onDragStart={() => setDragging(c.id)}
      onDragEnd={() => {
        setDragging(null);
        setOver(null);
      }}
      onMove={(t) => setType(c, t)}
    />
  );

  return (
    <>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 12, flexWrap: "wrap", marginBottom: 12 }}>
        <span style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)" }}>
          Changes show a dot; the database's type is on the card's hover text.
        </span>
        <div style={{ width: 240 }}>
          <TextInput
            icon={<Icon name="search" size={16} color="var(--token-color-foreground-faint)" />}
            placeholder="Find a committee…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
      </div>
      {unset.length ? (
        <div style={{ ...zoneStyle(false), marginBottom: 12, borderStyle: "dashed", borderColor: "var(--token-color-border-warning)", background: "var(--token-color-surface-warning)" }}>
          <ZoneHead title="Not set" count={unset.length} hint="Drag into Type A or Type B" />
          <div style={chipWrap}>{unset.map(card)}</div>
        </div>
      ) : null}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(260px, 1fr))", gap: 12 }}>
        {(["type-a", "type-b"] as const).map((column) => {
          const items = sorted.filter((c) => typeOf(c) === column);
          return (
            <div
              key={column}
              onDragOver={(e) => {
                e.preventDefault();
                e.dataTransfer.dropEffect = "move";
                if (over !== column) setOver(column);
              }}
              onDragLeave={(e) => {
                if (!e.currentTarget.contains(e.relatedTarget as Node | null)) setOver(null);
              }}
              onDrop={drop(column)}
              style={zoneStyle(over === column)}
            >
              <ZoneHead
                title={column === "type-a" ? "Type A" : "Type B"}
                count={items.length}
                hint={column === "type-a" ? "Main-Comm + Sub-Comms" : "All members count as Main"}
              />
              <div style={chipWrap}>
                {items.length ? items.map(card) : <span style={{ fontSize: 12, color: "var(--token-color-foreground-faint)", fontStyle: "italic" }}>Drop committees here</span>}
              </div>
            </div>
          );
        })}
      </div>
    </>
  );
}

function ZoneHead({ title, count, hint }: { title: string; count: number; hint: string }) {
  return (
    <div style={{ display: "flex", alignItems: "baseline", gap: 8, marginBottom: 10 }}>
      <span style={{ fontSize: 13, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>{title}</span>
      <span style={{ fontSize: 12, fontWeight: 600, color: "var(--token-color-foreground-faint)" }}>{count}</span>
      <span style={{ flex: 1 }} />
      <span style={{ fontSize: 11.5, color: "var(--token-color-foreground-faint)" }}>{hint}</span>
    </div>
  );
}

function TypeCard({
  cca,
  type,
  changed,
  dimmed,
  dragging,
  onDragStart,
  onDragEnd,
  onMove,
}: {
  cca: DirectoryCca;
  type: "none" | Column;
  changed: boolean;
  dimmed: boolean;
  dragging: boolean;
  onDragStart: () => void;
  onDragEnd: () => void;
  onMove: (t: Column | null) => void;
}) {
  const fromDb = cca.type === "type-a" ? "Type A" : cca.type === "type-b" ? "Type B" : "not set";
  // Keyboard and touch alternative to dragging: move to the other column.
  const targets: Column[] = type === "none" ? ["type-a", "type-b"] : [type === "type-a" ? "type-b" : "type-a"];
  return (
    <div
      draggable
      onDragStart={(e) => {
        e.dataTransfer.setData("text/plain", String(cca.id));
        e.dataTransfer.effectAllowed = "move";
        onDragStart();
      }}
      onDragEnd={onDragEnd}
      title={`${cca.name}: database says ${fromDb}${changed ? "; changed here" : ""}`}
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 6,
        height: 30,
        padding: "0 4px 0 9px",
        borderRadius: 7,
        cursor: "grab",
        userSelect: "none",
        fontSize: 12.5,
        fontWeight: 600,
        color: "var(--token-color-foreground-strong)",
        background: "var(--token-color-surface-primary)",
        border: `1px solid ${changed ? "var(--cupid-line)" : "var(--token-color-border-strong)"}`,
        boxShadow: "var(--token-surface-base-box-shadow)",
        opacity: dragging ? 0.4 : dimmed ? 0.35 : 1,
      }}
    >
      {changed ? <span aria-label="Changed" style={{ width: 6, height: 6, borderRadius: "50%", background: "var(--cupid)" }} /> : null}
      <span style={{ whiteSpace: "nowrap" }}>{cca.name}</span>
      {targets.map((t) => (
        <button key={t} aria-label={`Move ${cca.name} to ${t === "type-a" ? "Type A" : "Type B"}`} onClick={() => onMove(t)} style={miniButton}>
          {t === "type-a" ? "A" : "B"}
        </button>
      ))}
      {changed ? (
        <button aria-label={`Reset ${cca.name} to the database's type`} title="Back to the database's type" onClick={() => onMove(null)} style={miniButton}>
          ↺
        </button>
      ) : null}
    </div>
  );
}

// ---- committee caps: combinations -------------------------------------
function ComboList({ title, combos, onChange }: { title: string; combos: CommitteeCombo[]; onChange: (c: CommitteeCombo[]) => void }) {
  const set = (i: number, p: Partial<CommitteeCombo>) => onChange(combos.map((c, j) => (j === i ? { ...c, ...p } : c)));
  return (
    <div style={{ border: "1px solid var(--token-color-border-faint)", borderRadius: 8, padding: 12 }}>
      <div style={{ fontSize: 12.5, fontWeight: 700, color: "var(--token-color-foreground-strong)", marginBottom: 10 }}>{title}</div>
      <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
        {combos.map((c, i) => (
          <div key={i} style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
            <input
              type="number"
              min={0}
              max={MAX_VALUE}
              aria-label={`${title}, combination ${i + 1}: Main seats`}
              value={c.main}
              onChange={(e) => set(i, { main: clamp(e.target.valueAsNumber) })}
              style={numberStyle}
            />
            <span style={unitStyle}>Main</span>
            <span style={{ color: "var(--token-color-foreground-faint)" }}>+</span>
            <input
              type="number"
              min={0}
              max={MAX_VALUE}
              aria-label={`${title}, combination ${i + 1}: Sub seats`}
              value={c.sub}
              onChange={(e) => set(i, { sub: clamp(e.target.valueAsNumber) })}
              style={numberStyle}
            />
            <span style={unitStyle}>Sub</span>
            <span style={{ flex: 1 }} />
            <button
              aria-label={`Remove combination ${i + 1}`}
              disabled={combos.length <= 1}
              onClick={() => onChange(combos.filter((_, j) => j !== i))}
              style={{ ...miniButton, width: 28, height: 28, opacity: combos.length <= 1 ? 0.4 : 1 }}
            >
              ✕
            </button>
          </div>
        ))}
      </div>
      <button disabled={combos.length >= MAX_COMBOS} onClick={() => onChange([...combos, { main: 0, sub: 0 }])} style={{ ...linkStyle, marginTop: 10 }}>
        + Add combination
      </button>
      <div style={{ fontSize: 11.5, color: "var(--token-color-foreground-faint)", marginTop: 8 }}>
        Allows <b style={{ color: "var(--token-color-foreground-strong)" }}>{committeeCombos(combos)}</b>, or anything smaller
      </div>
    </div>
  );
}

// ---- shared bits ---------------------------------------------------------
function Block({ title, sub, children }: { title: string; sub: string; children: ReactNode }) {
  return (
    <Card padding="large" style={{ marginBottom: 16 }}>
      <div style={{ fontSize: 15, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>{title}</div>
      <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)", margin: "3px 0 14px", maxWidth: 640 }}>{sub}</div>
      {children}
    </Card>
  );
}

function NumberField({ label, value, onChange }: { label: string; value: number; onChange: (n: number) => void }) {
  return (
    <label style={{ display: "flex", flexDirection: "column", gap: 6, fontSize: 12, fontWeight: 600, color: "var(--token-color-foreground-strong)" }}>
      {label}
      <input type="number" min={0} max={MAX_VALUE} value={value} onChange={(e) => onChange(clamp(e.target.valueAsNumber))} style={numberStyle} />
    </label>
  );
}

const zoneStyle = (active: boolean) =>
  ({
    padding: 12,
    minHeight: 120,
    borderRadius: 10,
    border: `1.5px ${active ? "solid" : "dashed"} ${active ? "var(--cupid)" : "var(--token-color-border-strong)"}`,
    background: active ? "var(--cupid-soft)" : "var(--token-color-surface-faint)",
    transition: "background 120ms, border-color 120ms",
  }) as const;

const chipWrap = { display: "flex", flexWrap: "wrap", gap: 6 } as const;

const miniButton = {
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  minWidth: 22,
  height: 22,
  padding: "0 5px",
  borderRadius: 5,
  border: "1px solid var(--token-color-border-faint)",
  background: "var(--token-color-surface-faint)",
  color: "var(--token-color-foreground-faint)",
  font: "inherit",
  fontSize: 11,
  fontWeight: 700,
  cursor: "pointer",
} as const;

const unitStyle = { fontSize: 12.5, color: "var(--token-color-foreground-faint)" } as const;

const numberStyle = {
  width: 72,
  height: 32,
  boxSizing: "border-box",
  padding: "0 10px",
  borderRadius: 7,
  border: "1px solid var(--token-color-border-strong)",
  background: "var(--token-color-surface-primary)",
  color: "var(--token-color-foreground-strong)",
  font: "inherit",
  fontSize: 13,
} as const;

const linkStyle = {
  border: "none",
  background: "transparent",
  padding: 0,
  font: "inherit",
  fontSize: 12.5,
  fontWeight: 600,
  cursor: "pointer",
  color: "var(--token-color-foreground-action)",
} as const;

const saveBarStyle = {
  position: "sticky",
  top: 0,
  zIndex: 5,
  display: "flex",
  alignItems: "center",
  gap: 8,
  flexWrap: "wrap",
  padding: "10px 12px",
  marginBottom: 16,
  borderRadius: 8,
  background: "var(--token-color-surface-primary)",
  border: "1px solid var(--cupid-line)",
  boxShadow: "var(--token-surface-base-box-shadow)",
} as const;
