// Badge.tsx — outlined small badge (reference line 272, typeBadge usage).
export type BadgeColor = "highlight" | "action" | "success" | "neutral";

export interface BadgeProps {
  color: BadgeColor;
  text: string;
}

// Every tone is a token triple, so both themes are covered by the palette.
const TONES: Record<BadgeColor, { background: string; foreground: string; border: string }> = {
  highlight: {
    background: "var(--token-color-surface-highlight)",
    foreground: "var(--token-color-foreground-highlight-on-surface)",
    border: "var(--token-color-border-highlight)",
  },
  action: {
    background: "var(--token-color-surface-action)",
    foreground: "var(--token-color-foreground-action)",
    border: "var(--token-color-border-action)",
  },
  success: {
    background: "var(--token-color-surface-success)",
    foreground: "var(--token-color-foreground-success-on-surface)",
    border: "var(--token-color-border-success)",
  },
  neutral: {
    background: "var(--token-color-surface-primary)",
    foreground: "var(--token-color-foreground-faint)",
    border: "var(--token-color-border-strong)",
  },
};

export function Badge({ color, text }: BadgeProps) {
  const tone = TONES[color];
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
