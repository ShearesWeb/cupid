import { describe, expect, it } from "vitest";

import { CHOICES_PER_ROUND, choiceTone } from "./ChoiceCoverage.tsx";

const CRITICAL = "var(--token-color-foreground-critical-on-surface)";
const WARNING = "var(--token-color-foreground-warning-on-surface)";
const SUCCESS = "var(--token-color-foreground-success-on-surface)";

describe("choiceTone", () => {
  it("assumes a six-choice ballot", () => {
    expect(CHOICES_PER_ROUND).toBe(6);
  });

  it("separates never applying from applying thinly", () => {
    expect(choiceTone(0)).toEqual({ color: CRITICAL, label: "didn't apply" });
    expect(choiceTone(1)).toEqual({ color: CRITICAL, label: "5 unused" });
  });

  it("is critical below half the ballot", () => {
    expect(choiceTone(2)).toEqual({ color: CRITICAL, label: "4 unused" });
  });

  it("warns from half the ballot up to the last slot", () => {
    expect(choiceTone(3)).toEqual({ color: WARNING, label: "3 unused" });
    expect(choiceTone(5)).toEqual({ color: WARNING, label: "1 unused" });
  });

  it("passes on a full ballot", () => {
    expect(choiceTone(6)).toEqual({ color: SUCCESS, label: "full ballot" });
  });

  it("does not go negative if a round ever allows more", () => {
    expect(choiceTone(9)).toEqual({ color: SUCCESS, label: "full ballot" });
  });
});
