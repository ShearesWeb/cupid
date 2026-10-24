// CapSummary.tsx — the CCA caps part of a profile's quota box: a status pill,
// one line per cap (committee, Sports/Culture, total) with its counts, limit
// and state, and a caption. Everything is read off the Rust CommitteeCapEntry
// and this year's CapRules; nothing is recomputed here beyond adding up the
// counts it reports.
import { committeeCombos } from "../lib/directory.ts";
import type { CapRules, CommitteeCapCount, CommitteeCapEntry } from "../lib/types.ts";

type State = "ok" | "appeal" | "over" | "na";

const STATE: Record<State, { label: string; fg: string; bg: string }> = {
  ok: { label: "OK", fg: "var(--token-color-foreground-success-on-surface)", bg: "var(--token-color-surface-success)" },
  appeal: { label: "Appeal", fg: "var(--token-color-foreground-success-on-surface)", bg: "var(--token-color-surface-success)" },
  over: { label: "Over", fg: "var(--token-color-foreground-critical-on-surface)", bg: "rgba(220,38,38,0.12)" },
  na: { label: "n/a", fg: "var(--token-color-foreground-faint)", bg: "var(--token-color-surface-strong)" },
};

function committeeState(c: CommitteeCapCount, appealed: boolean): State {
  if (c.main + c.sub === 0) return "na";
  if (!c.exceedsStandard) return "ok";
  return appealed && !c.exceedsAppeal ? "appeal" : "over";
}

/** Within the no-appeal cap, within the limit that applies, or over it. */
function limitState(value: number, exceedsStandard: boolean, limit: number): State {
  if (!exceedsStandard) return "ok";
  return value <= limit ? "appeal" : "over";
}

export function CapSummary({ entry, loading, rules }: { entry: CommitteeCapEntry | null; loading: boolean; rules: CapRules }) {
  let pill: { text: string; fg: string; bg: string };
  if (!entry) pill = { text: loading ? "Checking…" : "No CCAs", fg: STATE.na.fg, bg: STATE.na.bg };
  else if (entry.overCap || entry.overCombinedCap) pill = { text: "Over cap", fg: STATE.over.fg, bg: STATE.over.bg };
  else if (entry.counts.some((c) => c.exceedsStandard || c.exceedsSportsCulture || c.exceedsTotal)) {
    pill = { text: "By appeal", fg: STATE.appeal.fg, bg: STATE.appeal.bg };
  } else pill = { text: "Within caps", fg: STATE.ok.fg, bg: STATE.ok.bg };

  const captions: { text: string; strong?: boolean }[] = [];
  if (!entry) {
    if (!loading) captions.push({ text: "No counted CCAs" });
  } else {
    if (entry.overCap) {
      captions.push({
        text: entry.appealed
          ? `Over appeal cap: max ${committeeCombos(rules.committeeCombosAppealed)}`
          : `Over committee cap: max ${committeeCombos(rules.committeeCombos)}`,
        strong: true,
      });
    }
    for (const c of entry.counts) {
      const prefix = c.semester === null ? "" : `Sem ${c.semester}: `;
      if (c.sportsCulture > c.sportsCultureLimit) captions.push({ text: `${prefix}Sports/Culture over ${c.sportsCultureLimit}`, strong: true });
      if (c.main + c.sub + c.sportsCulture > c.totalLimit) captions.push({ text: `${prefix}Total CCAs over ${c.totalLimit}`, strong: true });
    }
    if (entry.duplicateCcas.length) captions.push({ text: "Holds two seats in one committee", strong: true });
    if (!captions.length) {
      const used = [entry.appealed && "committee", entry.combinedAppealed && "CCA cap"].filter(Boolean);
      captions.push({ text: used.length ? `Appeal approved: ${used.join(" and ")}` : "Within all caps" });
    }
  }

  return (
    <>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 9 }}>
        <span style={{ fontSize: 10, fontWeight: 700, letterSpacing: "0.6px", textTransform: "uppercase", color: "var(--token-color-foreground-faint)" }}>CCA caps</span>
        <span
          style={{
            display: "inline-flex",
            alignItems: "center",
            height: 19,
            padding: "0 8px",
            borderRadius: 20,
            background: pill.bg,
            color: pill.fg,
            fontSize: 10.5,
            fontWeight: 700,
          }}
        >
          {pill.text}
        </span>
      </div>
      {entry ? (
        <div style={{ display: "flex", flexDirection: "column", gap: 5, marginBottom: 8 }}>
          {entry.counts.map((c) => {
            const total = c.main + c.sub + c.sportsCulture;
            return (
              <div key={c.semester ?? "year"} style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                {c.semester === null ? null : (
                  <span style={{ fontSize: 10, fontWeight: 700, color: "var(--token-color-foreground-faint)" }}>Sem {c.semester}</span>
                )}
                <CapLine label="Committee" value={`${c.main}M · ${c.sub}S`} state={committeeState(c, entry.appealed)} />
                <CapLine
                  label="Sports/Culture"
                  value={`${c.sportsCulture} / ${c.sportsCultureLimit}`}
                  title={`${c.sportsCulture} Sports/Cultural CCAs, limit ${c.sportsCultureLimit}`}
                  state={limitState(c.sportsCulture, c.exceedsSportsCulture, c.sportsCultureLimit)}
                />
                <CapLine
                  label="Total"
                  value={`${total} / ${c.totalLimit}`}
                  title={`${c.main + c.sub} committee + ${c.sportsCulture} Sports/Culture = ${total}, limit ${c.totalLimit}`}
                  state={limitState(total, c.exceedsTotal, c.totalLimit)}
                />
              </div>
            );
          })}
        </div>
      ) : null}
      {captions.map((c) => (
        <div
          key={c.text}
          style={{
            fontSize: 11,
            color: c.strong ? "var(--token-color-foreground-critical-on-surface)" : "var(--token-color-foreground-faint)",
            fontWeight: c.strong ? 700 : 400,
          }}
        >
          {c.text}
        </div>
      ))}
    </>
  );
}

function CapLine({ label, value, title, state }: { label: string; value: string; title?: string; state: State }) {
  const s = STATE[state];
  return (
    <div title={title} style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 11.5 }}>
      <span style={{ color: "var(--token-color-foreground-faint)", whiteSpace: "nowrap" }}>{label}</span>
      <span style={{ flex: 1 }} />
      <span style={{ fontWeight: 700, color: "var(--token-color-foreground-strong)", fontFamily: "var(--token-typography-font-stack-code)", whiteSpace: "nowrap" }}>
        {value}
      </span>
      <span
        style={{
          display: "inline-flex",
          alignItems: "center",
          height: 17,
          padding: "0 6px",
          borderRadius: 5,
          background: s.bg,
          color: s.fg,
          fontSize: 10,
          fontWeight: 700,
          flexShrink: 0,
        }}
      >
        {s.label}
      </span>
    </div>
  );
}
