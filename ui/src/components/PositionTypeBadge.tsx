// PositionTypeBadge.tsx — the one place a position type maps to a colour, so
// the badge reads the same on cards, detail headers and pickers.
import { Badge, type BadgeColor } from "./Badge.tsx";
import type { PositionType } from "../lib/types.ts";

const TYPE_COLOR: Record<PositionType, BadgeColor> = {
  main: "highlight",
  block: "action",
  sub: "success",
};

export function PositionTypeBadge({ type }: { type: PositionType }) {
  return <Badge color={TYPE_COLOR[type]} text={type} />;
}
