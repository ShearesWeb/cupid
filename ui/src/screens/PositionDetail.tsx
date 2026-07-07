// PositionDetail.tsx — task-14: position detail page.
// Ports reference/cca-console-design.html positionDetail (576-591). All displayed values are
// looked up from snapshot/idx — this screen never recomputes statuses or seat ordering.
import { Badge, MatchRow } from "../components/index.ts";
import { pk, type Indexes } from "../lib/indexes.ts";
import type { Snapshot, Status } from "../lib/types.ts";
import { DetailHero, IconTile, Section } from "./shared.tsx";

export interface PositionDetailProps {
  pid: number;
  snapshot: Snapshot;
  idx: Indexes;
  onOpenMatch: (aid: number, pid: number) => void;
}

// Seats are only ever existing/allocated/appealed (see snapshot::build_seats), but the shared
// Status union is broader, so this falls back to "New" for any other value.
function seatLabel(status: Status): string {
  if (status === "existing") return "Appointment";
  if (status === "appealed") return "Appeal";
  return "New";
}

export function PositionDetail({ pid, snapshot, idx, onOpenMatch }: PositionDetailProps) {
  const pos = idx.posById.get(pid);
  if (!pos) return null;
  const cca = idx.ccaById.get(pos.ccaId);
  const hasRun = snapshot.run !== null;

  const seated = idx.seatsByPos.get(pid) ?? [];
  const filled = seated.length;
  const full = filled >= pos.capacity;
  const posAppeals = snapshot.appeals.filter((ap) => ap.positionId === pid);

  return (
    <>
      <DetailHero
        leading={<IconTile name="folder" />}
        eyebrow={cca?.name ?? ""}
        title={pos.name}
        badge={<Badge color={pos.type === "main" ? "highlight" : "neutral"} text={pos.type} />}
        rightValue={`${filled}/${pos.capacity}`}
        rightLabel="filled"
        rightColor={full ? "var(--token-color-foreground-strong)" : "var(--token-color-foreground-critical-on-surface)"}
      />
      <Section title="Filled by">
        {seated.length ? (
          <div style={{ display: "flex", flexDirection: "column" }}>
            {seated.map((s, k) => (
              <MatchRow
                key={s.applicantId}
                num={k + 1}
                name={idx.appById.get(s.applicantId)?.name ?? ""}
                status={s.status}
                statusLabel={seatLabel(s.status)}
                onClick={() => onOpenMatch(s.applicantId, pid)}
              />
            ))}
          </div>
        ) : (
          <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-critical-on-surface)", fontStyle: "italic" }}>
            Empty — no one here yet.
          </div>
        )}
      </Section>
      {posAppeals.length ? (
        <Section title="Appealed (quota-exempt)">
          <div style={{ display: "flex", flexDirection: "column" }}>
            {posAppeals.map((ap, k) => {
              const o = idx.outcomeByPair.get(pk(ap.applicantId, pid));
              return (
                <MatchRow
                  key={`pa${k}`}
                  num={k + 1}
                  name={idx.appById.get(ap.applicantId)?.name ?? ""}
                  status={o?.status ?? "neutral"}
                  statusLabel={o?.label ?? "—"}
                  onClick={() => onOpenMatch(ap.applicantId, pid)}
                />
              );
            })}
          </div>
        </Section>
      ) : null}
      <Section title="Chair's ranking">
        <div style={{ display: "flex", flexDirection: "column" }}>
          {pos.chairRank.map((cid, i) => {
            const o = idx.outcomeByPair.get(pk(cid, pid));
            const pr = idx.prefRankOf(cid, pid);
            return (
              <MatchRow
                key={cid}
                num={i + 1}
                name={idx.appById.get(cid)?.name ?? ""}
                sub={hasRun || o?.status === "noreturn" ? (o?.detail ?? null) : null}
                meta={pr ? `their pref #${pr}` : "not in their prefs"}
                metaColor={pr ? undefined : "var(--token-color-foreground-critical-on-surface)"}
                status={o?.status ?? "neutral"}
                statusLabel={o?.label ?? "—"}
                onClick={() => onOpenMatch(cid, pid)}
              />
            );
          })}
        </div>
      </Section>
    </>
  );
}
