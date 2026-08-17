// UpdatePrompt.tsx — modal shown when a newer release exists. Wry renders no
// native confirm(), so the prompt has to live in the app.
import type { PendingUpdate } from "../lib/updater.ts";
import { Button } from "./Button.tsx";

export interface UpdatePromptProps {
  pending: PendingUpdate;
  installing: boolean;
  onInstall: () => void;
  onDismiss: () => void;
}

export function UpdatePrompt({ pending, installing, onInstall, onDismiss }: UpdatePromptProps) {
  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 70,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "rgba(0,0,0,0.4)",
      }}
    >
      <div
        style={{
          width: 400,
          padding: 20,
          borderRadius: 12,
          background: "var(--token-color-surface-primary)",
          boxShadow: "var(--token-elevation-high-box-shadow)",
        }}
      >
        <div style={{ fontSize: 15, fontWeight: 700, color: "var(--token-color-foreground-strong)", marginBottom: 4 }}>
          Update available
        </div>
        <div style={{ fontSize: 12, color: "var(--token-color-foreground-faint)", marginBottom: 14 }}>
          Cupid {pending.version} is ready — you are on {pending.current}. Installing restarts the app; an
          uncommitted run is lost and has to be re-run after restart.
        </div>
        {pending.notes ? (
          <div
            style={{
              maxHeight: 150,
              overflow: "auto",
              marginBottom: 14,
              padding: "9px 11px",
              borderRadius: 7,
              background: "var(--token-color-surface-faint)",
              border: "1px solid var(--token-color-border-faint)",
              fontSize: 11.5,
              lineHeight: 1.55,
              color: "var(--token-color-foreground-primary)",
              whiteSpace: "pre-wrap",
            }}
          >
            {pending.notes}
          </div>
        ) : null}
        <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
          <Button color="ghost" disabled={installing} onClick={onDismiss}>
            Later
          </Button>
          <Button color="primary" busy={installing} icon="download" onClick={onInstall}>
            {installing ? "Installing…" : "Install & restart"}
          </Button>
        </div>
      </div>
    </div>
  );
}
