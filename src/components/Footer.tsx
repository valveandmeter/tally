import { formatHours, type Totals } from "../lib/api";

interface Props {
  totals: Totals | null;
}

export function Footer({ totals }: Props) {
  if (!totals) return null;

  return (
    <footer className="popover__footer">
      <span>
        Today {formatHours(totals.todayHours)} · This week {formatHours(totals.weekHours)}
      </span>
      {totals.pendingCount > 0 && (
        <span className="popover__footer-pending">
          {totals.pendingCount} pending
        </span>
      )}
    </footer>
  );
}
