// Toasts.tsx — fixed bottom-right stack, auto-dismiss after 5s (task-12 brief).
import { useEffect } from "react";
import { Icon } from "./Icon.tsx";

export type ToastKind = "error" | "success";

export interface ToastItem {
  id: number;
  kind: ToastKind;
  text: string;
}

export interface ToastsProps {
  toasts: ToastItem[];
  onDismiss: (id: number) => void;
}

export function Toasts({ toasts, onDismiss }: ToastsProps) {
  if (toasts.length === 0) return null;
  return (
    <div
      style={{
        position: "fixed",
        bottom: 20,
        right: 20,
        display: "flex",
        flexDirection: "column",
        gap: 8,
        zIndex: 100,
      }}
    >
      {toasts.map((t) => (
        <ToastRow key={t.id} toast={t} onDismiss={onDismiss} />
      ))}
    </div>
  );
}

function ToastRow({ toast, onDismiss }: { toast: ToastItem; onDismiss: (id: number) => void }) {
  useEffect(() => {
    const timer = setTimeout(() => onDismiss(toast.id), 5000);
    return () => clearTimeout(timer);
  }, [toast.id, onDismiss]);

  const isError = toast.kind === "error";
  const tone = isError
    ? {
        background: "var(--token-color-surface-critical)",
        color: "var(--token-color-foreground-critical-on-surface)",
        border: "var(--token-color-border-critical)",
      }
    : {
        background: "var(--token-color-surface-success)",
        color: "var(--token-color-foreground-success-on-surface)",
        border: "var(--token-color-border-success)",
      };

  return (
    <div
      role="status"
      style={{
        display: "flex",
        alignItems: "center",
        gap: 9,
        minWidth: 240,
        maxWidth: 380,
        padding: "11px 14px",
        borderRadius: 9,
        background: tone.background,
        color: tone.color,
        border: `1px solid ${tone.border}`,
        fontSize: 12.5,
        fontWeight: 600,
        boxShadow: "var(--token-elevation-high-box-shadow)",
      }}
    >
      <Icon name={isError ? "x-circle" : "check-circle"} size={16} color={tone.color} />
      <span style={{ flex: 1 }}>{toast.text}</span>
      <button
        onClick={() => onDismiss(toast.id)}
        aria-label="Dismiss"
        style={{
          border: "none",
          background: "transparent",
          cursor: "pointer",
          padding: 2,
          display: "flex",
          color: tone.color,
          opacity: 0.7,
        }}
      >
        <Icon name="x" size={14} color={tone.color} />
      </button>
    </div>
  );
}
