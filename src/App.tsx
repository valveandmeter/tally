import { useEffect, useRef, useState } from "react";
import "./App.css";
import {
  createTimer,
  discardTimer,
  formatElapsed,
  listTimers,
  onTimersState,
  pauseTimer,
  renameTimer,
  startTimer,
  wrikeStatus,
  type ClientResult,
  type TaskResult,
  type Timer,
} from "./lib/api";
import { TimerRow } from "./components/TimerRow";
import { ConnectCard } from "./components/ConnectCard";
import { SearchPanel } from "./components/SearchPanel";

const AUTO_PAUSE_BANNER_MS = 4000;

type View = "timers" | "search";

function App() {
  const [timers, setTimers] = useState<Timer[]>([]);
  const [banner, setBanner] = useState<string | null>(null);
  const [justCreatedId, setJustCreatedId] = useState<string | null>(null);
  const [view, setView] = useState<View>("timers");
  const [wrikeConnected, setWrikeConnected] = useState<boolean | null>(null);
  const bannerTimeout = useRef<number | undefined>(undefined);

  useEffect(() => {
    listTimers().then(setTimers);
    wrikeStatus().then((account) => setWrikeConnected(account !== null));
    const unlistenPromise = onTimersState((state) => setTimers(state.timers));
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
      window.clearTimeout(bannerTimeout.current);
    };
  }, []);

  const showAutoPauseBanner = (label: string, elapsedSeconds: number) => {
    setBanner(`Paused ${label} at ${formatElapsed(elapsedSeconds)}`);
    window.clearTimeout(bannerTimeout.current);
    bannerTimeout.current = window.setTimeout(() => setBanner(null), AUTO_PAUSE_BANNER_MS);
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
            autoFocusLabel={timer.id === justCreatedId}
          />
        ))}
      </ul>

      <button className="popover__new-timer" onClick={() => setView("search")}>
        + New timer
      </button>
    </div>
  );
}

export default App;
