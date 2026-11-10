import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type TimerStatus = "running" | "paused";

export interface Timer {
  id: string;
  label: string;
  status: TimerStatus;
  elapsedSeconds: number;
}

export interface TimersState {
  timers: Timer[];
  runningId: string | null;
}

export interface AutoPaused {
  label: string;
  elapsedSeconds: number;
}

export function listTimers(): Promise<Timer[]> {
  return invoke("list_timers");
}

export function createTimer(label: string): Promise<Timer> {
  return invoke("create_timer", { label });
}

export function startTimer(id: string): Promise<AutoPaused | null> {
  return invoke("start_timer", { id });
}

export function pauseTimer(id: string): Promise<void> {
  return invoke("pause_timer", { id });
}

export function renameTimer(id: string, label: string): Promise<void> {
  return invoke("rename_timer", { id, label });
}

export function discardTimer(id: string): Promise<void> {
  return invoke("discard_timer", { id });
}

export function onTimersState(handler: (state: TimersState) => void): Promise<UnlistenFn> {
  return listen<TimersState>("timers://state", (event) => handler(event.payload));
}

export function formatElapsed(totalSeconds: number): string {
  const seconds = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${hours}:${pad(minutes)}:${pad(secs)}`;
}
