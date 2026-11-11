import { useEffect, useRef, useState } from "react";
import type { Timer } from "../lib/api";
import { formatElapsed } from "../lib/api";

interface Props {
  timer: Timer;
  onToggle: (timer: Timer) => void;
  onRename: (id: string, label: string) => void;
  onDiscard: (id: string) => void;
  onLog: (timer: Timer) => void;
  autoFocusLabel?: boolean;
}

const DISCARD_CONFIRM_THRESHOLD_SECONDS = 60;

export function TimerRow({ timer, onToggle, onRename, onDiscard, onLog, autoFocusLabel }: Props) {
  const [editing, setEditing] = useState(false);
  const [draftLabel, setDraftLabel] = useState(timer.label);
  const [confirmingDiscard, setConfirmingDiscard] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (autoFocusLabel) {
      setEditing(true);
    }
  }, [autoFocusLabel]);

  useEffect(() => {
    if (editing) {
      inputRef.current?.focus();
      inputRef.current?.select();
    }
  }, [editing]);

  const commitLabel = () => {
    setEditing(false);
    const trimmed = draftLabel.trim();
    if (trimmed && trimmed !== timer.label) {
      onRename(timer.id, trimmed);
    } else {
      setDraftLabel(timer.label);
    }
  };

  const handleDiscardClick = () => {
    if (timer.elapsedSeconds >= DISCARD_CONFIRM_THRESHOLD_SECONDS && !confirmingDiscard) {
      setConfirmingDiscard(true);
      return;
    }
    onDiscard(timer.id);
  };

  const isRunning = timer.status === "running";
  const isLoggable = Boolean(timer.wrikeTaskId || timer.wrikeBucketClientId);

  return (
    <li className={`timer-row${isRunning ? " timer-row--running" : " timer-row--paused"}`}>
      <div className="timer-row__main">
        {editing ? (
          <input
            ref={inputRef}
            className="timer-row__label-input"
            value={draftLabel}
            onChange={(e) => setDraftLabel(e.currentTarget.value)}
            onBlur={commitLabel}
            onKeyDown={(e) => {
              if (e.key === "Enter") commitLabel();
              if (e.key === "Escape") {
                setDraftLabel(timer.label);
                setEditing(false);
              }
            }}
          />
        ) : (
          <button className="timer-row__label" onClick={() => setEditing(true)}>
            {timer.label}
          </button>
        )}
        <span className="timer-row__elapsed">{formatElapsed(timer.elapsedSeconds)}</span>
      </div>
      <div className="timer-row__actions">
        <button className="timer-row__action" onClick={() => onToggle(timer)}>
          {isRunning ? "Pause" : "Resume"}
        </button>
        {isLoggable && (
          <button className="timer-row__action timer-row__action--primary" onClick={() => onLog(timer)}>
            Log to Wrike
          </button>
        )}
        {confirmingDiscard ? (
          <>
            <span className="timer-row__confirm-label">Discard?</span>
            <button className="timer-row__action timer-row__action--danger" onClick={handleDiscardClick}>
              Yes
            </button>
            <button className="timer-row__action" onClick={() => setConfirmingDiscard(false)}>
              No
            </button>
          </>
        ) : (
          <button className="timer-row__action timer-row__action--danger" onClick={handleDiscardClick}>
            Discard
          </button>
        )}
      </div>
    </li>
  );
}
