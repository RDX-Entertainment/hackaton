/** Money is stored as a plain number and always shown with two decimals. */
export function formatMoney(amount: number): string {
  return `$${amount.toFixed(2)}`;
}

/** Turns an unknown cell coming from a generic endpoint into printable text. */
export function formatCell(value: unknown): string {
  if (value === null || value === undefined) {
    return '';
  }

  return String(value);
}

/** Large counters read better broken up: 1234567 → "1.234.567". */
export function formatCount(value: number): string {
  return new Intl.NumberFormat('es-ES').format(Math.trunc(value));
}

/** Hours as the stats table counts them: 3.5 → "3h 30m". */
export function formatPlaytime(hours: number): string {
  const totalMinutes = Math.round(hours * 60);
  const whole = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;

  if (!whole) {
    return `${minutes}m`;
  }

  return `${whole}h ${minutes}m`;
}

/** An ISO timestamp as a plain local date, or a dash when there is none. */
export function formatDate(iso: string | null | undefined): string {
  if (!iso) {
    return '—';
  }

  const date = new Date(iso);
  return Number.isNaN(date.getTime()) ? '—' : date.toLocaleDateString();
}

/** Whole days since a timestamp, for "member for N days". */
export function daysSince(iso: string | null | undefined): number | null {
  if (!iso) {
    return null;
  }

  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) {
    return null;
  }

  return Math.max(0, Math.floor((Date.now() - then) / 86400000));
}
