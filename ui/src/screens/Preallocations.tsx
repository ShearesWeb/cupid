// Preallocations.tsx — manage operator-fixed (applicant, position) assignments.
// Preallocations persist in a local store file on this machine (never the
// database); the pair holds the position outright: it is seated before
// matching, consumes the seat and the holder's quota, and cannot be
// displaced. Granting or revoking one invalidates the current run, so
// outcome pills here fall back to pending until a re-run.
import { useState } from "react";

import { Avatar } from "../components/Avatar.tsx";
import { PositionTypeBadge } from "../components/PositionTypeBadge.tsx";
import { Button } from "../components/Button.tsx";
import { Card } from "../components/Card.tsx";
import { Icon } from "../components/Icon.tsx";
import { StatusPill } from "../components/StatusPill.tsx";
import { TextInput } from "../components/TextInput.tsx";
import type { Indexes } from "../lib/indexes";
import { pk } from "../lib/indexes";
import type { Snapshot } from "../lib/types";

export interface PreallocationsProps {
  snapshot: Snapshot;
  idx: Indexes;
  onAdd: (applicantId: number, positionId: number, note: string | null) => Promise<boolean>;
  onRemove: (applicantId: number, positionId: number) => Promise<boolean>;
  onOpenMatch: (aid: number, pid: number) => void;
  toast: (kind: "error" | "success", text: string) => void;
}

interface Option {
  id: number;
  label: string;
  sub?: string;
  badge?: "block" | "main" | "sub";
}

