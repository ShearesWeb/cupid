// TextInput.tsx — 34px input with optional label and leading icon, per task-11 brief.
import type { ChangeEvent, ReactNode } from "react";

export interface TextInputProps {
  label?: string;
  placeholder?: string;
  icon?: ReactNode;
  value: string;
  onChange: (e: ChangeEvent<HTMLInputElement>) => void;
}

export function TextInput({ label, placeholder, icon, value, onChange }: TextInputProps) {
  const field = (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 7,
        height: 34,
        padding: icon ? "0 10px" : "0 12px",
        borderRadius: 7,
        background: "var(--token-color-surface-primary)",
        border: "1px solid var(--token-color-border-strong)",
      }}
    >
      {icon}
      <input
        value={value}
        onChange={onChange}
        placeholder={placeholder}
        style={{
          flex: 1,
          minWidth: 0,
          border: "none",
          outline: "none",
          background: "transparent",
          font: "inherit",
          fontSize: 12.5,
          color: "var(--token-color-foreground-strong)",
        }}
      />
    </div>
  );

  if (!label) return field;

  return (
    <label style={{ display: "flex", flexDirection: "column", gap: 5 }}>
      <span style={{ fontSize: 11.5, fontWeight: 600, color: "var(--token-color-foreground-faint)" }}>{label}</span>
      {field}
    </label>
  );
}
