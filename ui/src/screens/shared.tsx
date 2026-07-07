// shared.tsx — screen atoms reused across Allocations/Review/detail screens.
// Ports reference/cca-console-design.html: emptyState (278-284), runPrompt (285-292),
// legend (293-301), pager (306-316).
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

const PRE_RUN_LEGEND: [Status, string][] = [
  ["existing", "Existing appointment"],
  ["allocated", "New allocation"],
  ["appealed", "Appeal (exempt)"],
];

const POST_RUN_LEGEND: [Status, string][] = [
  ["existing", "Existing"],
  ["allocated", "New allocation"],
  ["appealed", "Appeal (exempt)"],
  ["displaced", "Displaced"],
  ["quota", "Quota-blocked"],
  ["noreturn", "Didn't rank back"],
];

export function Legend({ hasRun }: { hasRun: boolean }) {
  const items = hasRun ? POST_RUN_LEGEND : PRE_RUN_LEGEND;
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
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
