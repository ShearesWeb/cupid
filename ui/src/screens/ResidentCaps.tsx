// ResidentCaps.tsx — the "CCA caps" section of a resident's profile: their
// committee and Sports/Cultural counts, any cap flags, and the two appeal
// checkboxes. Ticking or unticking is a draft until Save changes. Rust decides
// every count and flag; appeals are saved to a local file, never the database.
import { useState } from "react";
import { AppealCheckbox, Button, CapTags } from "../components/index.ts";
import { APPEAL_COPY, capCountText } from "../lib/directory.ts";
import type { AppealChange, AppealKind, CommitteeCapEntry, DirectorySnapshot } from "../lib/types.ts";
import { Section } from "./shared.tsx";

export interface ResidentCapsProps {
  userId: number;
  directory: DirectorySnapshot;
  /** From useResidentCaps: counts for anyone with a committee seat, flagged or not. */
  entry: CommitteeCapEntry | null;
  onSetAppeals: (changes: AppealChange[]) => Promise<boolean>;
}

const KINDS: AppealKind[] = ["committee", "combined"];

export function ResidentCaps({ userId, directory, entry, onSetAppeals }: ResidentCapsProps) {
  // Appealed residents are always in the report, so a missing entry means no appeal.
  const reported = directory.committeeCaps.entries.find((e) => e.userId === userId);
  const saved: Record<AppealKind, boolean> = { committee: !!reported?.appealed, combined: !!reported?.combinedAppealed };
  const [draft, setDraft] = useState<Partial<Record<AppealKind, boolean>>>({});
  const [saving, setSaving] = useState(false);

  const shown = { ...saved, ...draft };
  const changes: AppealChange[] = KINDS.filter((k) => shown[k] !== saved[k]).map((kind) => ({ userId, kind, appealed: shown[kind] }));
  const toggle = (kind: AppealKind) => setDraft((d) => ({ ...d, [kind]: !shown[kind] }));
  const save = async () => {
    setSaving(true);
    if (await onSetAppeals(changes)) setDraft({});
    setSaving(false);
  };
  const view = entry;

  return (
    <Section title="CCA caps">
      <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
        {view ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 6, flexWrap: "wrap" }}>
              <CapTags entry={view} />
              {!view.overCap && !view.overCombinedCap && !view.duplicateCcas.length ? (
                <span style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)" }}>Within all caps.</span>
              ) : null}
            </div>
            <div style={{ fontSize: 12.5, fontWeight: 600, color: "var(--token-color-foreground-strong)", fontFamily: "var(--token-typography-font-stack-code)" }}>
              {view.counts.map(capCountText).join("   ")}
            </div>
          </div>
        ) : (
          <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)", fontStyle: "italic" }}>No counted CCAs.</div>
        )}
        <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
          {KINDS.map((kind) => (
            <AppealCheckbox key={kind} {...APPEAL_COPY[kind]} checked={shown[kind]} busy={saving} onChange={() => toggle(kind)} />
          ))}
          <span style={{ flex: 1 }} />
          {changes.length ? (
            <Button color="ghost" icon="x" disabled={saving} onClick={() => setDraft({})}>
              Discard
            </Button>
          ) : null}
          <Button color="primary" icon="check" busy={saving} disabled={!changes.length} onClick={() => void save()}>
            Save changes
          </Button>
        </div>
        <div style={{ fontSize: 11.5, color: "var(--token-color-foreground-faint)" }}>
          {changes.length ? "Unsaved: press Save changes to keep it." : "Tick to approve an appeal, untick to withdraw it. Applies across all their CCAs."}
        </div>
      </div>
    </Section>
  );
}
