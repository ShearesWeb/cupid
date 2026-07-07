// App.tsx — app shell: TopBar, Sidebar, routing state, toasts (task-12).
// Ports reference/cca-console-design.html lines 182-268 (app frame, sidebar, topbar)
// and replaces the mock splash loader with a real "sync to load" empty state.
import { useCallback, useEffect, useMemo, useState } from "react";
import * as api from "./lib/api.ts";
import type { Snapshot } from "./lib/types.ts";
import { buildIndexes, type Indexes } from "./lib/indexes.ts";
import { fmtTime } from "./lib/format.ts";
import { Icon, Card, Button } from "./components/index.ts";
import { Toasts, type ToastItem, type ToastKind } from "./components/Toasts.tsx";
import { Allocations as AllocationsScreen } from "./screens/Allocations.tsx";
import { DetailPage as DetailPageScreen } from "./screens/DetailPage.tsx";

type Screen = "alloc" | "review";
type View = "position" | "applicant";
type Detail = { type: "applicant" | "position"; id: number } | null;
type Match = { aid: number; pid: number } | null;
type Theme = "light" | "dark";

interface CommitState {
  previewed: boolean;
  committed: boolean;
  archived: boolean;
  purged: boolean;
  committedCount: number;
  archiveRows: number;
}

const initialCommitState: CommitState = {
  previewed: false,
  committed: false,
  archived: false,
  purged: false,
  committedCount: 0,
  archiveRows: 0,
};

export interface UiState {
  snapshot: Snapshot | null;
  idx: Indexes | null;
  screen: Screen;
  view: View;
  search: string;
  page: number;
  detail: Detail;
  match: Match;
  syncing: boolean;
  running: boolean;
  commitState: CommitState;
  purgeText: string;
  toasts: ToastItem[];
}

export interface UiHandlers {
  doSync: () => void;
  doRun: () => void;
  openDetail: (type: "applicant" | "position", id: number) => void;
  openMatch: (aid: number, pid: number) => void;
  setScreen: (s: Screen) => void;
  setView: (v: View) => void;
  setSearch: (v: string) => void;
  setPage: (p: number) => void;
  toast: (kind: ToastKind, text: string) => void;
}

function errorMessage(e: unknown): string {
  if (e instanceof Error) return e.message;
  if (typeof e === "string") return e;
  try {
    return JSON.stringify(e);
  } catch {
    return String(e);
  }
}

let toastSeq = 0;