function Combo({
  placeholder,
  options,
  selected,
  onSelect,
}: {
  placeholder: string;
  options: (q: string) => Option[];
  selected: Option | null;
  onSelect: (o: Option | null) => void;
}) {
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const matches = open && !selected ? options(query).slice(0, 6) : [];
  return (
    <div style={{ position: "relative", flex: 1, minWidth: 220 }}>
      {selected ? (
        <button
          onClick={() => {
            onSelect(null);
            setQuery("");
          }}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 8,
            width: "100%",
            height: 34,
            padding: "0 10px",
            borderRadius: 7,
            border: "1px solid var(--token-color-border-action)",
            background: "var(--token-color-surface-action)",
            color: "var(--token-color-foreground-action)",
            cursor: "pointer",
            font: "inherit",
            fontSize: 12.5,
            fontWeight: 600,
          }}
        >
          <span style={{ flex: 1, textAlign: "left", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {selected.label}
          </span>
          <Icon name="x" size={14} color="var(--token-color-foreground-action)" />
        </button>
      ) : (
        <div onBlur={() => setTimeout(() => setOpen(false), 120)}>
          <TextInput
            placeholder={placeholder}
            icon={<Icon name="search" size={16} color="var(--token-color-foreground-faint)" />}
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              // Open only while the operator is typing; an empty field
              // (including right after a grant resets the form) stays closed.
              setOpen(e.target.value.trim().length > 0);
            }}
          />
        </div>
      )}
      {matches.length > 0 ? (
        <div
          style={{
            position: "absolute",
            top: 38,
            left: 0,
            right: 0,
            zIndex: 40,
            borderRadius: 9,
            background: "var(--token-color-surface-primary)",
            boxShadow: "var(--token-elevation-high-box-shadow)",
            border: "1px solid var(--token-color-border-faint)",
            overflow: "hidden",
          }}
        >
          {matches.map((o) => (
            <button
              key={o.id}
              onMouseDown={(e) => {
                e.preventDefault();
                onSelect(o);
                setOpen(false);
                setQuery("");
              }}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 8,
                width: "100%",
                padding: "8px 11px",
                border: "none",
                borderBottom: "1px solid var(--token-color-border-faint)",
                background: "transparent",
                cursor: "pointer",
                font: "inherit",
                fontSize: 12.5,
                textAlign: "left",
              }}
            >
              <span style={{ fontWeight: 600, color: "var(--token-color-foreground-strong)" }}>{o.label}</span>
              {o.badge ? <PositionTypeBadge type={o.badge} /> : null}
              <span
                style={{
                  marginLeft: "auto",
                  color: "var(--token-color-foreground-faint)",
                  fontSize: 11.5,
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                  maxWidth: "55%",
                }}
              >
                {o.sub}
              </span>
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}

export function Preallocations({ snapshot, idx, onAdd, onRemove, onOpenMatch, toast }: PreallocationsProps) {
  const [applicant, setApplicant] = useState<Option | null>(null);
  const [position, setPosition] = useState<Option | null>(null);
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);
  const [armed, setArmed] = useState<string | null>(null);
  const [removing, setRemoving] = useState<string | null>(null);

  const applicantOptions = (q: string): Option[] => {
    const needle = q.trim().toLowerCase();
    return snapshot.applicants
      .filter((a) => !needle || a.name.toLowerCase().includes(needle) || a.email.toLowerCase().includes(needle))
      .map((a) => ({ id: a.id, label: a.name, sub: a.email }));
  };
  const positionOptions = (q: string): Option[] => {
    const needle = q.trim().toLowerCase();
    return snapshot.positions
      .filter((p) => {
        const cca = idx.ccaById.get(p.ccaId)?.name ?? "";
        return !needle || p.name.toLowerCase().includes(needle) || cca.toLowerCase().includes(needle);
      })
      .map((p) => ({
        id: p.id,
        label: `${idx.ccaById.get(p.ccaId)?.name ?? "?"} · ${p.name}`,
        badge: p.type,
      }));
  };

  const duplicate =
    applicant && position && snapshot.preallocations.some((a) => a.applicantId === applicant.id && a.positionId === position.id);

  const grant = async () => {
    if (!applicant || !position || busy) return;
    if (idx.committedSet.has(pk(applicant.id, position.id))) {
      toast("error", `${applicant.label} already holds that position as an appointment.`);
      return;
    }
    setBusy(true);
    try {
      const ok = await onAdd(applicant.id, position.id, note.trim() ? note.trim() : null);
      if (ok) {
        setApplicant(null);
        setPosition(null);
        setNote("");
      }
    } finally {
      setBusy(false);
    }
  };

  const remove = async (aid: number, pid: number) => {
    const key = pk(aid, pid);
    setRemoving(key);
    try {
      await onRemove(aid, pid);
    } finally {
      setRemoving(null);
      setArmed(null);
    }
  };

  return (
    <div style={{ padding: "24px 28px 48px", maxWidth: 860, margin: "0 auto" }}>
      <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 4 }}>
        <h1 style={{ margin: 0, fontSize: 22, fontWeight: 700, letterSpacing: "-0.4px", color: "var(--token-color-foreground-strong)" }}>
          Preallocations
        </h1>
        <span
          style={{
            display: "inline-flex",
            alignItems: "center",
            height: 22,
            padding: "0 10px",
            borderRadius: 20,
            background: "rgba(219,42,99,0.10)",
            border: "1px solid rgba(219,42,99,0.32)",
            color: "#B91C53",
            fontSize: 11.5,
            fontWeight: 700,
          }}
        >
          {snapshot.preallocations.length} granted
        </span>
      </div>
      <p style={{ margin: "0 0 18px", fontSize: 13, color: "var(--token-color-foreground-faint)", maxWidth: 620 }}>
        Operator-fixed assignments. A preallocated resident holds the position outright: the pair is seated before
        matching, consumes the seat and quota, and cannot be displaced. Changing preallocations invalidates the current run.
      </p>

      <Card level="base" padding="none" style={{ overflow: "visible", marginBottom: 18 }}>
        <div
          style={{
            padding: "11px 16px",
            borderBottom: "1px solid var(--token-color-border-faint)",
            fontSize: 10.5,
            fontWeight: 700,
            letterSpacing: "0.7px",
            textTransform: "uppercase",
            color: "#DB2A63",
          }}
        >
          Preallocate a position
        </div>
        <div style={{ padding: 16, display: "flex", flexDirection: "column", gap: 12 }}>
          <div style={{ display: "flex", gap: 10, flexWrap: "wrap" }}>
            <Combo placeholder="Applicant name or email…" options={applicantOptions} selected={applicant} onSelect={setApplicant} />
            <span style={{ alignSelf: "center", display: "flex" }}>
              <Icon name="arrow-right" size={15} color="var(--token-color-foreground-faint)" />
            </span>
            <Combo placeholder="CCA or position…" options={positionOptions} selected={position} onSelect={setPosition} />
          </div>
          <div style={{ display: "flex", gap: 10, alignItems: "flex-end", flexWrap: "wrap" }}>
            <div style={{ flex: 1, minWidth: 260 }}>
              <TextInput
                label="Note (optional)"
                placeholder="Why does this resident hold the position outright?"
                value={note}
                onChange={(e) => setNote(e.target.value)}
              />
            </div>
            <Button color="primary" icon="plus" busy={busy} disabled={!applicant || !position || !!duplicate} onClick={grant}>
              {duplicate ? "Already granted" : "Preallocate"}
            </Button>
          </div>
        </div>
      </Card>

      {snapshot.preallocations.length === 0 ? (
        <Card
          level="base"
          padding="large"
          style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 8, padding: 48, textAlign: "center" }}
        >
          <div
            style={{
              width: 40,
              height: 40,
              borderRadius: 10,
              background: "var(--token-color-surface-strong)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <Icon name="tag" size={20} color="var(--token-color-foreground-faint)" />
          </div>
          <div style={{ fontSize: 14, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>No preallocations</div>
          <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)" }}>
            Preallocate one above to hand a resident a position outright.
          </div>
        </Card>
      ) : (
        <Card level="base" padding="none" style={{ overflow: "hidden" }}>
          {snapshot.preallocations.map((ap, i) => {
            const key = pk(ap.applicantId, ap.positionId);
            const app = idx.appById.get(ap.applicantId);
            const pos = idx.posById.get(ap.positionId);
            const cca = pos ? idx.ccaById.get(pos.ccaId) : undefined;
            const outcome = idx.outcomeByPair.get(key);
            const isArmed = armed === key;
            const isRemoving = removing === key;
            return (
              <div
                key={key}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 12,
                  padding: "11px 15px",
                  borderTop: i ? "1px solid var(--token-color-border-faint)" : "none",
                }}
              >
                <button
                  onClick={() => onOpenMatch(ap.applicantId, ap.positionId)}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 12,
                    flex: 1,
                    minWidth: 0,
                    border: "none",
                    background: "transparent",
                    cursor: "pointer",
                    font: "inherit",
                    textAlign: "left",
                    padding: 0,
                  }}
                >
                  <Avatar name={app?.name ?? "?"} size="small" />
                  <div style={{ minWidth: 0 }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 7, flexWrap: "wrap" }}>
                      <span style={{ fontSize: 13.5, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>
                        {app?.name ?? `Applicant ${ap.applicantId}`}
                      </span>
                      <Icon name="arrow-right" size={13} color="var(--token-color-foreground-faint)" />
                      <span style={{ fontSize: 13, fontWeight: 600, color: "var(--token-color-foreground-primary)" }}>
                        {cca?.name ?? "?"} · {pos?.name ?? `Position ${ap.positionId}`}
                      </span>
                      {outcome ? <StatusPill status={outcome.status} label={outcome.label} /> : null}
                    </div>
                    {ap.note ? (
                      <div style={{ fontSize: 11.5, color: "var(--token-color-foreground-faint)", marginTop: 2 }}>{ap.note}</div>
                    ) : null}
                  </div>
                </button>
                {isArmed ? (
                  <Button color="critical" busy={isRemoving} onClick={() => remove(ap.applicantId, ap.positionId)}>
                    Remove?
                  </Button>
                ) : (
                  <button
                    title="Remove preallocation"
                    onClick={() => setArmed(key)}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      width: 30,
                      height: 30,
                      borderRadius: 7,
                      border: "1px solid var(--token-color-border-strong)",
                      background: "var(--token-color-surface-primary)",
                      cursor: "pointer",
                    }}
                  >
                    <Icon name="trash" size={14} color="var(--token-color-foreground-critical-on-surface)" />
                  </button>
                )}
              </div>
            );
          })}
        </Card>
      )}
    </div>
  );
}
