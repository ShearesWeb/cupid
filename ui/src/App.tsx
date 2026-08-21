// App.tsx — app shell: TopBar, Sidebar, routing state, toasts (task-12).
// Ports reference/cca-console-design.html lines 182-268 (app frame, sidebar, topbar)
// and replaces the mock splash loader with a real "sync to load" empty state.
import { useCallback, useEffect, useMemo, useState } from "react";
import * as api from "./lib/api.ts";
import type { PositionType, Snapshot } from "./lib/types.ts";
import { buildIndexes, type Indexes } from "./lib/indexes.ts";
import { errorMessage, fmtTime } from "./lib/format.ts";
import { Icon, Card, Button } from "./components/index.ts";
import { Toasts, type ToastItem, type ToastKind } from "./components/Toasts.tsx";
import { UpdatePrompt } from "./components/UpdatePrompt.tsx";
import {
  appVersion,
  checkForUpdate,
  dismissUpdate,
  installUpdate,
  updatesSupported,
  type PendingUpdate,
} from "./lib/updater.ts";
import { Allocations as AllocationsScreen } from "./screens/Allocations.tsx";
import { DetailPage as DetailPageScreen } from "./screens/DetailPage.tsx";
import { EventSidebar as EventSidebarScreen } from "./screens/EventSidebar.tsx";
import { Preallocations as PreallocationsScreen } from "./screens/Preallocations.tsx";
import { Review as ReviewScreen, type CommitState } from "./screens/Review.tsx";
import { TextInput } from "./components/TextInput.tsx";

type Screen = "alloc" | "prealloc" | "review";
type View = "position" | "applicant";
type TypeFilter = "all" | PositionType;
type Detail = { type: "applicant" | "position"; id: number } | null;
type Match = { aid: number; pid: number } | null;
type Theme = "light" | "dark";

const initialCommitState: CommitState = {
  previewed: false,
  accessChecked: false,
  exported: false,
  archived: false,
  purged: false,
  exportedRows: 0,
  branch: null,
  prUrl: null,
  archiveRows: 0,
  excluded: [],
};

export interface UiState {
  snapshot: Snapshot | null;
  idx: Indexes | null;
  screen: Screen;
  view: View;
  search: string;
  typeFilter: TypeFilter;
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
  setTypeFilter: (v: TypeFilter) => void;
  setPage: (p: number) => void;
  toast: (kind: ToastKind, text: string) => void;
  setCommitState: (s: CommitState | ((prev: CommitState) => CommitState)) => void;
  setPurgeText: (v: string) => void;
  addPreallocation: (applicantId: number, positionId: number, note: string | null) => Promise<boolean>;
  removePreallocation: (applicantId: number, positionId: number) => Promise<boolean>;
  applySnapshot: (snap: Snapshot) => void;
}

let toastSeq = 0;

