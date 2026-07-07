// Spinner.tsx — port of reference line 268 (cca-console-design.html)
export interface SpinnerProps {
  color?: string;
}

export function Spinner({ color }: SpinnerProps) {
  return (
    <span
      style={{
        width: 14,
        height: 14,
        borderRadius: "50%",
        border: "2px solid " + (color ? "rgba(255,255,255,.4)" : "var(--token-color-border-strong)"),
        borderTopColor: color || "#DB2A63",
        display: "inline-block",
        animation: "cca-spin .7s linear infinite",
      }}
    />
  );
}
