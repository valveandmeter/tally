import { useEffect, useRef, useState } from "react";
import "./App.css";
import {
  confirmCatchAllTask,
  createTimer,
  discardTimer,
  formatElapsed,
  getWrikeTotals,
  listTimers,
  logTimer,
  onQueueChanged,
  onTimersState,
  pauseTimer,
  renameTimer,
  startTimer,
  wrikeStatus,
  type ClientResult,
  type TaskResult,
  type Timer,
  type Totals,
} from "./lib/api";
import { TimerRow } from "./components/TimerRow";
import { ConnectCard } from "./components/ConnectCard";
import { SearchPanel } from "./components/SearchPanel";
import { Footer } from "./components/Footer";
import { CatchAllConfirm } from "./components/CatchAllConfirm";

const AUTO_PAUSE_BANNER_MS = 4000;
const LOG_CONFIRM_BANNER_MS = 4000;

function formatLoggedMinutes(totalMinutes: number): string {
  const hours = Math.floor(totalMinutes / 60);
  const mins = totalMinutes % 60;
  return `${hours}:${mins.toString().padStart(2, "0")}`;
}

interface CatchAllConfirmState {
  timerId: string;
  clientId: string;
  clientName: string;
  suggestedTitle: string;
}

type View = "timers" | "search";

function App() {
  const [timers, setTimers] = useState<Timer[]>([]);
  const [banner, setBanner] = useState<string | null>(null);
  const [justCreatedId, setJustCreatedId] = useState<string | null>(null);
  const [view, setView] = useState<View>("timers");
  const [wrikeConnected, setWrikeConnected] = useState<boolean | null>(null);
  const [totals, setTotals] = useState<Totals | null>(null);
  const [catchAllConfirm, setCatchAllConfirm] = useState<CatchAllConfirmState | null>(null);
  const bannerTimeout = useRef<number | undefined>(undefined);

  const refreshTotals = () => {
    getWrikeTotals()
      .then(setTotals)
      .catch(() => setTotals(null));
  };

  useEffect(() => {
    listTimers().then(setTimers);
    wrikeStatus().then((account) => {
      const connected = account !== null;
      setWrikeConnected(connected);
      if (connected) refreshTotals();
    });
    const unlistenTimersPromise = onTimersState((state) => setTimers(state.timers));
    const unlistenQueuePromise = onQueueChanged(() => refreshTotals());
    return () => {
      unlistenTimersPromise.then((unlisten) => unlisten());
      unlistenQueuePromise.then((unlisten) => unlisten());
      window.clearTimeout(bannerTimeout.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const showBanner = (text: string, ms: number) => {
    setBanner(text);
    window.clearTimeout(bannerTimeout.current);
    bannerTimeout.current = window.setTimeout(() => setBanner(null), ms);
  };

  const showAutoPauseBanner = (label: string, elapsedSeconds: number) => {
    showBanner(`Paused ${label} at ${formatElapsed(elapsedSeconds)}`, AUTO_PAUSE_BANNER_MS);
  };

  const handleToggle = async (timer: Timer) => {
    if (timer.status === "running") {
      await pauseTimer(timer.id);
      return;
    }
    const autoPaused = await startTimer(timer.id);
    if (autoPaused) {
      showAutoPauseBanner(autoPaused.label, autoPaused.elapsedSeconds);
    }
  };

  const handleRename = (id: string, label: string) => {
    renameTimer(id, label);
    setJustCreatedId(null);
  };

  const handleDiscard = (id: string) => {
    discardTimer(id);
  };

  const handleLog = async (timer: Timer) => {
    const outcome = await logTimer(timer.id);
    if (outcome.kind === "logged") {
      showBanner(`Logged ${formatLoggedMinutes(outcome.minutes)} to Wrike · ${outcome.label}`, LOG_CONFIRM_BANNER_MS);
      refreshTotals();
    } else {
      setCatchAllConfirm({
        timerId: timer.id,
        clientId: outcome.clientId,
        clientName: outcome.clientName,
        suggestedTitle: outcome.suggestedTitle,
      });
    }
  };

  const handleConfirmCatchAll = async () => {
    if (!catchAllConfirm) return;
    const { timerId, clientId } = catchAllConfirm;
    setCatchAllConfirm(null);
    await confirmCatchAllTask(clientId);
    const outcome = await logTimer(timerId);
    if (outcome.kind === "logged") {
      showBanner(`Logged ${formatLoggedMinutes(outcome.minutes)} to Wrike · ${outcome.label}`, LOG_CONFIRM_BANNER_MS);
      refreshTotals();
    }
  };

  const handleSelectTask = async (task: TaskResult) => {
    const label = `${task.clientName}${task.projectPath ? ` › ${task.projectPath}` : ""} › ${task.title}`;
    const timer = await createTimer(label, task.id);
    setJustCreatedId(timer.id);
    setView("timers");
  };

  const handleSelectBucket = async (client: ClientResult) => {
    const timer = await createTimer(`${client.name} (no task)`, undefined, client.id);
    setJustCreatedId(timer.id);
    setView("timers");
  };

  const handleQuickTimer = async () => {
    const timer = await createTimer("New timer");
    setJustCreatedId(timer.id);
    setView("timers");
  };

  const runningCount = timers.filter((t) => t.status === "running").length;
  const pausedCount = timers.length - runningCount;

  if (view === "search") {
    if (wrikeConnected === false) {
      return (
        <div className="popover">
          <header className="popover__header">
            <h1>Timers</h1>
            <button className="popover__back" onClick={() => setView("timers")} aria-label="Back">
              ←
            </button>
          </header>
          <ConnectCard
            onConnected={() => {
              setWrikeConnected(true);
              refreshTotals();
            }}
          />
          <button className="popover__quick-timer" onClick={handleQuickTimer}>
            Skip for now — create a quick timer without Wrike
          </button>
        </div>
      );
    }
    return (
      <div className="popover">
        <SearchPanel
          onSelectTask={handleSelectTask}
          onSelectBucket={handleSelectBucket}
          onClose={() => setView("timers")}
        />
      </div>
    );
  }

  return (
    <div className="popover">
      <header className="popover__header">
        <h1>Timers</h1>
        <button className="popover__settings" aria-label="Settings" disabled>
          ⚙
        </button>
      </header>

      {banner && <div className="popover__banner">{banner}</div>}

      {timers.length > 0 && (
        <p className="popover__summary">
          {runningCount} running · {pausedCount} paused
        </p>
      )}

      <ul className="timer-list">
        {timers.map((timer) => (
          <TimerRow
            key={timer.id}
            timer={timer}
            onToggle={handleToggle}
            onRename={handleRename}
            onDiscard={handleDiscard}
            onLog={handleLog}
            autoFocusLabel={timer.id === justCreatedId}
          />
        ))}
      </ul>

      {catchAllConfirm && (
        <CatchAllConfirm
          clientName={catchAllConfirm.clientName}
          suggestedTitle={catchAllConfirm.suggestedTitle}
          onConfirm={handleConfirmCatchAll}
          onCancel={() => setCatchAllConfirm(null)}
        />
      )}

      <button className="popover__new-timer" onClick={() => setView("search")}>
        + New timer
      </button>

      <Footer totals={totals} />
    </div>
  );
}

export default App;
