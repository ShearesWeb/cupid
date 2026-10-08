// App.tsx — app shell: TopBar, Sidebar, routing state, toasts (task-12).
// Ports reference/cca-console-design.html lines 182-268 (app frame, sidebar, topbar)
// and replaces the mock splash loader with a real "sync to load" empty state.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as api from "./lib/api.ts";
import type { CommitmentPeriod, ConnectEvent, DirectorySnapshot, Snapshot } from "./lib/types.ts";
import { buildIndexes, type Indexes } from "./lib/indexes.ts";
import { buildDirectoryIndex, kindLabel } from "./lib/directory.ts";
import { errorMessage, fmtTime } from "./lib/format.ts";
import { Icon, Button } from "./components/index.ts";
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
import { initialAllocState, type AllocState } from "./lib/allocState.ts";
import { DetailPage as DetailPageScreen } from "./screens/DetailPage.tsx";
import { EventSidebar as EventSidebarScreen } from "./screens/EventSidebar.tsx";
import { Preallocations as PreallocationsScreen } from "./screens/Preallocations.tsx";
import { Review as ReviewScreen, type CommitState } from "./screens/Review.tsx";
import { Ccas as CcasScreen } from "./screens/Ccas.tsx";
import { TextInput } from "./components/TextInput.tsx";
import { ConnectLog, type ConnectLogState } from "./components/ConnectLog.tsx";

type Screen = "alloc" | "ccas" | "prealloc" | "review";
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
  directory: DirectorySnapshot | null;
  idx: Indexes | null;
  screen: Screen;
  alloc: AllocState;
  ccaOpen: number | null;
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
  patchAlloc: (patch: Partial<AllocState>) => void;
  setCcaOpen: (id: number | null) => void;
  jumpToCca: (id: number) => void;
  toast: (kind: ToastKind, text: string) => void;
  setCommitState: (s: CommitState | ((prev: CommitState) => CommitState)) => void;
  setPurgeText: (v: string) => void;
  addPreallocation: (applicantId: number, positionId: number, note: string | null) => Promise<boolean>;
  removePreallocation: (applicantId: number, positionId: number) => Promise<boolean>;
  addAppointment: (userId: number, positionId: number, period: CommitmentPeriod) => Promise<boolean>;
  removeAppointment: (userId: number, positionId: number) => Promise<boolean>;
  updateAppointmentPeriod: (userId: number, positionId: number, period: CommitmentPeriod) => Promise<boolean>;
  applySnapshot: (snap: Snapshot) => void;
}

let toastSeq = 0;

