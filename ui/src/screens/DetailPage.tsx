// DetailPage.tsx — task-14: back-button wrapper that dispatches to ApplicantDetail/PositionDetail.
// Ports reference/cca-console-design.html renderDetailPage (221-227).
import { Card, Icon } from "../components/index.ts";
import type { Indexes } from "../lib/indexes.ts";
import type { Snapshot } from "../lib/types.ts";
import { ApplicantDetail } from "./ApplicantDetail.tsx";
import { PositionDetail } from "./PositionDetail.tsx";

export interface DetailPageProps {
  detail: { type: "applicant" | "position"; id: number };
  snapshot: Snapshot;
  idx: Indexes;
  screen: "alloc" | "prealloc" | "review";
  onBack: () => void;
  onOpenMatch: (aid: number, pid: number) => void;
  onOpenDetail: (type: "applicant" | "position", id: number) => void;
}

export function DetailPage(props: DetailPageProps) {
  const { detail, snapshot, idx, screen, onBack, onOpenMatch } = props;
  const isApp = detail.type === "applicant";
  return (
    <div style={{ padding: "24px 28px 48px", maxWidth: 860, margin: "0 auto" }}>
      <button
        onClick={onBack}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 6,
          height: 32,
          padding: "0 12px 0 9px",
          borderRadius: 7,
          border: "1px solid var(--token-color-border-strong)",
          background: "var(--token-color-surface-primary)",
          cursor: "pointer",
          font: "inherit",
          fontSize: 12.5,
          fontWeight: 600,
          color: "var(--token-color-foreground-strong)",
          marginBottom: 16,
        }}
      >
        <Icon name="chevron-left" size={15} color="var(--token-color-foreground-strong)" />
        Back to {screen === "alloc" ? "allocations" : screen === "prealloc" ? "preallocations" : "review"}
      </button>
      <Card padding="large">
        {isApp ? (
          <ApplicantDetail aid={detail.id} snapshot={snapshot} idx={idx} onOpenMatch={onOpenMatch} />
        ) : (
          <PositionDetail pid={detail.id} snapshot={snapshot} idx={idx} onOpenMatch={onOpenMatch} />
        )}
      </Card>
    </div>
  );
}