function App() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [theme, setTheme] = useState<Theme>("light");
  const [screen, setScreenState] = useState<Screen>("alloc");
  const [view, setViewState] = useState<View>("position");
  const [search, setSearchState] = useState("");
  const [typeFilter, setTypeFilterState] = useState<TypeFilter>("all");
  const [page, setPage] = useState(0);
  const [detail, setDetail] = useState<Detail>(null);
  const [match, setMatch] = useState<Match>(null);
  const [syncing, setSyncing] = useState(false);
  const [running, setRunning] = useState(false);
  const [commitState, setCommitState] = useState<CommitState>(initialCommitState);
  const [purgeText, setPurgeText] = useState("");
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  // Connection state: null until credentials are supplied (DATABASE_URL may
  // seed it backend-side, discovered by the connection_info query on mount).
  const [connLoaded, setConnLoaded] = useState(false);
  const [connInfo, setConnInfo] = useState<string | null>(null);
  const [connBusy, setConnBusy] = useState(false);
  const [connError, setConnError] = useState<string | null>(null);
  const [changingConn, setChangingConn] = useState(false);

  // Updates: version for the sidebar, a pending release for the modal.
  const [version, setVersion] = useState<string | null>(null);
  const [pendingUpdate, setPendingUpdate] = useState<PendingUpdate | null>(null);
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [installingUpdate, setInstallingUpdate] = useState(false);

  const idx = useMemo(() => (snapshot ? buildIndexes(snapshot) : null), [snapshot]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  useEffect(() => {
    api
      .connectionInfo()
      .then((info) => setConnInfo(info))
      .catch(() => setConnInfo(null))
      .finally(() => setConnLoaded(true));
  }, []);

  // Launch check: silent on failure (offline, no release yet) so a bad network
  // never blocks the console. The manual check in the sidebar does report.
  useEffect(() => {
    appVersion()
      .then(setVersion)
      .catch(() => setVersion(null));
    checkForUpdate()
      .then(setPendingUpdate)
      .catch(() => {});
  }, []);

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
      snap.warnings.forEach((w) => toast("error", w));
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
      setCommitState(initialCommitState);
      setPurgeText("");
    } catch (e) {
      toast("error", errorMessage(e));
    } finally {
      setRunning(false);
    }
  };

  // Preallocation changes invalidate the run server-side (the snapshot comes
  // back with run: null), so the review stepper resets alongside it.
  const addPreallocation = async (applicantId: number, positionId: number, note: string | null) => {
    try {
      const snap = await api.addPreallocation(applicantId, positionId, note);
      setSnapshot(snap);
      setCommitState(initialCommitState);
      setPurgeText("");
      toast("success", "Preallocation granted. Re-run matching to apply it.");
      return true;
    } catch (e) {
      toast("error", errorMessage(e));
      return false;
    }
  };

  const removePreallocation = async (applicantId: number, positionId: number) => {
    try {
      const snap = await api.removePreallocation(applicantId, positionId);
      setSnapshot(snap);
      setCommitState(initialCommitState);
      setPurgeText("");
      toast("success", "Preallocation removed. Re-run matching to apply it.");
      return true;
    } catch (e) {
      toast("error", errorMessage(e));
      return false;
    }
  };

  // Verify credentials, adopt the new target, and pull its corpus. The old
  // snapshot dies with the old database; a connect failure leaves everything
  // untouched and surfaces inline on the form (toasts vanish too fast for
  // credential errors).
  const doConnect = async (projectRef: string, password: string, region: string) => {
    if (connBusy) return;
    setConnBusy(true);
    setConnError(null);
    try {
      const label = await api.connect(projectRef, password, region.trim() ? region.trim() : null);
      localStorage.setItem("cupid.projectRef", projectRef.trim());
      localStorage.setItem("cupid.region", region.trim());
      setConnInfo(label);
      setChangingConn(false);
      setSnapshot(null);
      setCommitState(initialCommitState);
      setPurgeText("");
      setDetail(null);
      setMatch(null);
      toast("success", `Connected to ${label}.`);
    } catch (e) {
      setConnError(errorMessage(e));
      return;
    } finally {
      setConnBusy(false);
    }
    await doSync();
  };

  // Replace the snapshot without touching stepper state: commit and purge
  // return fresh corpora mid-finalize, and the stepper must keep its place.
  const applySnapshot = (snap: Snapshot) => setSnapshot(snap);

  const doCheckUpdate = async () => {
    if (checkingUpdate) return;
    if (!updatesSupported) {
      toast("success", "Update checks are disabled in dev builds.");
      return;
    }
    setCheckingUpdate(true);
    try {
      const found = await checkForUpdate();
      if (found) setPendingUpdate(found);
      else toast("success", version ? `Cupid ${version} is the latest release.` : "You are on the latest release.");
    } catch (e) {
      toast("error", `Update check failed: ${errorMessage(e)}`);
    } finally {
      setCheckingUpdate(false);
    }
  };

  const doInstallUpdate = async () => {
    if (!pendingUpdate || installingUpdate) return;
    setInstallingUpdate(true);
    try {
      await installUpdate(pendingUpdate);
    } catch (e) {
      setInstallingUpdate(false);
      setPendingUpdate(null);
      toast("error", `Update failed: ${errorMessage(e)}`);
    }
  };

  const doDismissUpdate = () => {
    if (installingUpdate || !pendingUpdate) return;
    void dismissUpdate(pendingUpdate);
    setPendingUpdate(null);
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
  const setTypeFilter = (v: TypeFilter) => {
    setTypeFilterState(v);
    setPage(0);
  };

  const ui: UiState = {
    snapshot,
    idx,
    screen,
    view,
    search,
    typeFilter,
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
    setTypeFilter,
    setPage,
    toast,
    setCommitState,
    setPurgeText,
    addPreallocation,
    removePreallocation,
    applySnapshot,
  };

  if (!snapshot) {
    return (
      <>
        <Splash
          syncing={syncing}
          onSync={doSync}
          connLoaded={connLoaded}
          connInfo={connInfo}
          connBusy={connBusy}
          connError={connError}
          onConnect={doConnect}
        />
        {pendingUpdate ? (
          <UpdatePrompt
            pending={pendingUpdate}
            installing={installingUpdate}
            onInstall={() => void doInstallUpdate()}
            onDismiss={doDismissUpdate}
          />
        ) : null}
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
        <Sidebar
          screen={screen}
          setScreen={setScreen}
          hasRun={snapshot.run !== null}
          connInfo={connInfo}
          onChangeDb={() => {
            setConnError(null);
            setChangingConn(true);
          }}
          version={version}
          checkingUpdate={checkingUpdate}
          onCheckUpdate={() => void doCheckUpdate()}
        />
        <main style={{ flex: 1, minWidth: 0, overflow: "auto" }}>
          {detail ? (
            <DetailPage ui={ui} handlers={handlers} onBack={() => setDetail(null)} />
          ) : screen === "alloc" ? (
            <Allocations ui={ui} handlers={handlers} />
          ) : screen === "prealloc" ? (
            <PreallocationsWrapper ui={ui} handlers={handlers} />
          ) : (
            <Review ui={ui} handlers={handlers} />
          )}
        </main>
        {match ? <EventSidebar ui={ui} handlers={handlers} onClose={() => setMatch(null)} /> : null}
      </div>
      {changingConn ? (
        <div
          style={{
            position: "fixed",
            inset: 0,
            zIndex: 60,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "rgba(0,0,0,0.4)",
          }}
        >
          <div
            style={{
              width: 400,
              padding: 20,
              borderRadius: 12,
              background: "var(--token-color-surface-primary)",
              boxShadow: "var(--token-elevation-high-box-shadow)",
            }}
          >
            <div style={{ fontSize: 15, fontWeight: 700, color: "var(--token-color-foreground-strong)", marginBottom: 4 }}>
              Switch database
            </div>
            <div style={{ fontSize: 12, color: "var(--token-color-foreground-faint)", marginBottom: 14 }}>
              Currently {connInfo ?? "not connected"}. Connecting drops the loaded corpus and syncs the new project.
            </div>
            <ConnectForm
              busy={connBusy}
              error={connError}
              onConnect={doConnect}
              onCancel={() => {
                setChangingConn(false);
                setConnError(null);
              }}
            />
          </div>
        </div>
      ) : null}
      {pendingUpdate ? (
        <UpdatePrompt
          pending={pendingUpdate}
          installing={installingUpdate}
          onInstall={() => void doInstallUpdate()}
          onDismiss={doDismissUpdate}
        />
      ) : null}
      <Toasts toasts={toasts} onDismiss={dismissToast} />
    </div>
  );
}

// ---- Connection form -------------------------------------------------
function ConnectForm({
  busy,
  error,
  onConnect,
  onCancel,
}: {
  busy: boolean;
  error: string | null;
  onConnect: (projectRef: string, password: string, region: string) => void;
  onCancel?: () => void;
}) {
  // Project ref and region persist across launches; the password never does.
  const [projectRef, setProjectRef] = useState(() => localStorage.getItem("cupid.projectRef") ?? "");
  const [password, setPassword] = useState("");
  const [region, setRegion] = useState(() => localStorage.getItem("cupid.region") ?? "");
  const ready = projectRef.trim().length > 0 && password.length > 0 && !busy;
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10, width: "100%", textAlign: "left" }}>
      <TextInput
        label="Supabase project ref"
        placeholder="e.g. abcdefghijklmnopqrst"
        value={projectRef}
        onChange={(e) => setProjectRef(e.target.value)}
      />
      <TextInput
        label="Database password"
        placeholder="Postgres password for the project"
        type="password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
      />
      <TextInput
        label="Region (needed on networks without IPv6)"
        placeholder="e.g. ap-southeast-1 — uses the session pooler"
        value={region}
        onChange={(e) => setRegion(e.target.value)}
      />
      {error ? (
        <div
          style={{
            padding: "8px 11px",
            borderRadius: 7,
            fontSize: 12,
            background: "rgba(220,38,38,0.10)",
            border: "1px solid rgba(220,38,38,0.35)",
            color: "var(--token-color-foreground-critical-on-surface)",
            overflowWrap: "anywhere",
          }}
        >
          {error}
        </div>
      ) : null}
      <div style={{ display: "flex", gap: 8, justifyContent: "flex-end", marginTop: 2 }}>
        {onCancel ? (
          <Button color="ghost" onClick={onCancel}>
            Cancel
          </Button>
        ) : null}
        <Button
          color="primary"
          icon="download"
          busy={busy}
          disabled={!ready}
          onClick={() => onConnect(projectRef, password, region)}
        >
          {busy ? "Connecting…" : "Connect"}
        </Button>
      </div>
    </div>
  );
}

