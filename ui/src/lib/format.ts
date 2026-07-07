export function fmtTime(iso: string | null): string {
  if (iso === null) return "—";
  return new Date(iso).toLocaleString("en-SG", {
    day: "2-digit",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
}
