// Avatar.tsx — initials circle, per task-11 brief.
export interface AvatarProps {
  name: string;
  size?: "small" | "medium";
}

function initials(name: string): string {
  const words = name.trim().split(/\s+/).filter(Boolean);
  return words
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase() ?? "")
    .join("");
}

export function Avatar({ name, size = "medium" }: AvatarProps) {
  const px = size === "small" ? 24 : 32;
  const fontSize = size === "small" ? 10.5 : 13;
  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        width: px,
        height: px,
        borderRadius: "50%",
        background: "var(--token-color-surface-strong)",
        color: "var(--token-color-foreground-faint)",
        fontWeight: 700,
        fontSize,
        flexShrink: 0,
      }}
    >
      {initials(name)}
    </span>
  );
}
