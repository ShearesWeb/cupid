// ChairCoverage.tsx — how deep a chair's shortlist is, measured against the
// seats still open rather than the whole capacity: seats already held are not
// the chair's to fill. Two names per open seat is the depth to aim for, since
// candidates are lost to quota clashes and to other CCAs, so the bar tops out
// at that 200% goal.
import { CoverageMeter } from "./CoverageMeter.tsx";

export function coverageTone(ranked: number, openSeats: number): { color: string; label: string } {
  if (openSeats <= 0) {
    return { color: "var(--token-color-foreground-faint)", label: "no open seats" };
  }
  if (ranked === 0) {
    return { color: "var(--token-color-foreground-critical-on-surface)", label: "none ranked" };
  }
  // Floor, not round: 99.5% coverage must not read as a full 100%.
  const pct = Math.floor((ranked / openSeats) * 100);
  const color =
    pct < 100
      ? "var(--token-color-foreground-critical-on-surface)"
      : pct < 200
        ? "var(--token-color-foreground-warning-on-surface)"
        : "var(--token-color-foreground-success-on-surface)";
  return { color, label: `${pct}% of open seats` };
}

export function ChairCoverage({ ranked, openSeats }: { ranked: number; openSeats: number }) {
  const tone = coverageTone(ranked, openSeats);
  return (
    <CoverageMeter
      count={ranked}
      countLabel="ranked by chair"
      verdict={tone.label}
      color={tone.color}
      fill={openSeats > 0 ? Math.min(ranked / (openSeats * 2), 1) : 1}
    />
  );
}
