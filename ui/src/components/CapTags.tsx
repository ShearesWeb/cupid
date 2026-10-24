// CapTags.tsx — cap status tags and appeal toggles for one resident, shared by
// the CCA screens and the CCA caps screen so both read the same.
import { CAP_STATUS, capCountText, capStatuses } from "../lib/directory.ts";
import type { CommitteeCapEntry } from "../lib/types.ts";

const TONE = {
  critical: { bg: "var(--token-color-surface-critical)", fg: "var(--token-color-foreground-critical-on-surface)", border: "var(--token-color-border-critical)" },
  warning: { bg: "var(--token-color-surface-warning)", fg: "var(--token-color-foreground-warning-on-surface)", border: "var(--token-color-border-warning)" },
  success: { bg: "var(--token-color-surface-success)", fg: "var(--token-color-foreground-success-on-surface)", border: "var(--token-color-border-success)" },
  neutral: { bg: "var(--token-color-surface-faint)", fg: "var(--token-color-foreground-faint)", border: "var(--token-color-border-strong)" },
} as const;

export function CapTags({ entry }: { entry: CommitteeCapEntry }) {
  const counts = entry.counts.map(capCountText).join("; ");
  return (
    <>
      {capStatuses(entry).map((status) => {
        const { label, tone, hint } = CAP_STATUS[status];
        const t = TONE[tone];
        return (
          <span
            key={status}
            title={`${hint}\n${counts}`}
            style={{
              display: "inline-flex",
              alignItems: "center",
              height: 18,
              padding: "0 6px",
              borderRadius: 5,
              background: t.bg,
              color: t.fg,
              border: `1px solid ${t.border}`,
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
      })}
    </>
  );
}

/** One approved-appeal toggle. Ticking it records the JCRC's approval locally. */
export function AppealCheckbox({ label, title, checked, busy, onChange }: { label: string; title: string; checked: boolean; busy: boolean; onChange: () => void }) {
  return (
    <label
      title={title}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 7,
        height: 30,
        padding: "0 10px",
        borderRadius: 7,
        fontSize: 12.5,
        fontWeight: 600,
        whiteSpace: "nowrap",
        cursor: busy ? "progress" : "pointer",
        border: `1px solid ${checked ? "var(--token-color-border-success)" : "var(--token-color-border-strong)"}`,
        background: checked ? "var(--token-color-surface-success)" : "transparent",
        color: "var(--token-color-foreground-strong)",
      }}
    >
      <input type="checkbox" checked={checked} disabled={busy} onChange={onChange} style={{ accentColor: "var(--cupid)", margin: 0 }} />
      {label}
    </label>
  );
}
