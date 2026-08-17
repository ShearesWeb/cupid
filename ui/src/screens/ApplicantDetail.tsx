// ApplicantDetail.tsx — task-14: applicant detail page.
// Ports reference/cca-console-design.html applicantDetail (556-574). All displayed values are
// looked up from snapshot/idx — this screen never recomputes statuses or quota rules.
import { Avatar, ChoiceCoverage, MatchRow, QuotaWidget } from "../components/index.ts";
import { pk, type Indexes } from "../lib/indexes.ts";
import type { Snapshot } from "../lib/types.ts";
import { DetailHero, OutcomeLegend, Section } from "./shared.tsx";

export interface ApplicantDetailProps {
  aid: number;
  snapshot: Snapshot;
  idx: Indexes;
  onOpenMatch: (aid: number, pid: number) => void;
}

export function ApplicantDetail({ aid, snapshot, idx, onOpenMatch }: ApplicantDetailProps) {
  const a = idx.appById.get(aid);
  if (!a) return null;
  const hasRun = snapshot.run !== null;
  const quota = idx.quotaByApp.get(aid);

  const posLabel = (pid: number) => {
    const pos = idx.posById.get(pid);
    const cca = pos ? idx.ccaById.get(pos.ccaId) : undefined;
    return `${cca?.name ?? ""} · ${pos?.name ?? ""}`;
  };

  const existingPos = snapshot.committed.filter((c) => c.applicantId === aid).map((c) => c.positionId);
  const newPos = hasRun ? idx.newAllocations.filter((o) => o.applicantId === aid).map((o) => o.positionId) : [];
  const myPreallocations = snapshot.preallocations.filter((ap) => ap.applicantId === aid);

  return (
    <>
      <DetailHero
        leading={<Avatar name={a.name} size="medium" />}
        eyebrow={a.email}
        title={a.name}
        right={quota ? <QuotaWidget quota={quota} hasRun={hasRun} /> : null}
      />
      <div style={{ margin: "-8px 0 20px" }}>
        <OutcomeLegend />
      </div>
      <Section title="Existing appointments">
        {existingPos.length ? (
          <div style={{ display: "flex", flexDirection: "column" }}>
            {existingPos.map((pid, k) => (
              <MatchRow
                key={`e${pid}`}
                num={k + 1}
                name={posLabel(pid)}
                status="existing"
                statusLabel="Appointment"
                onClick={() => onOpenMatch(aid, pid)}
              />
            ))}
          </div>
        ) : (
          <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)", fontStyle: "italic" }}>
            No existing appointments.
          </div>
        )}
      </Section>
      {hasRun ? (
        <Section title="New allocations">
          {newPos.length ? (
            <div style={{ display: "flex", flexDirection: "column" }}>
              {newPos.map((pid, k) => (
                <MatchRow
                  key={`n${pid}`}
                  num={k + 1}
                  name={posLabel(pid)}
                  status="allocated"
                  statusLabel="New"
                  onClick={() => onOpenMatch(aid, pid)}
                />
              ))}
            </div>
          ) : (
            <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)", fontStyle: "italic" }}>
              No new allocations this run.
            </div>
          )}
        </Section>
      ) : null}
      {myPreallocations.length ? (
        <Section title="Preallocations">
          <div style={{ display: "flex", flexDirection: "column" }}>
            {myPreallocations.map((ap, k) => {
              const o = idx.outcomeByPair.get(pk(aid, ap.positionId));
              return (
                <MatchRow
                  key={`ap${k}`}
                  num={k + 1}
                  name={posLabel(ap.positionId)}
                  status={o?.status ?? "neutral"}
                  statusLabel={o?.label ?? "—"}
                  onClick={() => onOpenMatch(aid, ap.positionId)}
                />
              );
            })}
          </div>
        </Section>
      ) : null}
      <Section title="Ranked preferences">
        <div style={{ marginBottom: 10 }}>
          <ChoiceCoverage ranked={a.prefs.length} />
        </div>
        <div style={{ display: "flex", flexDirection: "column" }}>
          {a.prefs.map((pid, i) => {
            const o = idx.outcomeByPair.get(pk(aid, pid));
            const cr = idx.chairRankOf(pid, aid);
            return (
              <MatchRow
                key={pid}
                num={i + 1}
                name={posLabel(pid)}
                sub={hasRun ? (o?.detail ?? null) : null}
                meta={cr ? `chair #${cr}` : null}
                status={o?.status ?? "neutral"}
                statusLabel={o?.label ?? "—"}
                onClick={() => onOpenMatch(aid, pid)}
              />
            );
          })}
        </div>
      </Section>
    </>
  );
}
