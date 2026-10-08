import type { ConnectEvent, ConnectStage, StepStatus } from "../lib/types.ts";
import { Icon } from "./Icon.tsx";
import { Spinner } from "./Spinner.tsx";

export interface LogStep {
  label: string;
  status: StepStatus;
  detail: string | null;
}

export interface LogTarget {
  host: string;
  port: number;
  steps: LogStep[];
}

/** Connect events so far, plus the data load that follows a successful connect. */
export interface ConnectLogState {
  events: ConnectEvent[];
  load: LogStep | null;
}

const STAGE_LABEL: Record<ConnectStage, string> = {
  dns: "DNS",
  tcp: "TCP",
  signIn: "Sign in",
};

/** Fold the event stream into one row per step: a step's final status replaces its running row. */
export function groupEvents(events: ConnectEvent[]): LogTarget[] {
  const targets: LogTarget[] = [];
  for (const event of events) {
    if (event.kind === "target") {
      targets.push({ host: event.host, port: event.port, steps: [] });
      continue;
    }
    const target = targets[targets.length - 1];
    if (!target) continue;
    const step = { label: STAGE_LABEL[event.stage], status: event.status, detail: event.detail };
    const last = target.steps[target.steps.length - 1];
    if (last && last.label === step.label && last.status === "running") {
      target.steps[target.steps.length - 1] = step;
    } else {
      target.steps.push(step);
    }
  }
  return targets;
}

const FAINT = "var(--token-color-foreground-faint)";

function StatusMark({ status }: { status: StepStatus }) {
  const box = { width: 14, height: 14, display: "inline-flex", alignItems: "center", justifyContent: "center" };
  if (status === "running") {
    return (
      <span style={box}>
        <Spinner />
      </span>
    );
  }
  const [name, color] =
    status === "ok"
      ? ["check", "var(--token-color-foreground-success-on-surface)"]
      : status === "failed"
        ? ["x", "var(--token-color-foreground-critical-on-surface)"]
        : ["arrow-right", FAINT];
  return (
    <span style={box}>
      <Icon name={name} size={14} color={color} />
    </span>
  );
}

function StepRow({ step }: { step: LogStep }) {
  return (
    <div style={{ display: "flex", gap: 8, alignItems: "flex-start", paddingLeft: 4 }}>
      <StatusMark status={step.status} />
      <span style={{ width: 58, flexShrink: 0, color: "var(--token-color-foreground-strong)" }}>{step.label}</span>
      <span
        style={{
          flex: 1,
          minWidth: 0,
          overflowWrap: "anywhere",
          color: step.status === "failed" ? "var(--token-color-foreground-critical-on-surface)" : FAINT,
        }}
      >
        {step.detail ?? ""}
      </span>
    </div>
  );
}

/** Step-by-step trace of a connect: which layer of the network path answered and which broke. */
export function ConnectLog({ log }: { log: ConnectLogState }) {
  const targets = groupEvents(log.events);
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 4,
        width: "100%",
        padding: "10px 12px",
        borderRadius: 8,
        background: "var(--token-color-surface-primary)",
        border: "1px solid var(--token-color-border-faint)",
        fontFamily: "var(--token-typography-font-stack-code)",
        fontSize: 11.5,
        lineHeight: 1.5,
        textAlign: "left",
      }}
    >
      {targets.map((target, i) => (
        <div key={i} style={{ display: "flex", flexDirection: "column", gap: 2 }}>
          <div style={{ color: FAINT, overflowWrap: "anywhere" }}>
            {target.host}:{target.port}
          </div>
          {target.steps.map((step, j) => (
            <StepRow key={j} step={step} />
          ))}
        </div>
      ))}
      {log.load ? (
        <div style={{ marginTop: 4 }}>
          <StepRow step={log.load} />
        </div>
      ) : null}
    </div>
  );
}
