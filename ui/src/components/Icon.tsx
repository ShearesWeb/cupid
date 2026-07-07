import type { CSSProperties } from "react";

export interface IconProps {
  name: string;
  size?: number;
  color?: string;
}

// The mask itself is applied by the .cca-icon rule in base.css; this
// component only resolves the asset URL and supplies size/color.
// The url MUST be double-quoted: Vite inlines small SVGs as data: URLs
// containing single quotes, which are bad-url-tokens inside an unquoted
// CSS url() and silently invalidate the whole declaration.
export function Icon({ name, size = 16, color = "currentColor" }: IconProps) {
  const url = new URL(`../assets/icons/${name}-16.svg`, import.meta.url).href;
  return (
    <span
      aria-hidden
      className="cca-icon"
      style={
        {
          width: size,
          height: size,
          backgroundColor: color,
          "--icon-url": `url("${url}")`,
        } as CSSProperties
      }
    />
  );
}
