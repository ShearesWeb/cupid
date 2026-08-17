import { describe, expect, it } from "vitest";

import { coverageTone } from "./ChairCoverage.tsx";

const CRITICAL = "var(--token-color-foreground-critical-on-surface)";
const WARNING = "var(--token-color-foreground-warning-on-surface)";
const SUCCESS = "var(--token-color-foreground-success-on-surface)";
const FAINT = "var(--token-color-foreground-faint)";

describe("coverageTone", () => {
  it("calls an empty shortlist out by name", () => {
    expect(coverageTone(0, 8)).toEqual({ color: CRITICAL, label: "none ranked" });
  });

  it("is critical while the shortlist is shorter than the open seats", () => {
    expect(coverageTone(1, 8)).toEqual({ color: CRITICAL, label: "12% of open seats" });
    expect(coverageTone(7, 8)).toEqual({ color: CRITICAL, label: "87% of open seats" });
  });

  it("warns between one and two names per open seat", () => {
    // Exactly one per seat leaves no room for a quota clash, so it is not yet green.
    expect(coverageTone(8, 8)).toEqual({ color: WARNING, label: "100% of open seats" });
    expect(coverageTone(15, 8)).toEqual({ color: WARNING, label: "187% of open seats" });
  });

  it("passes at two names per open seat and beyond", () => {
    expect(coverageTone(16, 8)).toEqual({ color: SUCCESS, label: "200% of open seats" });
    expect(coverageTone(40, 8)).toEqual({ color: SUCCESS, label: "500% of open seats" });
  });

  it("measures against the open seats, not the whole capacity", () => {
    // Ten seats, six already held: the chair is filling four, so eight names
    // clear the goal even though they are short of the capacity.
    expect(coverageTone(8, 4)).toEqual({ color: SUCCESS, label: "200% of open seats" });
  });

  it("asks for nothing once every seat is held", () => {
    expect(coverageTone(0, 0)).toEqual({ color: FAINT, label: "no open seats" });
    expect(coverageTone(9, 0)).toEqual({ color: FAINT, label: "no open seats" });
  });
});
