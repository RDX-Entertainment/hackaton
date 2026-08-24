/*
 * Everything the browser will tell us about the machine it runs on, shaped as
 * the key/value rows neofetch prints next to a distro logo.
 *
 * All of it is read-only and local: no fingerprinting is sent anywhere, it just
 * gets drawn on the sign-in screen. Every field is probed defensively, because
 * half of these are non-standard and browser-specific — a missing one becomes a
 * dash rather than an exception.
 */

export type InfoRow = {
  key: string;
  value: string;
};

/** Fields that exist in Chromium but are not in the standard DOM typings. */
type ExtendedNavigator = Navigator & {
  userAgentData?: {
    platform?: string;
    brands?: { brand: string; version: string }[];
  };
};

const UNKNOWN = '—';

function nav(): ExtendedNavigator {
  return navigator as ExtendedNavigator;
}

/** Reads the browser name and major version out of the user agent. */
export function browserName(): string {
  const ua = navigator.userAgent;

  const candidates: [RegExp, string][] = [
    [/Firefox\/(\d+)/, 'Firefox'],
    [/Edg\/(\d+)/, 'Edge'],
    [/OPR\/(\d+)/, 'Opera'],
    [/Chrome\/(\d+)/, 'Chrome'],
    [/Version\/(\d+).*Safari/, 'Safari'],
  ];

  for (const [pattern, name] of candidates) {
    const match = ua.match(pattern);
    if (match) {
      return `${name} ${match[1]}`;
    }
  }

  return 'Unknown';
}

/** The rendering engine, which is what a kernel line is closest to here. */
function engineName(): string {
  const ua = navigator.userAgent;

  if (ua.includes('Firefox')) {
    return 'Gecko';
  }

  if (ua.includes('Chrome') || ua.includes('Chromium') || ua.includes('Edg')) {
    return 'Blink';
  }

  if (ua.includes('Safari')) {
    return 'WebKit';
  }

  return UNKNOWN;
}

function platformName(): string {
  const hinted = nav().userAgentData?.platform;
  if (hinted) {
    return hinted;
  }

  const ua = navigator.userAgent;
  const known = ['Windows NT 10.0', 'Windows', 'Mac OS X', 'Linux', 'Android', 'iPhone'];
  const found = known.find((name) => ua.includes(name));

  if (found === 'Windows NT 10.0') {
    return 'Windows 10/11';
  }

  return found ?? UNKNOWN;
}

/** Formats milliseconds the way uptime(1) does. */
export function formatUptime(milliseconds: number): string {
  const totalSeconds = Math.floor(milliseconds / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours) {
    return `${hours}h ${minutes}m ${seconds}s`;
  }

  if (minutes) {
    return `${minutes}m ${seconds}s`;
  }

  return `${seconds}s`;
}

function yesNo(value: boolean): string {
  return value ? 'yes' : 'no';
}

/**
 * The static half of the panel: everything that will not change while the page
 * is open. The uptime and clock rows are appended by the component, which
 * re-reads them every second.
 */
export function collectSystemInfo(): InfoRow[] {
  const rows: InfoRow[] = [];

  const push = (key: string, value: string | number | undefined | null) => {
    rows.push({ key, value: value === undefined || value === null ? UNKNOWN : String(value) });
  };

  push('os', platformName());
  push('host', window.location.host);
  push('kernel', `${engineName()} · ${browserName()}`);
  push('shell', 'bash 5.2.21');
  push('de', 'React 19');
  push('wm', 'Vite 6 · Redux Toolkit');
  push('terminal', browserName());
  push('resolution', `${screen.width}x${screen.height} @${window.devicePixelRatio}x`);
  push('viewport', `${window.innerWidth}x${window.innerHeight}`);
  push('colors', `${screen.colorDepth}-bit`);
  push('locale', navigator.language);
  push('languages', navigator.languages.slice(0, 3).join(' '));
  push('timezone', Intl.DateTimeFormat().resolvedOptions().timeZone);
  push('online', yesNo(navigator.onLine));
  push('theme-os', window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
  push('motion', window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'reduced' : 'full');

  return rows;
}
