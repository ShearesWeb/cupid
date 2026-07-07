// Card.tsx — surface-primary panel, per task-11 brief.
import type { CSSProperties, ReactNode } from "react";

export interface CardProps {
  level?: "base";
  padding?: "none" | "small" | "large";
  style?: CSSProperties;
  children?: ReactNode;
}

const PADDING: Record<"none" | "small" | "large", number> = {
  none: 0,
  small: 12,
  large: 24,
};

export function Card({ padding = "none", style, children }: CardProps) {
  const base: CSSProperties = {
    background: "var(--token-color-surface-primary)",
    borderRadius: 12,
    boxShadow: "var(--token-surface-base-box-shadow)",
    padding: PADDING[padding],
  };
  return <div style={{ ...base, ...style }}>{children}</div>;
}