function App() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [theme, setTheme] = useState<Theme>("light");
  const [screen, setScreenState] = useState<Screen>("alloc");
  const [view, setViewState] = useState<View>("position");
  const [search, setSearchState] = useState("");
  const [page, setPage] = useState(0);
  const [detail, setDetail] = useState<Detail>(null);
  const [match, setMatch] = useState<Match>(null);
  const [syncing, setSyncing] = useState(false);
  const [running, setRunning] = useState(false);
  const [commitState, setCommitState] = useState<CommitState>(initialCommitState);
  const [purgeText, setPurgeText] = useState("");
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const idx = useMemo(() => (snapshot ? buildIndexes(snapshot) : null), [snapshot]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  const toast = (kind: ToastKind, text: string) => {
    toastSeq += 1;
    const id = toastSeq;
    setToasts((prev) => [...prev, { id, kind, text }]);
  };
  // Stable identity: ToastRow keys its 5s auto-dismiss timer effect on this
  // callback, so recreating it each render would reset the timer on every
  // App re-render (nav click, theme toggle, typing).
  const dismissToast = useCallback(
    (id: number) => setToasts((prev) => prev.filter((t) => t.id !== id)),
    [],
  );

  const doSync = async () => {
    if (syncing) return;
    setSyncing(true);
    try {
      const snap = await api.sync();
      setSnapshot(snap);
      setCommitState(initialCommitState);
      setPurgeText("");
      setDetail(null);
      setMatch(null);
    } catch (e) {
      toast("error", errorMessage(e));
    } finally {
      setSyncing(false);
    }
  };

  const doRun = async () => {
    if (running) return;
    setRunning(true);
    try {
      const snap = await api.runMatching();
      setSnapshot(snap);
    } catch (e) {
      toast("error", errorMessage(e));
    } finally {
      setRunning(false);
    }
  };

  const openDetail = (type: "applicant" | "position", id: number) => {
    setDetail({ type, id });
    setMatch(null);
  };
  const openMatch = (aid: number, pid: number) => setMatch({ aid, pid });
  const setScreen = (s: Screen) => {
    setScreenState(s);
    setDetail(null);
    setMatch(null);
  };
  const setView = (v: View) => {
    setViewState(v);
    setPage(0);
    setSearchState("");
    setDetail(null);
    setMatch(null);
  };
  const setSearch = (v: string) => {
    setSearchState(v);
    setPage(0);
  };

  const ui: UiState = {
    snapshot,
    idx,
    screen,
    view,
    search,
    page,
    detail,
    match,
    syncing,
    running,
    commitState,
    purgeText,
    toasts,
  };

  const handlers: UiHandlers = {
    doSync,
    doRun,
    openDetail,
    openMatch,
    setScreen,
    setView,
    setSearch,
    setPage,
    toast,
  };

  if (!snapshot) {
    return (
      <>
        <Splash syncing={syncing} onSync={doSync} />
        <Toasts toasts={toasts} onDismiss={dismissToast} />
      </>
    );
  }

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        display: "flex",
        flexDirection: "column",
        background: "var(--token-color-page-faint)",
        color: "var(--token-color-foreground-primary)",
        fontFamily: "var(--token-typography-font-stack-display)",
        fontSize: 13,
        overflow: "hidden",
      }}
    >
      <TopBar
        snapshot={snapshot}
        syncing={syncing}
        running={running}
        theme={theme}
        setTheme={setTheme}
        doSync={doSync}
        doRun={doRun}
      />
      <div style={{ flex: 1, display: "flex", minHeight: 0 }}>
        <Sidebar screen={screen} setScreen={setScreen} hasRun={snapshot.run !== null} />
        <main style={{ flex: 1, minWidth: 0, overflow: "auto" }}>
          {detail ? (
            <DetailPage ui={ui} handlers={handlers} onBack={() => setDetail(null)} />
          ) : screen === "alloc" ? (
            <Allocations ui={ui} handlers={handlers} />
          ) : (
            <Review ui={ui} handlers={handlers} />
          )}
        </main>
        {match ? <EventSidebar ui={ui} onClose={() => setMatch(null)} /> : null}
      </div>
      <Toasts toasts={toasts} onDismiss={dismissToast} />
    </div>
  );
}

