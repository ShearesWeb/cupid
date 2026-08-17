// statusStyle.ts — direct port of reference lines 77-88 (cca-console-design.html)
import type { Status } from "../lib/types.ts";

export interface StatusStyle {
  bg: string;
  fg: string;
  bd: string;
  dot: string;
}

const MAP: Record<Status, StatusStyle> = {
  existing: {
    bg: "var(--token-color-surface-action)",
    fg: "var(--token-color-foreground-action)",
    bd: "var(--token-color-border-action)",
    dot: "var(--token-color-foreground-action)",
  },
  allocated: {
    bg: "var(--token-color-surface-success)",
    fg: "var(--token-color-foreground-success-on-surface)",
    bd: "var(--token-color-border-success)",
    dot: "var(--token-color-foreground-success)",
  },
  preallocated: {
    bg: "rgba(219,42,99,0.10)",
    fg: "#B91C53",
    bd: "rgba(219,42,99,0.32)",
    dot: "#DB2A63",
  },
  displaced: {
    bg: "var(--token-color-surface-warning)",
    fg: "var(--token-color-foreground-warning-on-surface)",
    bd: "var(--token-color-border-warning)",
    dot: "var(--token-color-foreground-warning)",
  },
  quota: {
    bg: "var(--token-color-surface-highlight)",
    fg: "var(--token-color-foreground-highlight-on-surface)",
    bd: "var(--token-color-border-highlight)",
    dot: "var(--token-color-foreground-highlight)",
  },
  noreturn: {
    bg: "var(--token-color-surface-faint)",
    fg: "var(--token-color-foreground-faint)",
    bd: "var(--token-color-border-strong)",
    dot: "var(--token-color-border-strong)",
  },
  neutral: {
    bg: "var(--token-color-surface-strong)",
    fg: "var(--token-color-foreground-faint)",
    bd: "var(--token-color-border-faint)",
    dot: "var(--token-color-foreground-faint)",
  },
};

export function statusStyle(status: Status): StatusStyle {
  return MAP[status] ?? MAP.neutral;
}
