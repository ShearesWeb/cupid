// shared.tsx — screen atoms reused across Allocations/Review/detail screens.
// Ports reference/cca-console-design.html: emptyState (278-284), runPrompt (285-292),
// legend (293-301), pager (306-316), section (457-460), detailHero (461-473), iconTile (540-542).
import type { ReactNode } from "react";
import { Card, Button, Icon } from "../components/index.ts";
import { statusStyle } from "../components/statusStyle.ts";
import type { Status } from "../lib/types.ts";

export function EmptyState({ title, sub }: { title: string; sub: string }) {
  return (
    <Card
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
        <Icon name="search" size={20} color="var(--token-color-foreground-faint)" />
      </div>
      <div style={{ fontSize: 14, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>{title}</div>
      <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)" }}>{sub}</div>
    </Card>
  );
}

export function RunPrompt({ msg, running, onRun }: { msg: string; running: boolean; onRun: () => void }) {
  return (
    <Card
      padding="large"
      style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 14, padding: 56, textAlign: "center" }}
    >
      <div
        style={{
          width: 48,
          height: 48,
          borderRadius: 12,
          background: "var(--token-color-surface-strong)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <Icon name="layers" size={24} color="var(--token-color-foreground-faint)" />
      </div>
      <div style={{ fontSize: 15, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>No fresh run yet</div>
      <div style={{ fontSize: 13, color: "var(--token-color-foreground-faint)", maxWidth: 380 }}>{msg}</div>
      <Button color="primary" busy={running} icon="layers" onClick={onRun}>
        {running ? "Running…" : "Run matching"}
      </Button>
    </Card>
  );
}

// Seats are only ever existing/allocated/preallocated: the allocations
// screen colours seats, so its legend stops there.
const SEAT_LEGEND: [Status, string][] = [
  ["existing", "Existing appointment"],
  ["allocated", "New allocation"],
  ["preallocated", "Preallocated"],
];

// The ranked lists on the detail pages also show why someone is NOT seated.
const OUTCOME_LEGEND: [Status, string][] = [
  ["existing", "Existing"],
  ["allocated", "New allocation"],
  ["preallocated", "Preallocated"],
  ["displaced", "Displaced"],
  ["quota", "Quota-blocked"],
  ["noreturn", "Didn't rank back"],
];

export function Legend() {
  return <LegendRow items={SEAT_LEGEND} />;
}

export function OutcomeLegend() {
  return <LegendRow items={OUTCOME_LEGEND} />;
}

function LegendRow({ items }: { items: [Status, string][] }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 16, flexWrap: "wrap", rowGap: 8 }}>
      {items.map(([s, label]) => {
        const st = statusStyle(s);
        return (
          <span
            key={s}
            style={{ display: "inline-flex", alignItems: "center", gap: 7, fontSize: 12, color: "var(--token-color-foreground-faint)" }}
          >
            <span style={{ width: 11, height: 11, borderRadius: 3, background: st.bg, border: "1.5px solid " + st.dot }} />
            {label}
          </span>
        );
      })}
    </div>
  );
}

export function Pager({
  total,
  perPage,
  page,
  onSetPage,
}: {
  total: number;
  perPage: number;
  page: number;
  onSetPage: (p: number) => void;
}) {
  const pages = Math.max(1, Math.ceil(total / perPage));
  const clamped = Math.min(page, pages - 1);
  if (pages <= 1) return null;
  const btn = (icon: string, disabled: boolean, dir: number) => (
    <button
      disabled={disabled}
      onClick={() => onSetPage(clamped + dir)}
      style={{
        width: 32,
        height: 32,
        borderRadius: 7,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        cursor: disabled ? "default" : "pointer",
        opacity: disabled ? 0.4 : 1,
        background: "var(--token-color-surface-primary)",
        border: "1px solid var(--token-color-border-strong)",
      }}
    >
      <Icon name={icon} size={16} color="var(--token-color-foreground-strong)" />
    </button>
  );
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "center", gap: 12, marginTop: 22 }}>
      {btn("chevron-left", clamped <= 0, -1)}
      <span
        style={{
          fontSize: 12.5,
          fontWeight: 600,
          color: "var(--token-color-foreground-faint)",
          fontFamily: "var(--token-typography-font-stack-code)",
        }}
      >
        Page {clamped + 1} of {pages}
      </span>
      {btn("chevron-right", clamped >= pages - 1, 1)}
    </div>
  );
}

// ---- detail-page atoms (port of reference lines 457-542) -----------------
export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div style={{ marginBottom: 20 }}>
      <div
        style={{
          fontSize: 10.5,
          fontWeight: 700,
          letterSpacing: "0.6px",
          textTransform: "uppercase",
          color: "#DB2A63",
          marginBottom: 9,
        }}
      >
        {title}
      </div>
      {children}
    </div>
  );
}

export function DetailHero({
  leading,
  eyebrow,
  title,
  badge,
  right,
  rightValue,
  rightLabel,
  rightColor,
}: {
  leading: ReactNode;
  eyebrow: string;
  title: string;
  badge?: ReactNode;
  right?: ReactNode;
  rightValue?: string;
  rightLabel?: string;
  rightColor?: string;
}) {
  return (
    <div style={{ display: "flex", alignItems: "flex-start", gap: 13, marginBottom: 22 }}>
      {leading}
      <div style={{ flex: 1, minWidth: 0, paddingTop: 2 }}>
        <div
          style={{
            fontSize: 11.5,
            color: "var(--token-color-foreground-faint)",
            fontWeight: 600,
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
          }}
        >
          {eyebrow}
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 9, marginTop: 2 }}>
          <span style={{ fontSize: 19, fontWeight: 700, color: "var(--token-color-foreground-strong)", letterSpacing: "-0.3px" }}>
            {title}
          </span>
          {badge ?? null}
        </div>
      </div>
      {right !== undefined
        ? right
        : rightValue != null
          ? (
              <div style={{ textAlign: "right", flexShrink: 0 }}>
                <div
                  style={{
                    fontSize: 18,
                    fontWeight: 700,
                    fontFamily: "var(--token-typography-font-stack-code)",
                    color: rightColor || "var(--token-color-foreground-strong)",
                    lineHeight: 1,
                  }}
                >
                  {rightValue}
                </div>
                <div
                  style={{
                    fontSize: 10,
                    color: "var(--token-color-foreground-faint)",
                    marginTop: 3,
                    textTransform: "uppercase",
                    letterSpacing: "0.5px",
                    fontWeight: 600,
                  }}
                >
                  {rightLabel}
                </div>
              </div>
            )
          : null}
    </div>
  );
}

export function IconTile({ name }: { name: string }) {
  return (
    <div
      style={{
        width: 42,
        height: 42,
        borderRadius: 11,
        background: "var(--token-color-surface-strong)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        flexShrink: 0,
      }}
    >
      <Icon name={name} size={21} color="var(--token-color-foreground-faint)" />
    </div>
  );
}