// ---- Splash / empty state --------------------------------------------
function Splash({
  syncing,
  onSync,
  connLoaded,
  connInfo,
  connBusy,
  connError,
  onConnect,
}: {
  syncing: boolean;
  onSync: () => void;
  connLoaded: boolean;
  connInfo: string | null;
  connBusy: boolean;
  connError: string | null;
  onConnect: (projectRef: string, password: string, region: string) => void;
}) {
  const [showForm, setShowForm] = useState(false);
  const needsForm = connLoaded && (!connInfo || showForm);
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
      {!connLoaded ? null : syncing ? (
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
      ) : needsForm ? (
        <>
          <div style={{ fontSize: 15, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>
            Connect to your database
          </div>
          <div style={{ fontSize: 13, maxWidth: 340, textAlign: "center" }}>
            Enter the Supabase project ref and database password. Direct connections resolve over IPv6 only, so
            on any other network add the project's region to route through the session pooler.
          </div>
          <div style={{ width: 340 }}>
            <ConnectForm
              busy={connBusy}
              error={connError}
              onConnect={onConnect}
              onCancel={connInfo ? () => setShowForm(false) : undefined}
            />
          </div>
        </>
      ) : (
        <>
          <div style={{ fontSize: 15, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>
            Sync to load the corpus
          </div>
          <div style={{ fontSize: 13, maxWidth: 340, textAlign: "center" }}>
            Connected to <strong>{connInfo}</strong>. Sync to pull applicants, positions, and appointments.
          </div>
          <Button color="primary" icon="download" onClick={onSync}>
            Sync
          </Button>
          <button
            onClick={() => setShowForm(true)}
            style={{
              border: "none",
              background: "transparent",
              cursor: "pointer",
              font: "inherit",
              fontSize: 12,
              color: "var(--token-color-foreground-faint)",
              textDecoration: "underline",
            }}
          >
            Use a different database
          </button>
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
  connInfo,
  onChangeDb,
  version,
  checkingUpdate,
  onCheckUpdate,
}: {
  screen: Screen;
  setScreen: (s: Screen) => void;
  hasRun: boolean;
  connInfo: string | null;
  onChangeDb: () => void;
  version: string | null;
  checkingUpdate: boolean;
  onCheckUpdate: () => void;
}) {
  const items: { id: Screen; label: string; icon: string }[] = [
    { id: "alloc", label: "Allocations", icon: "layers" },
    { id: "prealloc", label: "Preallocations", icon: "tag" },
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
          <div
            title={connInfo ?? undefined}
            style={{
              color: "var(--token-color-foreground-faint)",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
          >
            {connInfo ?? "Not connected"}
          </div>
          <button
            onClick={onChangeDb}
            style={{
              border: "none",
              background: "transparent",
              cursor: "pointer",
              font: "inherit",
              fontSize: 11.5,
              fontWeight: 600,
              color: "#DB2A63",
              padding: 0,
              textAlign: "left",
            }}
          >
            Switch database…
          </button>
        </div>
      </Card>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: 8,
          padding: "10px 4px 2px",
          fontSize: 11,
          color: "var(--token-color-foreground-faint)",
        }}
      >
        <span>v{version ?? "—"}</span>
        <button
          onClick={onCheckUpdate}
          disabled={checkingUpdate}
          style={{
            border: "none",
            background: "transparent",
            cursor: checkingUpdate ? "default" : "pointer",
            font: "inherit",
            fontSize: 11,
            fontWeight: 600,
            color: checkingUpdate ? "var(--token-color-foreground-faint)" : "#DB2A63",
            padding: 0,
          }}
        >
          {checkingUpdate ? "Checking…" : "Check for updates"}
        </button>
      </div>
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
function PreallocationsWrapper({ ui, handlers }: { ui: UiState; handlers: UiHandlers }) {
  if (!ui.snapshot || !ui.idx) return null;
  return (
    <PreallocationsScreen
      snapshot={ui.snapshot}
      idx={ui.idx}
      onAdd={handlers.addPreallocation}
      onRemove={handlers.removePreallocation}
      onOpenMatch={handlers.openMatch}
      toast={handlers.toast}
    />
  );
}

function Allocations({ ui, handlers }: { ui: UiState; handlers: UiHandlers }) {
  if (!ui.snapshot || !ui.idx) return null;
  return (
    <AllocationsScreen
      snapshot={ui.snapshot}
      idx={ui.idx}
      view={ui.view}
      search={ui.search}
      typeFilter={ui.typeFilter}
      page={ui.page}
      onSetView={handlers.setView}
      onSetSearch={handlers.setSearch}
      onSetTypeFilter={handlers.setTypeFilter}
      onSetPage={handlers.setPage}
      onOpenDetail={handlers.openDetail}
      onOpenMatch={handlers.openMatch}
      hasRun={ui.snapshot.run !== null}
      running={ui.running}
      onRun={handlers.doRun}
    />
  );
}

function Review({ ui, handlers }: { ui: UiState; handlers: UiHandlers }) {
  if (!ui.snapshot || !ui.idx) return null;
  return (
    <ReviewScreen
      snapshot={ui.snapshot}
      idx={ui.idx}
      commitState={ui.commitState}
      purgeText={ui.purgeText}
      onCommitState={handlers.setCommitState}
      onPurgeText={handlers.setPurgeText}
      onOpenMatch={handlers.openMatch}
      toast={handlers.toast}
      running={ui.running}
      onRun={handlers.doRun}
      onApplySnapshot={handlers.applySnapshot}
    />
  );
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

function EventSidebar({ ui, handlers, onClose }: { ui: UiState; handlers: UiHandlers; onClose: () => void }) {
  if (!ui.snapshot || !ui.idx || !ui.match) return null;
  return (
    <EventSidebarScreen
      match={ui.match}
      snapshot={ui.snapshot}
      idx={ui.idx}
      onClose={onClose}
      onOpenDetail={handlers.openDetail}
    />
  );
}

export default App;
