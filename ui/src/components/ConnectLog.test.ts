import { describe, expect, it } from "vitest";

import { groupEvents } from "./ConnectLog.tsx";
import type { ConnectEvent } from "../lib/types.ts";

const target = (host: string, port: number): ConnectEvent => ({ kind: "target", host, port });
const step = (stage: "dns" | "tcp" | "signIn", status: "running" | "ok" | "failed" | "skipped", detail: string | null = null): ConnectEvent => ({
  kind: "step",
  stage,
  status,
  detail,
});

describe("groupEvents", () => {
  it("replaces a running step with its outcome", () => {
    const groups = groupEvents([target("h", 5432), step("dns", "running"), step("dns", "ok", "1.2.3.4")]);
    expect(groups).toEqual([{ host: "h", port: 5432, steps: [{ label: "DNS", status: "ok", detail: "1.2.3.4" }] }]);
  });

  it("keeps the step in progress visible", () => {
    const groups = groupEvents([target("h", 5432), step("dns", "ok"), step("tcp", "running")]);
    expect(groups[0].steps.map((s) => [s.label, s.status])).toEqual([
      ["DNS", "ok"],
      ["TCP", "running"],
    ]);
  });

  it("starts a new group per target so a fallback port reads separately", () => {
    const groups = groupEvents([
      target("pooler", 5432),
      step("tcp", "running"),
      step("tcp", "failed", "dropped"),
      target("pooler", 6543),
      step("tcp", "skipped"),
    ]);
    expect(groups.map((g) => [g.port, g.steps.map((s) => s.status)])).toEqual([
      [5432, ["failed"]],
      [6543, ["skipped"]],
    ]);
  });
});
