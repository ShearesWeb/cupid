// QuotaWidget.tsx — port of the reference quotaWidget visual frame (cca-console-design.html
// lines 474-539: token chips, status word colors, caption, appeal row) with a SEMANTIC
// deviation from the mock: this engine's quota rule is `HeldCounts::within_quota` (main+block
// <= 2, sub <= 3, not both main>=1 and sub>=2) — not the mock's "3 valid loadouts" checklist —
// so the "Valid loadouts" section is dropped entirely. Everything here is read straight off
// `QuotaView` (idx.quotaByApp.get(aid)); no domain recomputation happens in the UI.
import type { ReactNode } from "react";
import { Icon } from "./Icon.tsx";
import { statusStyle } from "./statusStyle.ts";
import type { QuotaView } from "../lib/types.ts";

export interface QuotaWidgetProps {
  quota: QuotaView;
  hasRun: boolean;
}

function loadoutText(main: number, block: number, sub: number): string {
  const parts: string[] = [];
  if (main) parts.push(`${main} main`);
  if (block) parts.push(`${block} block`);
  if (sub) parts.push(`${sub} sub`);
  return parts.join(" + ");
}

export function QuotaWidget({ quota, hasRun }: QuotaWidgetProps) {
  const { main, block, sub, appealed, canAddMain, canAddBlock, canAddSub, over } = quota;
  const full = !over && !canAddMain && !canAddBlock && !canAddSub;

  let statusWord: string;
  let sFg: string;
  let sBg: string;
  if (over) {
    statusWord = "Over quota";
    sFg = "var(--token-color-foreground-critical-on-surface)";
    sBg = "rgba(220,38,38,0.12)";
  } else if (full) {
    statusWord = "Full";
    sFg = "var(--token-color-foreground-success-on-surface)";
    sBg = "var(--token-color-surface-success)";
  } else {
    statusWord = "Under quota";
    sFg = "var(--token-color-foreground-faint)";
    sBg = "var(--token-color-surface-strong)";
  }

  const held = main + block + sub;
  let caption: string;
  if (over) {
    caption = `Holding ${loadoutText(main, block, sub)}, breaks the rule`;
  } else if (full) {
    caption = `Holding ${loadoutText(main, block, sub)}`;
  } else {
    const addable: string[] = [];
    if (canAddMain) addable.push("main");
    if (canAddBlock) addable.push("block");
    if (canAddSub) addable.push("sub");
    caption = (held === 0 ? "Open, room for " : "Room for ") + addable.join(" or ");
  }

  // Provenance (existing vs newly allocated) is not carried on QuotaView, so every token
  // uses one uniform style: allocated once a run exists, existing otherwise. This is a
  // simplification vs. the mock, which colored each token by its own history.
  const tokStyle = statusStyle(hasRun ? "allocated" : "existing");
  const chip = (key: string, label: string, width: number) => (
    <span
      key={key}
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        width,
        height: 21,
        borderRadius: 5,
        background: tokStyle.dot,
        color: "#fff",
        fontSize: 11,
        fontWeight: 800,
        fontFamily: "var(--token-typography-font-stack-code)",
        flexShrink: 0,
      }}
    >
      {label}
    </span>
  );
  const tokens: ReactNode[] = [];
  for (let i = 0; i < main; i++) tokens.push(chip(`m${i}`, "M", 34));
  for (let i = 0; i < block; i++) tokens.push(chip(`b${i}`, "B", 34));
  for (let i = 0; i < sub; i++) tokens.push(chip(`s${i}`, "S", 21));

  const appealSt = statusStyle("appealed");

  return (
    <div
      style={{
        width: 226,
        padding: "12px 14px 13px",
        borderRadius: 11,
        background: "var(--token-color-surface-faint)",
        border: "1px solid var(--token-color-border-faint)",
        flexShrink: 0,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 9 }}>
        <span
          style={{
            fontSize: 10,
            fontWeight: 700,
            letterSpacing: "0.6px",
            textTransform: "uppercase",
            color: "var(--token-color-foreground-faint)",
          }}
        >
          Quota
        </span>
        <span
          style={{
            display: "inline-flex",
            alignItems: "center",
            height: 19,
            padding: "0 8px",
            borderRadius: 20,
            background: sBg,
            color: sFg,
            fontSize: 10.5,
            fontWeight: 700,
          }}
        >
          {statusWord}
        </span>
      </div>
      <div style={{ marginBottom: 8, minHeight: 21 }}>
        {tokens.length ? (
          <div style={{ display: "flex", gap: 4, flexWrap: "wrap" }}>{tokens}</div>
        ) : (
          <span style={{ fontSize: 12, color: "var(--token-color-foreground-faint)", fontStyle: "italic" }}>
            Nothing held yet
          </span>
        )}
      </div>
      <div
        style={{
          fontSize: 11,
          color: over ? "var(--token-color-foreground-critical-on-surface)" : "var(--token-color-foreground-faint)",
          fontWeight: over ? 700 : 400,
        }}
      >
        {caption}
      </div>
      {appealed > 0 ? (
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 7,
            marginTop: 10,
            paddingTop: 9,
            borderTop: "1px dashed var(--token-color-border-strong)",
          }}
        >
          <span
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 4,
              height: 20,
              padding: "0 9px",
              borderRadius: 20,
              background: appealSt.bg,
              color: appealSt.fg,
              border: "1px solid " + appealSt.bd,
              fontSize: 10.5,
              fontWeight: 700,
              whiteSpace: "nowrap",
              flexShrink: 0,
            }}
          >
            <Icon name="plus" size={12} color={appealSt.dot} />
            {appealed} appeal
          </span>
          <span style={{ fontSize: 10.5, color: "var(--token-color-foreground-faint)", whiteSpace: "nowrap" }}>
            exempt, uncounted
          </span>
        </div>
      ) : null}
    </div>
  );
}
