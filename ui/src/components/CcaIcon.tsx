// CcaIcon.tsx — round CCA glyph (design ccaIcon). The mock keyed glyphs by
// CCA id; real CCAs pick one by kind, and a CCA's own image wins when it has one.
import { Icon } from "./Icon.tsx";
import { ccaKindIcon } from "../lib/directory.ts";

export function CcaIcon({ kind, imageUrl, size = 22 }: { kind: string | undefined; imageUrl?: string | null; size?: number }) {
  const frame = {
    width: size,
    height: size,
    flex: `0 0 ${size}px`,
    borderRadius: "50%",
    background: "var(--cupid-soft)",
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    overflow: "hidden",
  } as const;
  if (imageUrl) return <img src={imageUrl} alt="" style={{ ...frame, objectFit: "cover" }} />;
  return (
    <span style={frame}>
      <Icon name={ccaKindIcon(kind)} size={Math.round(size * 0.58)} color="var(--cupid-strong)" />
    </span>
  );
}
