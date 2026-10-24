// ApplicantDetail.tsx — task-14: applicant detail page, and the profile page
// for every resident. Ports reference/cca-console-design.html applicantDetail
// (556-574). Residents outside the allocation (no preferences, no allocatable
// seat) get the same header and their CCA memberships instead. Every profile
// lists the resident's Sports and Cultural CCAs, and every profile
// ends with the CCA caps section, where appeals are approved or withdrawn.
// All displayed values are looked up from snapshot/idx/directory — this screen
// never recomputes statuses, quota or cap rules.
import type { ReactNode } from "react";
import { Avatar, CapSummary, CcaIcon, ChoiceCoverage, MatchRow, QuotaWidget } from "../components/index.ts";
import { isEditable, periodLabel, typeLabel } from "../lib/directory.ts";
import { pk, type Indexes } from "../lib/indexes.ts";
import { useResidentCaps } from "../lib/residentCaps.ts";
import type { AppealChange, CcaKind, CommitteeCapEntry, DirectoryAppointment, DirectorySnapshot, Snapshot } from "../lib/types.ts";
import type { MemberRef } from "./Ccas.tsx";
import { ResidentCaps } from "./ResidentCaps.tsx";
import { DetailHero, OutcomeLegend, Section } from "./shared.tsx";

export interface ApplicantDetailProps {
  aid: number;
  snapshot: Snapshot;
  idx: Indexes;
  directory: DirectorySnapshot;
  onOpenMatch: (aid: number, pid: number) => void;
  loadCaps: (userId: number) => Promise<CommitteeCapEntry | null>;
  onSetAppeals: (changes: AppealChange[]) => Promise<boolean>;
  /** Opens the seat's CCA with the Edit member window. */
  onOpenMember: (member: MemberRef) => void;
}

export function ApplicantDetail(props: ApplicantDetailProps) {
  const { aid, snapshot, idx, directory, onOpenMatch } = props;
  const a = idx.appById.get(aid);
  const { entry, loading } = useResidentCaps(aid, directory, props.loadCaps);
  const caps = <ResidentCaps key={aid} userId={aid} directory={directory} entry={entry} onSetAppeals={props.onSetAppeals} />;
  const capSummary = <CapSummary entry={entry} loading={loading} rules={directory.committeeCaps.rules} />;
  if (!a) return <ResidentProfile {...props} caps={caps} capSummary={capSummary} />;
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
  const newPos = hasRun ? idx.newAllocations.filter((o) => o.applicantId === aid).map((o) => o.positionId) : [];
  const myPreallocations = snapshot.preallocations.filter((ap) => ap.applicantId === aid);

  return (
    <>
      <DetailHero
        leading={<Avatar name={a.name} size="medium" />}
        eyebrow={a.email}
        title={a.name}
        right={<QuotaWidget quota={quota ?? null} hasRun={hasRun} caps={capSummary} />}
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
                leading={posIcon(pid)}
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
      <Memberships {...props} title="Sports & Culture" empty="No Sports or Cultural CCAs." include={isSportsCulture} />
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
      {caps}
    </>
  );
}

/** Profile for a resident outside the allocation: header, memberships, caps. */
function ResidentProfile(props: ApplicantDetailProps & { caps: ReactNode; capSummary: ReactNode }) {
  const { aid, directory, caps, capSummary } = props;
  const user = directory.users.find((u) => u.id === aid);
  if (!user) return null;
  return (
    <>
      <DetailHero
        leading={<Avatar name={user.name} size="medium" />}
        eyebrow={user.email}
        title={user.name}
        right={<QuotaWidget quota={null} hasRun={false} caps={capSummary} />}
      />
      <Memberships {...props} title="Committees & other CCAs" empty="No committee or other CCA memberships." include={(kind) => !isSportsCulture(kind)} />
      <Memberships {...props} title="Sports & Culture" empty="No Sports or Cultural CCAs." include={isSportsCulture} />
      {caps}
    </>
  );
}

const isSportsCulture = (kind: CcaKind | undefined) => kind === "sports" || kind === "culture";

const TEAM_STATUS: Record<DirectoryAppointment["teamStatus"], string | null> = {
  none: null,
  shortlisted: "Shortlisted",
  reserve: "Reserve",
  "main-team": "Main team",
  varsity: "Varsity",
};

/** A resident's directory memberships in the CCA kinds `include` accepts. */
function Memberships({
  aid,
  directory,
  onOpenMember,
  title,
  empty,
  include,
}: ApplicantDetailProps & { title: string; empty: string; include: (kind: CcaKind | undefined) => boolean }) {
  const positionById = new Map(directory.positions.map((p) => [p.id, p]));
  const ccaById = new Map(directory.ccas.map((c) => [c.id, c]));
  const rows = directory.appointments
    .filter((x) => x.userId === aid)
    .map((x) => {
      const pos = positionById.get(x.positionId);
      return { x, pos, cca: pos ? ccaById.get(pos.ccaId) : undefined };
    })
    .filter(({ cca }) => include(cca?.kind))
    .sort((a, b) => (a.cca?.name ?? "").localeCompare(b.cca?.name ?? ""));
  return (
    <Section title={title}>
      {rows.length ? (
        <div style={{ display: "flex", flexDirection: "column" }}>
          {rows.map(({ x, pos, cca }, k) => (
            <MatchRow
              key={x.positionId}
              num={k + 1}
              name={`${cca?.name ?? ""} · ${pos?.name ?? ""}`}
              leading={<CcaIcon kind={cca?.kind} imageUrl={cca?.imageUrl} size={20} />}
              sub={[periodLabel(x.commitmentPeriod), TEAM_STATUS[x.teamStatus]].filter(Boolean).join(" · ")}
              status="existing"
              statusLabel={pos ? typeLabel(pos.positionType) : "Member"}
              // Resident seats are read-only: there is no Edit member window to open.
              onClick={() => {
                if (pos && isEditable(pos)) onOpenMember({ userId: aid, positionId: x.positionId });
              }}
            />
          ))}
        </div>
      ) : (
        <div style={{ fontSize: 12.5, color: "var(--token-color-foreground-faint)", fontStyle: "italic" }}>{empty}</div>
      )}
    </Section>
  );
}
