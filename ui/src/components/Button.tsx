// Button.tsx — port of brandButton/ghostButton (reference lines 262-267),
// plus a `critical` variant built from the critical tokens.
import type { CSSProperties, ReactNode } from "react";
import { Icon } from "./Icon.tsx";
import { Spinner } from "./Spinner.tsx";

export interface ButtonProps {
  color?: "primary" | "critical" | "ghost";
  disabled?: boolean;
  busy?: boolean;
  icon?: string;
  onClick: () => void;
  children: ReactNode;
}

interface Variant {
  background: string;
  color: string;
  border: string;
  fontWeight: number;
  padding: string;
  boxShadow?: string;
  iconColor?: string;
}

function variantFor(color: "primary" | "critical" | "ghost", busy: boolean): Variant {
  if (color === "primary") {
    return {
      background: busy ? "#B91C53" : "#DB2A63",
      color: "#fff",
      border: "1px solid #B91C53",
      fontWeight: 700,
      padding: "0 16px",
      boxShadow: "0 1px 2px rgba(0,0,0,.12)",
      iconColor: "#fff",
    };
  }
  if (color === "critical") {
    return {
      background: "var(--token-color-surface-critical)",
      color: "var(--token-color-foreground-critical-on-surface)",
      border: "1px solid var(--token-color-border-critical)",
      fontWeight: 700,
      padding: "0 14px",
    };
  }
  return {
    background: "var(--token-color-surface-primary)",
    color: "var(--token-color-foreground-strong)",
    border: "1px solid var(--token-color-border-strong)",
    fontWeight: 600,
    padding: "0 14px",
  };
}

export function Button({ color = "primary", disabled, busy, icon, onClick, children }: ButtonProps) {
  const v = variantFor(color, !!busy);
  const style: CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: 7,
    height: 34,
    padding: v.padding,
    borderRadius: 7,
    font: "inherit",
    fontSize: 12.5,
    fontWeight: v.fontWeight,
    cursor: busy || disabled ? "default" : "pointer",
    background: v.background,
    color: v.color,
    border: v.border,
    boxShadow: v.boxShadow,
  };
  return (
    <button onClick={onClick} disabled={disabled || busy} style={style}>
      {busy ? <Spinner color={v.iconColor} /> : icon ? <Icon name={icon} size={15} color={v.iconColor} /> : null}
      {children}
    </button>
  );
}
