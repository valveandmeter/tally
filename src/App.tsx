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
  type Timer,
} from "./lib/api";
import { TimerRow } from "./components/TimerRow";

const AUTO_PAUSE_BANNER_MS = 4000;

function App() {
  const [timers, setTimers] = useState<Timer[]>([]);
  const [banner, setBanner] = useState<string | null>(null);
  const [justCreatedId, setJustCreatedId] = useState<string | null>(null);
  const bannerTimeout = useRef<number | undefined>(undefined);

  useEffect(() => {
    listTimers().then(setTimers);
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

  const handleNewTimer = async () => {
    const timer = await createTimer("New timer");
    setJustCreatedId(timer.id);
  };

  const runningCount = timers.filter((t) => t.status === "running").length;
  const pausedCount = timers.length - runningCount;

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

      <button className="popover__new-timer" onClick={handleNewTimer}>
        + New timer
      </button>
    </div>
  );
}

export default App;
