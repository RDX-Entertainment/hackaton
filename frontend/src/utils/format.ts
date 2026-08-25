/** Money is stored as a plain number and always shown with two decimals. */
export function formatMoney(amount: number): string {
  return `$${amount.toFixed(2)}`;
}

/**
 * El bitcoin viaja en satoshis: un entero de siete decimales. 10000001 son 1,0000001 BTC.
 * El servidor no ve una coma en su vida; ponerla es cosa de aquí.
 */
const SATOSHI = 10_000_000;
const BITCOIN_DECIMALS = 7;

export function formatBitcoin(satoshis: number): string {
  const whole = Math.trunc(satoshis / SATOSHI);
  const rest = Math.abs(satoshis % SATOSHI);

  return `${whole},${String(rest).padStart(BITCOIN_DECIMALS, '0')}`;
}

/**
 * Lo tecleado, de vuelta a satoshis. Acepta coma o punto porque el jugador escribe la que
 * tiene a mano, y devuelve null si lo escrito no es una cantidad.
 */
export function parseBitcoin(typed: string): number | null {
  const clean = typed.trim().replace(',', '.');

  if (!/^\d*\.?\d*$/.test(clean) || clean === '' || clean === '.') {
    return null;
  }

  const [whole, decimals = ''] = clean.split('.');

  if (decimals.length > BITCOIN_DECIMALS) {
    return null;
  }

  const padded = decimals.padEnd(BITCOIN_DECIMALS, '0');

  return Number(whole || '0') * SATOSHI + Number(padded || '0');
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
