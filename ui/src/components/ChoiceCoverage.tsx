// ChoiceCoverage.tsx — how much of their ballot an applicant used. Unlike a
// chair shortlist there is no stretch goal here: the round hands every
// resident a fixed number of choices, and an unused slot is a chance forgone.
import { CoverageMeter } from "./CoverageMeter.tsx";

/** Choices the intranet form allows one resident per allocation round. */
export const CHOICES_PER_ROUND = 6;

export function choiceTone(ranked: number): { color: string; label: string } {
  if (ranked === 0) {
    return { color: "var(--token-color-foreground-critical-on-surface)", label: "didn't apply" };
  }
  if (ranked >= CHOICES_PER_ROUND) {
    return { color: "var(--token-color-foreground-success-on-surface)", label: "full ballot" };
  }
  // Below half the ballot a single quota clash can leave them with nothing.
  const color =
    ranked * 2 < CHOICES_PER_ROUND
      ? "var(--token-color-foreground-critical-on-surface)"
      : "var(--token-color-foreground-warning-on-surface)";
  return { color, label: `${CHOICES_PER_ROUND - ranked} unused` };
}

export function ChoiceCoverage({ ranked }: { ranked: number }) {
  const tone = choiceTone(ranked);
  return (
    <CoverageMeter
      count={ranked}
      countLabel={`of ${CHOICES_PER_ROUND} ranked`}
      verdict={tone.label}
      color={tone.color}
      segments={CHOICES_PER_ROUND}
    />
  );
}
