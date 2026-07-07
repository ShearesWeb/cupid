// MatchRow.tsx — port of reference lines 543-555 (matchRow). Shared by detail-page
// sections (ApplicantDetail/PositionDetail) and later screens that list ranked/seated pairs.
import { Icon } from "./Icon.tsx";
import { StatusPill } from "./StatusPill.tsx";
import { statusStyle } from "./statusStyle.ts";
import type { Status } from "../lib/types.ts";

export interface MatchRowProps {
  num: number;
  name: string;
  sub?: string | null;
  meta?: string | null;
  metaColor?: string;
  status: Status;
  statusLabel: string;
  onClick: () => void;
}

const TINTED: Status[] = ["existing", "allocated", "appealed"];

export function MatchRow({ num, name, sub, meta, metaColor, status, statusLabel, onClick }: MatchRowProps) {
  const st = statusStyle(status);
  const tint = TINTED.includes(status);
  return (
    <button
      onClick={onClick}
      style={{
        display: "flex",
        alignItems: "center",
        flexWrap: "wrap",
        gap: "6px 12px",
        width: "100%",
        padding: "11px 12px",
        border: "none",
        borderBottom: "1px solid var(--token-color-border-faint)",
        background: tint ? st.bg : "transparent",
        cursor: "pointer",
        font: "inherit",
        textAlign: "left",
      }}
    >
      <span
        style={{
          fontFamily: "var(--token-typography-font-stack-code)",
          fontSize: 12,
          fontWeight: 700,
          color: "var(--token-color-foreground-faint)",
          width: 22,
          flexShrink: 0,
        }}
      >
        {num}.
      </span>
      <div style={{ flex: "1 1 160px", minWidth: 0 }}>
        <div
          style={{
            fontSize: 13.5,
            fontWeight: 600,
            color: "var(--token-color-foreground-strong)",
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
          }}
        >
          {name}
        </div>
        {sub ? (
          <div style={{ fontSize: 11.5, color: "var(--token-color-foreground-faint)", marginTop: 2, lineHeight: 1.4 }}>
            {sub}
          </div>
        ) : null}
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 10, flexShrink: 0, marginLeft: "auto" }}>
        {meta ? (
          <span
            style={{
              fontSize: 11,
              fontWeight: 600,
              fontFamily: "var(--token-typography-font-stack-code)",
              color: metaColor || "var(--token-color-foreground-faint)",
              whiteSpace: "nowrap",
            }}
          >
            {meta}
          </span>
        ) : null}
        {statusLabel && statusLabel !== "—" ? <StatusPill status={status} label={statusLabel} /> : null}
        <Icon name="chevron-right" size={15} color="var(--token-color-foreground-faint)" />
      </div>
    </button>
  );
}
