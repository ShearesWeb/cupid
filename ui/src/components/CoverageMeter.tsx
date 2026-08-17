// CoverageMeter.tsx — the shared "how much of the target is covered" row: a
// count on the left, a tinted verdict on the right, and one track under both.
// The track is segmented when the target is a fixed small number of slots and
// continuous when it is a ratio, so a countable target stays countable.
export interface CoverageMeterProps {
  count: number;
  countLabel: string;
  verdict: string;
  color: string;
  /** Continuous fill from 0 to 1. Used only when `segments` is absent. */
  fill?: number;
  /** Discrete track: one pill per slot, the first `count` of them lit. */
  segments?: number;
}

export function CoverageMeter({ count, countLabel, verdict, color, fill = 0, segments }: CoverageMeterProps) {
  const empty = "var(--token-color-surface-strong)";
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 5 }}>
      {/* Both labels stay on one line: a wrapped verdict reads as a defect. */}
      <div style={{ display: "flex", alignItems: "baseline", justifyContent: "space-between", gap: 8, fontSize: 12, whiteSpace: "nowrap" }}>
        <span style={{ color: "var(--token-color-foreground-faint)", overflow: "hidden", textOverflow: "ellipsis" }}>
          <span style={{ fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>{count}</span> {countLabel}
        </span>
        <span style={{ color, fontWeight: 600 }}>{verdict}</span>
      </div>
      {segments ? (
        <div style={{ display: "flex", gap: 3 }}>
          {Array.from({ length: segments }, (_, i) => (
            <span key={i} style={{ flex: 1, height: 4, borderRadius: 3, background: i < count ? color : empty }} />
          ))}
        </div>
      ) : (
        <div style={{ height: 4, borderRadius: 3, background: empty, overflow: "hidden" }}>
          <div style={{ width: `${fill * 100}%`, height: "100%", borderRadius: 3, background: color }} />
        </div>
      )}
    </div>
  );
}
