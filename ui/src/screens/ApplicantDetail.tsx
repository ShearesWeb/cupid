// ApplicantDetail.tsx — task-14: applicant detail page.
// Ports reference/cca-console-design.html applicantDetail (556-574). All displayed values are
// looked up from snapshot/idx — this screen never recomputes statuses or quota rules.
import { Avatar, Badge, CcaIcon, ChoiceCoverage, Icon, MatchRow, QuotaWidget } from "../components/index.ts";
import { heldOutsideAllocation, periodLabel, typeLabel, type HeldPosition } from "../lib/directory.ts";
import { pk, type Indexes } from "../lib/indexes.ts";
import type { DirectorySnapshot, Snapshot } from "../lib/types.ts";
import { DetailHero, OutcomeLegend, Section } from "./shared.tsx";

export interface ApplicantDetailProps {
  aid: number;
  snapshot: Snapshot;
  directory: DirectorySnapshot | null;
  idx: Indexes;
  onOpenMatch: (aid: number, pid: number) => void;
  onOpenCca: (ccaId: number) => void;
}

export function ApplicantDetail({ aid, snapshot, directory, idx, onOpenMatch, onOpenCca }: ApplicantDetailProps) {
  const a = idx.appById.get(aid);
  if (!a) return null;
  const hasRun = snapshot.run !== null;
  const quota = idx.quotaByApp.get(aid);

  const posLabel = (pid: number) => {
    const pos = idx.posById.get(pid);
    const cca = pos ? idx.ccaById.get(pos.ccaId) : undefined;
    return `${cca?.name ?? ""} · ${pos?.name ?? ""}`;
  };
  const posIcon = (pid: number) => {
    const pos = idx.posById.get(pid);
    return pos ? <CcaIcon kind={idx.ccaById.get(pos.ccaId)?.kind} size={20} /> : null;
  };

  const existingPos = snapshot.committed.filter((c) => c.applicantId === aid).map((c) => c.positionId);
  const otherHeld = heldOutsideAllocation(directory, aid, new Set(existingPos));
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
      <Section title="Existing positions">
        {otherHeld.length ? (
          <div style={{ display: "flex", flexDirection: "column" }}>
            {otherHeld.map((h) => (
              <HeldPositionRow key={h.position.id} held={h} onClick={() => onOpenCca(h.position.ccaId)} />
            ))}
          </div>
        ) : (
          <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)", fontStyle: "italic" }}>
            No other positions.
          </div>
        )}
      </Section>
      <Section title="Committee positions">
        {existingPos.length ? (
          <div style={{ display: "flex", flexDirection: "column" }}>
            {existingPos.map((pid, k) => (
              <MatchRow
                key={`e${pid}`}
                num={k + 1}
                name={posLabel(pid)}
                leading={posIcon(pid)}
                status="existing"
                statusLabel="Appointment"
                onClick={() => onOpenMatch(aid, pid)}
              />
            ))}
          </div>
        ) : (
          <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)", fontStyle: "italic" }}>
            No committee positions.
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
                leading={posIcon(pid)}
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
                leading={posIcon(pid)}
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

// Untinted on purpose: these roles sit outside the run, so the outcome legend
// colours above must not apply to them.
function HeldPositionRow({ held, onClick }: { held: HeldPosition; onClick: () => void }) {
  const { appointment, position, cca } = held;
  return (
    <button
      onClick={onClick}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 10,
        width: "100%",
        padding: "11px 12px",
        border: "none",
        borderBottom: "1px solid var(--token-color-border-faint)",
        background: "transparent",
        cursor: "pointer",
        font: "inherit",
        textAlign: "left",
      }}
    >
      <CcaIcon kind={cca?.kind} imageUrl={cca?.imageUrl} size={20} />
      <span
        style={{
          flex: 1,
          minWidth: 0,
          fontSize: 13.5,
          fontWeight: 600,
          color: "var(--token-color-foreground-strong)",
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
        }}
      >
        {cca?.name ?? ""} · {position.name}
      </span>
      <Badge color="neutral" text={typeLabel(position.positionType)} />
      {appointment.status !== "existing" ? <Badge color="highlight" text="pending" /> : null}
      <span
        style={{
          fontSize: 11,
          fontWeight: 600,
          fontFamily: "var(--token-typography-font-stack-code)",
          color: "var(--token-color-foreground-faint)",
          whiteSpace: "nowrap",
          minWidth: 52,
          textAlign: "right",
        }}
      >
        {periodLabel(appointment.commitmentPeriod)}
      </span>
      <Icon name="chevron-right" size={15} color="var(--token-color-foreground-faint)" />
    </button>
  );
}
