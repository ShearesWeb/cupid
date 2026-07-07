// Icon.tsx — CSS-mask tinting, per task-11 brief (verbatim implementation)
export interface IconProps {
  name: string;
  size?: number;
  color?: string;
}

export function Icon({ name, size = 16, color = "currentColor" }: IconProps) {
  const url = new URL(`../assets/icons/${name}-16.svg`, import.meta.url).href;
  return (
    <span
      aria-hidden
      style={{
        display: "inline-block",
        width: size,
        height: size,
        flexShrink: 0,
        backgroundColor: color,
        WebkitMaskImage: `url(${url})`,
        maskImage: `url(${url})`,
        WebkitMaskRepeat: "no-repeat",
        maskRepeat: "no-repeat",
        WebkitMaskSize: "contain",
        maskSize: "contain",
      }}
    />
  );
}
