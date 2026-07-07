// StatusPill.tsx — port of reference lines 273-277 (cca-console-design.html)
import type { Status } from "../lib/types.ts";
import { statusStyle } from "./statusStyle.ts";

export interface StatusPillProps {
  status: Status;
  label: string;
}

export function StatusPill({ status, label }: StatusPillProps) {
  const st = statusStyle(status);
  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 5,
        height: 20,
        padding: "0 8px",
        borderRadius: 20,
        background: st.bg,
        color: st.fg,
        border: "1px solid " + st.bd,
        fontSize: 11,
        fontWeight: 700,
        whiteSpace: "nowrap",
      }}
    >
      <span style={{ width: 6, height: 6, borderRadius: "50%", background: st.dot }} />
      {label}
    </span>
  );
}
