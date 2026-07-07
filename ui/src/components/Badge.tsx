// Badge.tsx — outlined small badge (reference line 272, typeBadge usage).
export interface BadgeProps {
  color: "highlight" | "neutral";
  text: string;
}

export function Badge({ color, text }: BadgeProps) {
  const tone =
    color === "highlight"
      ? {
          background: "var(--token-color-surface-highlight)",
          foreground: "var(--token-color-foreground-highlight-on-surface)",
          border: "var(--token-color-border-highlight)",
        }
      : {
          background: "var(--token-color-surface-primary)",
          foreground: "var(--token-color-foreground-faint)",
          border: "var(--token-color-border-strong)",
        };
  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        height: 18,
        padding: "0 6px",
        borderRadius: 5,
        background: tone.background,
        color: tone.foreground,
        border: "1px solid " + tone.border,
        fontSize: 10,
        fontWeight: 700,
        letterSpacing: 0.3,
        textTransform: "uppercase",
        whiteSpace: "nowrap",
      }}
    >
      {text}
    </span>
  );
}
