interface Props {
  clientName: string;
  suggestedTitle: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function CatchAllConfirm({ clientName, suggestedTitle, onConfirm, onCancel }: Props) {
  return (
    <div className="catch-all-confirm">
      <p>
        No catch-all task exists yet for <strong>{clientName}</strong>.
      </p>
      <p>
        Create "{suggestedTitle}" in Wrike and log there?
      </p>
      <div className="catch-all-confirm__actions">
        <button className="timer-row__action timer-row__action--primary" onClick={onConfirm}>
          Create &amp; log
        </button>
        <button className="timer-row__action" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}