// ---- Splash / empty state --------------------------------------------
function Splash({ syncing, onSync }: { syncing: boolean; onSync: () => void }) {
  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 16,
        background: "var(--token-color-page-faint)",
        color: "var(--token-color-foreground-faint)",
        fontFamily: "var(--token-typography-font-stack-display)",
        fontSize: 14,
      }}
    >
      <div
        style={{
          width: 48,
          height: 48,
          borderRadius: "50%",
          background: "#DB2A63",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          boxShadow: "0 1px 2px rgba(0,0,0,.18)",
        }}
      >
        <Icon name="heart" size={24} color="#fff" />
      </div>
      {syncing ? (
        <>
          <div style={{ fontSize: 15, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>
            Syncing&hellip;
          </div>
          <span
            style={{
              width: 18,
              height: 18,
              borderRadius: "50%",
              border: "2px solid var(--token-color-border-strong)",
              borderTopColor: "#DB2A63",
              display: "inline-block",
              animation: "cca-spin .7s linear infinite",
            }}
          />
        </>
      ) : (
        <>
          <div style={{ fontSize: 15, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>
            Sync to load the corpus
          </div>
          <div style={{ fontSize: 13, maxWidth: 320, textAlign: "center" }}>
            Cupid has no cached data yet. Sync with the database to pull applicants, positions, and appointments.
          </div>
          <Button color="primary" icon="download" onClick={onSync}>
            Sync
          </Button>
        </>
      )}
    </div>
  );
}

// ---- Top bar ------------------------------------------------------------
function TopBar({
  snapshot,
  syncing,
  running,
  theme,
  setTheme,
  doSync,
  doRun,
}: {
  snapshot: Snapshot;
  syncing: boolean;
  running: boolean;
  theme: Theme;
  setTheme: (t: Theme) => void;
  doSync: () => void;
  doRun: () => void;
}) {
  return (
    <header
      style={{
        height: 58,
        flex: "0 0 58px",
        display: "flex",
        alignItems: "center",
        gap: 22,
        padding: "0 20px",
        background: "var(--token-color-surface-primary)",
        boxShadow: "var(--token-surface-base-box-shadow)",
        zIndex: 30,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
        <div
          style={{
            width: 30,
            height: 30,
            borderRadius: "50%",
            background: "#DB2A63",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            boxShadow: "0 1px 2px rgba(0,0,0,.18)",
          }}
        >
          <Icon name="heart" size={16} color="#fff" />
        </div>
        <div style={{ fontSize: 17, fontWeight: 700, color: "var(--token-color-foreground-strong)", letterSpacing: "-0.4px" }}>
          Cupid
        </div>
      </div>
      <div style={{ flex: 1 }} />
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 7,
          color: "var(--token-color-foreground-faint)",
          fontSize: 12,
          whiteSpace: "nowrap",
        }}
      >
        <Icon name="clock" size={14} color="var(--token-color-foreground-faint)" />
        Synced{" "}
        <span style={{ color: "var(--token-color-foreground-primary)", fontWeight: 600 }}>
          {fmtTime(snapshot.syncedAt)}
        </span>
      </div>
      <RunPill running={running} hasRun={snapshot.run !== null} />
      <ThemeToggle theme={theme} setTheme={setTheme} />
      <div style={{ display: "flex", gap: 8 }}>
        <Button color="ghost" busy={syncing} icon="download" onClick={doSync}>
          {syncing ? "Syncing…" : "Sync"}
        </Button>
        <Button color="primary" busy={running} icon="layers" onClick={doRun}>
          {running ? "Running…" : snapshot.run !== null ? "Re-run" : "Run matching"}
        </Button>
      </div>
    </header>
  );
}

function RunPill({ running, hasRun }: { running: boolean; hasRun: boolean }) {
  let label: string;
  let color: string;
  let bg: string;
  let bd: string;
  let anim = "none";
  if (running) {
    label = "Matching…";
    color = "var(--token-color-foreground-action)";
    bg = "var(--token-color-surface-action)";
    bd = "var(--token-color-border-action)";
    anim = "cca-pulse 1s ease-in-out infinite";
  } else if (!hasRun) {
    label = "No run yet";
    color = "var(--token-color-foreground-faint)";
    bg = "var(--token-color-surface-faint)";
    bd = "var(--token-color-border-faint)";
  } else {
    label = "Fresh run ready";
    color = "var(--token-color-foreground-success-on-surface)";
    bg = "var(--token-color-surface-success)";
    bd = "var(--token-color-border-success)";
  }
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 7,
        height: 28,
        padding: "0 11px",
        borderRadius: 20,
        background: bg,
        border: `1px solid ${bd}`,
        color,
        fontSize: 12,
        fontWeight: 600,
        whiteSpace: "nowrap",
      }}
    >
      <span style={{ width: 7, height: 7, borderRadius: "50%", background: color, animation: anim }} />
      {label}
    </div>
  );
}

function ThemeToggle({ theme, setTheme }: { theme: Theme; setTheme: (t: Theme) => void }) {
  const seg = (val: Theme, label: string) => (
    <button
      key={val}
      onClick={() => theme !== val && setTheme(val)}
      style={{
        border: "none",
        cursor: "pointer",
        font: "inherit",
        fontSize: 11.5,
        fontWeight: 600,
        padding: "5px 10px",
        borderRadius: 6,
        background: theme === val ? "var(--token-color-surface-primary)" : "transparent",
        color: theme === val ? "var(--token-color-foreground-strong)" : "var(--token-color-foreground-faint)",
        boxShadow: theme === val ? "var(--token-surface-base-box-shadow)" : "none",
      }}
    >
      {label}
    </button>
  );
  return (
    <div style={{ display: "flex", gap: 2, padding: 2, borderRadius: 8, background: "var(--token-color-surface-strong)" }}>
      {seg("light", "Light")}
      {seg("dark", "Dark")}
    </div>
  );
}

