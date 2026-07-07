// Tauri command rejections surface as Error, string, or opaque JSON — normalize to text.
export function errorMessage(e: unknown): string {
  if (e instanceof Error) return e.message;
  if (typeof e === "string") return e;
  try {
    return JSON.stringify(e);
  } catch {
    return String(e);
  }
}

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
