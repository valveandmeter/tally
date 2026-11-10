import { useEffect, useMemo, useRef, useState } from "react";
import { searchWrike, setClientPinned, type ClientResult, type TaskResult } from "../lib/api";

const DEBOUNCE_MS = 250;

interface Props {
  onSelectTask: (task: TaskResult) => void;
  onSelectBucket: (client: ClientResult) => void;
  onClose: () => void;
}

type Row =
  | { type: "client"; client: ClientResult }
  | { type: "bucket"; client: ClientResult }
  | { type: "task"; task: TaskResult };

export function SearchPanel({ onSelectTask, onSelectBucket, onClose }: Props) {
  const [query, setQuery] = useState("");
  const [clientFilter, setClientFilter] = useState<ClientResult | null>(null);
  const [assignedToMe, setAssignedToMe] = useState(false);
  const [includeCompleted, setIncludeCompleted] = useState(false);
  const [clients, setClients] = useState<ClientResult[]>([]);
  const [tasks, setTasks] = useState<TaskResult[]>([]);
  const [highlighted, setHighlighted] = useState(0);
  const debounceRef = useRef<number | undefined>(undefined);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const runSearch = () => {
    searchWrike(query, clientFilter?.id ?? null, assignedToMe, includeCompleted).then((results) => {
      setClients(results.clients);
      setTasks(results.tasks);
      setHighlighted(0);
    });
  };

  useEffect(() => {
    window.clearTimeout(debounceRef.current);
    debounceRef.current = window.setTimeout(runSearch, DEBOUNCE_MS);
    return () => window.clearTimeout(debounceRef.current);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [query, clientFilter, assignedToMe, includeCompleted]);

  const rows: Row[] = useMemo(() => {
    const clientRows: Row[] = clientFilter ? [] : clients.map((client) => ({ type: "client", client }));
    const bucketRow: Row[] = clientFilter ? [{ type: "bucket", client: clientFilter }] : [];
    const taskRows: Row[] = tasks.map((task) => ({ type: "task", task }));
    return [...clientRows, ...bucketRow, ...taskRows];
  }, [clients, tasks, clientFilter]);

  const selectRow = (row: Row) => {
    if (row.type === "client") {
      setClientFilter(row.client);
      setQuery("");
    } else if (row.type === "bucket") {
      onSelectBucket(row.client);
    } else {
      onSelectTask(row.task);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setHighlighted((i) => Math.min(i + 1, rows.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setHighlighted((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const row = rows[highlighted];
      if (row) selectRow(row);
    } else if (e.key === "Escape") {
      e.preventDefault();
      if (clientFilter) {
        setClientFilter(null);
      } else {
        onClose();
      }
    }
  };

  const togglePin = async (client: ClientResult, e: React.MouseEvent) => {
    e.stopPropagation();
    await setClientPinned(client.id, !client.pinned);
    runSearch();
  };

  return (
    <div className="search-panel">
      <div className="search-panel__bar">
        <button className="search-panel__back" onClick={onClose} aria-label="Back">
          ←
        </button>
        {clientFilter && (
          <span className="search-panel__chip">
            {clientFilter.name}
            <button onClick={() => setClientFilter(null)} aria-label="Clear client filter">
              ×
            </button>
          </span>
        )}
        <input
          ref={inputRef}
          className="search-panel__input"
          value={query}
          onChange={(e) => setQuery(e.currentTarget.value)}
          onKeyDown={handleKeyDown}
          placeholder={clientFilter ? "Search tasks…" : "Search clients or tasks…"}
        />
      </div>

      <div className="search-panel__filters">
        <label>
          <input
            type="checkbox"
            checked={assignedToMe}
            onChange={(e) => setAssignedToMe(e.currentTarget.checked)}
          />
          Assigned to me
        </label>
        <label>
          <input
            type="checkbox"
            checked={includeCompleted}
            onChange={(e) => setIncludeCompleted(e.currentTarget.checked)}
          />
          Include completed
        </label>
      </div>

      <div className="search-panel__results">
        {!clientFilter && clients.length > 0 && (
          <div className="search-panel__group-label">Clients</div>
        )}
        {!clientFilter &&
          clients.map((client) => {
            const rowIndex = rows.findIndex((r) => r.type === "client" && r.client.id === client.id);
            return (
              <div
                key={client.id}
                className={`search-row${rowIndex === highlighted ? " search-row--highlighted" : ""}`}
                onClick={() => setClientFilter(client)}
              >
                <button className="search-row__pin" onClick={(e) => togglePin(client, e)} aria-label="Pin client">
                  {client.pinned ? "★" : "☆"}
                </button>
                <span className="search-row__label">{client.name}</span>
                <span className="search-row__meta">{client.openTaskCount} open</span>
                <button
                  className="search-row__action"
                  onClick={(e) => {
                    e.stopPropagation();
                    onSelectBucket(client);
                  }}
                >
                  Track time
                </button>
              </div>
            );
          })}

        {clientFilter && (
          <div
            className={`search-row search-row--bucket${
              rows[0]?.type === "bucket" && highlighted === 0 ? " search-row--highlighted" : ""
            }`}
            onClick={() => onSelectBucket(clientFilter)}
          >
            <span className="search-row__label">Track to {clientFilter.name} (no task)</span>
          </div>
        )}

        {tasks.length > 0 && <div className="search-panel__group-label">Tasks</div>}
        {tasks.map((task) => {
          const rowIndex = rows.findIndex((r) => r.type === "task" && r.task.id === task.id);
          return (
            <div
              key={task.id}
              className={`search-row${rowIndex === highlighted ? " search-row--highlighted" : ""}`}
              onClick={() => onSelectTask(task)}
            >
              <span className="search-row__label">
                {task.clientName}
                {task.projectPath ? ` › ${task.projectPath}` : ""} › {task.title}
              </span>
              <span className="search-row__meta">{task.status}</span>
            </div>
          );
        })}

        {rows.length === 0 && <p className="search-panel__empty">No matches.</p>}
      </div>
    </div>
  );
}