// ---- Sidebar --------------------------------------------------------------
function Sidebar({
  screen,
  setScreen,
  hasRun,
}: {
  screen: Screen;
  setScreen: (s: Screen) => void;
  hasRun: boolean;
}) {
  const items: { id: Screen; label: string; icon: string }[] = [
    { id: "alloc", label: "Allocations", icon: "layers" },
    { id: "review", label: "Review & commit", icon: "lock" },
  ];
  return (
    <nav
      style={{
        width: 210,
        flex: "0 0 210px",
        display: "flex",
        flexDirection: "column",
        background: "var(--token-color-surface-primary)",
        borderRight: "1px solid var(--token-color-border-faint)",
        padding: "14px 10px",
        overflow: "auto",
      }}
    >
      <div
        style={{
          fontSize: 10,
          fontWeight: 700,
          letterSpacing: "0.8px",
          textTransform: "uppercase",
          color: "var(--token-color-foreground-faint)",
          padding: "4px 10px 10px",
        }}
      >
        Workspace
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
        {items.map((it) => {
          const active = screen === it.id;
          return (
            <button
              key={it.id}
              onClick={() => setScreen(it.id)}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 10,
                height: 38,
                padding: "0 11px",
                borderRadius: 8,
                font: "inherit",
                fontSize: 13,
                fontWeight: active ? 700 : 500,
                cursor: "pointer",
                textAlign: "left",
                background: active ? "var(--token-color-surface-strong)" : "transparent",
                color: active ? "var(--token-color-foreground-strong)" : "var(--token-color-foreground-primary)",
                border: "none",
                borderLeft: `3px solid ${active ? "#DB2A63" : "transparent"}`,
              }}
            >
              <Icon name={it.icon} size={16} color={active ? "#DB2A63" : "var(--token-color-foreground-faint)"} />
              {it.label}
            </button>
          );
        })}
      </div>
      <div style={{ flex: 1 }} />
      <Card padding="small" style={{ padding: 12 }}>
        <div
          style={{
            fontSize: 10,
            fontWeight: 700,
            letterSpacing: "0.6px",
            textTransform: "uppercase",
            color: "var(--token-color-foreground-faint)",
            marginBottom: 8,
          }}
        >
          Session
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 7, fontSize: 12 }}>
          <SLine
            k="Run"
            v={hasRun ? "Fresh" : "None"}
            c={hasRun ? "var(--token-color-foreground-success)" : "var(--token-color-foreground-faint)"}
          />
          <SLine k="Source" v="Live" c="var(--token-color-foreground-success)" />
        </div>
      </Card>
    </nav>
  );
}

function SLine({ k, v, c }: { k: string; v: string; c: string }) {
  return (
    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
      <span style={{ color: "var(--token-color-foreground-faint)" }}>{k}</span>
      <span
        style={{
          display: "flex",
          alignItems: "center",
          gap: 5,
          fontWeight: 700,
          color: "var(--token-color-foreground-strong)",
        }}
      >
        <span style={{ width: 7, height: 7, borderRadius: "50%", background: c }} />
        {v}
      </span>
    </div>
  );
}

// ---- Screens (placeholders; Tasks 14-16 replace the rest) ------------------
function Allocations({ ui, handlers }: { ui: UiState; handlers: UiHandlers }) {
  if (!ui.snapshot || !ui.idx) return null;
  return (
    <AllocationsScreen
      snapshot={ui.snapshot}
      idx={ui.idx}
      view={ui.view}
      search={ui.search}
      page={ui.page}
      onSetView={handlers.setView}
      onSetSearch={handlers.setSearch}
      onSetPage={handlers.setPage}
      onOpenDetail={handlers.openDetail}
      onOpenMatch={handlers.openMatch}
      hasRun={ui.snapshot.run !== null}
      running={ui.running}
      onRun={handlers.doRun}
    />
  );
}

function Review(_props: { ui: UiState; handlers: UiHandlers }) {
  return <div>review</div>;
}

function DetailPage({ ui, handlers, onBack }: { ui: UiState; handlers: UiHandlers; onBack: () => void }) {
  if (!ui.snapshot || !ui.idx || !ui.detail) return null;
  return (
    <DetailPageScreen
      detail={ui.detail}
      snapshot={ui.snapshot}
      idx={ui.idx}
      screen={ui.screen}
      onBack={onBack}
      onOpenMatch={handlers.openMatch}
      onOpenDetail={handlers.openDetail}
    />
  );
}

function EventSidebar(_props: { ui: UiState; onClose: () => void }) {
  return <div>match</div>;
}

export default App;