function App() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [directory, setDirectory] = useState<DirectorySnapshot | null>(null);
  const [theme, setTheme] = useState<Theme>("light");
  const [screen, setScreenState] = useState<Screen>("ccas");
  const [alloc, setAlloc] = useState<AllocState>(initialAllocState);
  const [ccaOpen, setCcaOpen] = useState<number | null>(null);
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
  // Step trace of the connect in progress, kept after a failure so the
  // operator can see which layer of the network path broke.
  const [connLog, setConnLog] = useState<ConnectLogState | null>(null);

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

  // Resolves to the failure message, or null once the corpus is loaded.
  const doSync = async (): Promise<string | null> => {
    if (syncing) return null;
    setSyncing(true);
    try {
      const snap = await api.sync();
      const directorySnapshot = await api.directorySnapshot();
      setSnapshot(snap);
      setDirectory(directorySnapshot);
      setCommitState(initialCommitState);
      setPurgeText("");
      setDetail(null);
      setMatch(null);
      snap.warnings.forEach((w) => toast("error", w));
      return null;
    } catch (e) {
      const message = errorMessage(e);
      toast("error", message);
      return message;
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

  // Appointment edits return the refreshed directory; the CCA screen words
  // its own success toasts, so only failures are reported here.
  const editDirectory = async (edit: () => Promise<DirectorySnapshot>) => {
    try {
      setDirectory(await edit());
      return true;
    } catch (e) {
      toast("error", errorMessage(e));
      return false;
    }
  };
  const addAppointment = (userId: number, positionId: number, period: CommitmentPeriod) =>
    editDirectory(() => api.addAppointment(userId, positionId, period));
  const removeAppointment = (userId: number, positionId: number) =>
    editDirectory(() => api.removeAppointment(userId, positionId));
  const updateAppointmentPeriod = (userId: number, positionId: number, period: CommitmentPeriod) =>
    editDirectory(() => api.updateAppointmentPeriod(userId, positionId, period));

  // Verify credentials, adopt the new target, and pull its corpus. The old
  // snapshot dies with the old database; a connect failure leaves everything
  // untouched and surfaces inline on the form (toasts vanish too fast for
  // credential errors).
  const doConnect = async (projectRef: string, password: string, region: string) => {
    if (connBusy) return;
    setConnBusy(true);
    setConnError(null);
    setConnLog({ events: [], load: null });
    const onEvent = (event: ConnectEvent) =>
      setConnLog((log) => (log ? { ...log, events: [...log.events, event] } : log));
    try {
      const label = await api.connect(projectRef, password, region.trim() ? region.trim() : null, onEvent);
      localStorage.setItem("cupid.projectRef", projectRef.trim());
      localStorage.setItem("cupid.region", region.trim());
      setConnInfo(label);
      setChangingConn(false);
      setSnapshot(null);
      setDirectory(null);
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
    setConnLog((log) => log && { ...log, load: { label: "Load data", status: "running", detail: null } });
    const failure = await doSync();
    setConnLog((log) =>
      failure === null ? null : log && { ...log, load: { label: "Load data", status: "failed", detail: failure } },
    );
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
    setCcaOpen(null);
    setDetail(null);
    setMatch(null);
  };
  const patchAlloc = (patch: Partial<AllocState>) => {
    setAlloc((prev) => ({ ...prev, ...patch }));
    if (patch.view) setMatch(null);
  };
  // Global search: a CCA opens its directory page; a person opens their
  // applicant detail over the applicant view, filtered to them.
  const jumpToCca = (id: number) => {
    setScreen("ccas");
    setCcaOpen(id);
  };
  const jumpToApplicant = (id: number, name: string) => {
    setScreen("alloc");
    setAlloc((prev) => ({ ...prev, view: "applicant", search: name, appFilter: "all", page: 0 }));
    setDetail({ type: "applicant", id });
  };

  const ui: UiState = {
    snapshot,
    directory,
    idx,
    screen,
    alloc,
    ccaOpen,
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
    patchAlloc,
    setCcaOpen,
    jumpToCca,
    toast,
    setCommitState,
    setPurgeText,
    addPreallocation,
    removePreallocation,
    addAppointment,
    removeAppointment,
    updateAppointmentPeriod,
    applySnapshot,
  };

  if (!snapshot) {
    return (
      <>
        <Splash
          syncing={syncing}
          onSync={() => {
            setConnLog(null);
            void doSync();
          }}
          connLoaded={connLoaded}
          connInfo={connInfo}
          connBusy={connBusy}
          connError={connError}
          connLog={connLog}
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
        directory={directory}
        onJumpToCca={jumpToCca}
        onJumpToApplicant={jumpToApplicant}
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
          ccaCount={directory?.ccas.length ?? null}
          preallocationCount={snapshot.preallocations.length}
          hasRun={snapshot.run !== null}
          connInfo={connInfo}
          onChangeDb={() => {
            setConnError(null);
            setConnLog(null);
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
          ) : screen === "ccas" ? (
            <Ccas ui={ui} handlers={handlers} />
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
              log={connLog}
              onConnect={doConnect}
              onCancel={() => {
                setChangingConn(false);
                setConnError(null);
                setConnLog(null);
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
  log,
  onConnect,
  onCancel,
}: {
  busy: boolean;
  error: string | null;
  log: ConnectLogState | null;
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
        placeholder="e.g. ap-southeast-1: routes through the pooler"
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
      {log ? <ConnectLog log={log} /> : null}
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
  connLog,
  onConnect,
}: {
  syncing: boolean;
  onSync: () => void;
  connLoaded: boolean;
  connInfo: string | null;
  connBusy: boolean;
  connError: string | null;
  connLog: ConnectLogState | null;
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
      {!connLoaded ? null : syncing && connLog ? (
        <>
          <div style={{ fontSize: 15, fontWeight: 700, color: "var(--token-color-foreground-strong)" }}>
            Loading data&hellip;
          </div>
          <div style={{ width: 340 }}>
            <ConnectLog log={connLog} />
          </div>
        </>
      ) : syncing ? (
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
            on any other network add the project's region to route through the pooler.
          </div>
          <div style={{ width: 340 }}>
            <ConnectForm
              busy={connBusy}
              error={connError}
              log={connLog}
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
          {connLog ? (
            <div style={{ width: 340 }}>
              <ConnectLog log={connLog} />
            </div>
          ) : null}
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
  directory,
  onJumpToCca,
  onJumpToApplicant,
  syncing,
  running,
  theme,
  setTheme,
  doSync,
  doRun,
}: {
  snapshot: Snapshot;
  directory: DirectorySnapshot | null;
  onJumpToCca: (id: number) => void;
  onJumpToApplicant: (id: number, name: string) => void;
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
      <div style={{ flex: 1, display: "flex", justifyContent: "center" }}>
        <GlobalSearch snapshot={snapshot} directory={directory} onJumpToCca={onJumpToCca} onJumpToApplicant={onJumpToApplicant} />
      </div>
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

type SearchResult = { kind: "cca" | "person"; id: number; label: string; sub: string };

// Jump-to box (design globalSearch): up to five CCAs from the directory and
// five applicants from the allocation corpus. "/" or Ctrl/Cmd+K focuses it.
function GlobalSearch({
  snapshot,
  directory,
  onJumpToCca,
  onJumpToApplicant,
}: {
  snapshot: Snapshot;
  directory: DirectorySnapshot | null;
  onJumpToCca: (id: number) => void;
  onJumpToApplicant: (id: number, name: string) => void;
}) {
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  const dx = useMemo(() => (directory ? buildDirectoryIndex(directory) : null), [directory]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement | null)?.tagName;
      const typing = tag === "INPUT" || tag === "TEXTAREA";
      if ((e.key === "/" && !typing) || ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k")) {
        e.preventDefault();
        input.current?.focus();
        input.current?.select();
      }
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, []);

  const q = query.trim().toLowerCase();
  const ccaResults: SearchResult[] =
    q && directory && dx
      ? directory.ccas
          .filter((c) => c.name.toLowerCase().includes(q))
          .slice(0, 5)
          .map((c) => {
            const n = dx.memberCount(c.id);
            return { kind: "cca", id: c.id, label: c.name, sub: `${kindLabel(c.kind)} · ${n} member${n === 1 ? "" : "s"}` };
          })
      : [];
  const personResults: SearchResult[] = q
    ? snapshot.applicants
        .filter((a) => a.name.toLowerCase().includes(q) || a.email.toLowerCase().includes(q))
        .slice(0, 5)
        .map((a) => ({ kind: "person", id: a.id, label: a.name, sub: a.email }))
    : [];
  const results = [...ccaResults, ...personResults];

  const go = (r: SearchResult) => {
    setQuery("");
    setOpen(false);
    input.current?.blur();
    if (r.kind === "cca") onJumpToCca(r.id);
    else onJumpToApplicant(r.id, r.label);
  };

  return (
    <div style={{ position: "relative", width: "100%", maxWidth: 380 }}>
      <span style={{ position: "absolute", left: 11, top: 10, display: "flex", pointerEvents: "none" }}>
        <Icon name="search" size={14} color="var(--token-color-foreground-faint)" />
      </span>
      <input
        ref={input}
        value={query}
        placeholder="Jump to a CCA or person…  ( / )"
        aria-label="Search CCAs and people"
        onChange={(e) => {
          setQuery(e.target.value);
          setOpen(true);
        }}
        onFocus={() => setOpen(true)}
        onBlur={() => setOpen(false)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && results[0]) go(results[0]);
          else if (e.key === "Escape") {
            setQuery("");
            setOpen(false);
          }
        }}
        style={{
          width: "100%",
          height: 34,
          padding: "0 12px 0 33px",
          borderRadius: 20,
          border: "1px solid var(--token-color-border-primary)",
          background: "var(--token-color-page-faint)",
          color: "var(--token-color-foreground-primary)",
          font: "inherit",
          fontSize: 13,
          outline: "none",
        }}
      />
      {open && q ? (
        <div
          style={{
            position: "absolute",
            top: 40,
            left: 0,
            right: 0,
            zIndex: 60,
            borderRadius: 10,
            overflow: "hidden",
            background: "var(--token-color-surface-primary)",
            boxShadow: "var(--token-elevation-high-box-shadow)",
          }}
        >
          {results.length ? (
            results.map((r) => (
              <button
                key={r.kind + r.id}
                onMouseDown={(e) => {
                  e.preventDefault();
                  go(r);
                }}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 9,
                  width: "100%",
                  padding: "8px 12px",
                  border: "none",
                  background: "transparent",
                  cursor: "pointer",
                  font: "inherit",
                  textAlign: "left",
                }}
              >
                <Icon name={r.kind === "person" ? "user" : "grid"} size={14} color="var(--cupid)" />
                <span style={{ flex: 1, minWidth: 0 }}>
                  <span
                    style={{
                      display: "block",
                      fontSize: 13,
                      fontWeight: 600,
                      color: "var(--token-color-foreground-strong)",
                      whiteSpace: "nowrap",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                    }}
                  >
                    {r.label}
                  </span>
                  <span style={{ display: "block", fontSize: 11, color: "var(--token-color-foreground-faint)" }}>{r.sub}</span>
                </span>
                <span style={{ fontSize: 10, fontWeight: 700, letterSpacing: "0.5px", textTransform: "uppercase", color: "var(--cupid-strong)" }}>
                  {r.kind === "person" ? "Person" : "CCA"}
                </span>
              </button>
            ))
          ) : (
            <div style={{ padding: 12, fontSize: 12.5, color: "var(--token-color-foreground-faint)" }}>No matching CCAs or people</div>
          )}
        </div>
      ) : null}
    </div>
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
type NavItem = { id: Screen; label: string; icon: string; count?: number | null; dot?: string | null };

function Sidebar({
  screen,
  setScreen,
  ccaCount,
  preallocationCount,
  hasRun,
  connInfo,
  onChangeDb,
  version,
  checkingUpdate,
  onCheckUpdate,
}: {
  screen: Screen;
  setScreen: (s: Screen) => void;
  ccaCount: number | null;
  preallocationCount: number;
  hasRun: boolean;
  connInfo: string | null;
  onChangeDb: () => void;
  version: string | null;
  checkingUpdate: boolean;
  onCheckUpdate: () => void;
}) {
  const runColor = hasRun ? "var(--token-color-foreground-success)" : "var(--token-color-foreground-faint)";
  const groups: { label: string; items: NavItem[] }[] = [
    { label: "Home", items: [{ id: "ccas", label: "CCAs", icon: "grid", count: ccaCount }] },
    {
      label: "Allocation",
      items: [
        { id: "prealloc", label: "Preallocations", icon: "tag", count: preallocationCount },
        { id: "alloc", label: "Allocations", icon: "layers" },
      ],
    },
    { label: "Commit", items: [{ id: "review", label: "Review & commit", icon: "lock", dot: hasRun ? runColor : null }] },
  ];
  const linkStyle = { border: "none", background: "transparent", font: "inherit", fontWeight: 600, padding: 0, textAlign: "left" } as const;
  return (
    <nav
      style={{
        width: 220,
        flex: "0 0 220px",
        display: "flex",
        flexDirection: "column",
        background: "var(--token-color-surface-primary)",
        borderRight: "1px solid var(--token-color-border-faint)",
        overflow: "auto",
      }}
    >
      <div style={{ display: "flex", flexDirection: "column", gap: 18, padding: "16px 10px" }}>
        {groups.map((g) => (
          <div key={g.label} style={{ display: "flex", flexDirection: "column", gap: 2 }}>
            <div
              style={{
                fontSize: 10.5,
                fontWeight: 700,
                letterSpacing: "0.6px",
                textTransform: "uppercase",
                color: "var(--token-color-foreground-faint)",
                padding: "0 10px 6px",
              }}
            >
              {g.label}
            </div>
            {g.items.map((it) => {
              const active = screen === it.id;
              return (
                <button
                  key={it.id}
                  onClick={() => setScreen(it.id)}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 10,
                    height: 36,
                    padding: "0 10px",
                    borderRadius: 7,
                    font: "inherit",
                    fontSize: 13,
                    fontWeight: active ? 700 : 500,
                    cursor: "pointer",
                    textAlign: "left",
                    background: active ? "var(--cupid-soft)" : "transparent",
                    color: active ? "var(--cupid-strong)" : "var(--token-color-foreground-primary)",
                    border: "none",
                  }}
                >
                  <Icon name={it.icon} size={16} color={active ? "var(--cupid-strong)" : "var(--token-color-foreground-faint)"} />
                  <span style={{ flex: 1, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>{it.label}</span>
                  {it.count != null ? (
                    <span
                      style={{
                        minWidth: 20,
                        height: 18,
                        padding: "0 6px",
                        borderRadius: 999,
                        display: "inline-flex",
                        alignItems: "center",
                        justifyContent: "center",
                        fontSize: 11,
                        fontWeight: 700,
                        background: active ? "var(--token-color-surface-primary)" : "var(--token-color-surface-strong)",
                        color: active ? "var(--cupid-strong)" : "var(--token-color-foreground-faint)",
                      }}
                    >
                      {it.count}
                    </span>
                  ) : null}
                  {it.dot ? <span title="Run ready to review" style={{ width: 7, height: 7, borderRadius: "50%", background: it.dot }} /> : null}
                </button>
              );
            })}
          </div>
        ))}
      </div>
      <div style={{ flex: 1 }} />
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 6,
          padding: "12px 20px 14px",
          borderTop: "1px solid var(--token-color-border-faint)",
          fontSize: 12,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 7 }}>
          <span style={{ width: 7, height: 7, borderRadius: "50%", background: runColor }} />
          <span style={{ fontWeight: 600, color: "var(--token-color-foreground-strong)" }}>{hasRun ? "Matching run ready" : "No matching run"}</span>
        </div>
        <div title={connInfo ?? undefined} style={{ display: "flex", alignItems: "center", gap: 7, color: "var(--token-color-foreground-faint)" }}>
          <Icon name="server" size={13} color="var(--token-color-foreground-faint)" />
          <span style={{ whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>{connInfo ?? "Not connected"}</span>
        </div>
        <button onClick={onChangeDb} style={{ ...linkStyle, cursor: "pointer", fontSize: 11.5, color: "var(--cupid)" }}>
          Switch database…
        </button>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            gap: 8,
            paddingTop: 4,
            fontSize: 11,
            color: "var(--token-color-foreground-faint)",
          }}
        >
          <span>v{version ?? "—"}</span>
          <button
            onClick={onCheckUpdate}
            disabled={checkingUpdate}
            style={{
              ...linkStyle,
              cursor: checkingUpdate ? "default" : "pointer",
              fontSize: 11,
              color: checkingUpdate ? "var(--token-color-foreground-faint)" : "var(--cupid)",
            }}
          >
            {checkingUpdate ? "Checking…" : "Check for updates"}
          </button>
        </div>
      </div>
    </nav>
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
      state={ui.alloc}
      onPatch={handlers.patchAlloc}
      onOpenDetail={handlers.openDetail}
      onOpenMatch={handlers.openMatch}
      hasRun={ui.snapshot.run !== null}
    />
  );
}

function Ccas({ ui, handlers }: { ui: UiState; handlers: UiHandlers }) {
  if (!ui.directory) return null;
  return (
    <CcasScreen
      directory={ui.directory}
      openCcaId={ui.ccaOpen}
      onOpenCca={handlers.setCcaOpen}
      onAdd={handlers.addAppointment}
      onRemove={handlers.removeAppointment}
      onUpdatePeriod={handlers.updateAppointmentPeriod}
      toast={handlers.toast}
    />
  );
}

function Review({ ui, handlers }: { ui: UiState; handlers: UiHandlers }) {
  if (!ui.snapshot || !ui.idx) return null;
  return (
    <ReviewScreen
      snapshot={ui.snapshot}
      directory={ui.directory ?? { users: [], ccas: [], positions: [], appointments: [], changes: [] }}
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
      directory={ui.directory}
      idx={ui.idx}
      screen={ui.screen}
      onBack={onBack}
      onOpenMatch={handlers.openMatch}
      onOpenDetail={handlers.openDetail}
      onOpenCca={handlers.jumpToCca}
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
