import { useEffect, useMemo, useState } from 'react';

import { browserName, collectSystemInfo, formatUptime } from '../utils/systemInfo';

/*
 * The wordmark, in the place a distro logo sits in a neofetch run. Solid blocks
 * rather than outlines, so it holds up at small sizes the way the Arch logo
 * does in the oh-my-zsh templates this copies.
 */
const LOGO = [
  ' ▄▄▄▄▄▄        ▄▄▄▄▄▄ ',
  ' ██████        ██████ ',
  ' ██████        ██████ ',
  ' ██████        ██████ ',
  ' ██████▄▄▄▄▄▄▄▄██████ ',
  ' ████████████████████ ',
  ' ██████▀▀▀▀▀▀▀▀██████ ',
  ' ██████        ██████ ',
  ' ██████        ██████ ',
  ' ██████        ██████ ',
  ' ▀▀▀▀▀▀        ▀▀▀▀▀▀ ',
].join('\n');

/** neofetch closes with two rows of palette swatches. */
const SWATCHES = [
  'var(--color-bg)',
  'var(--color-danger)',
  'var(--color-accent)',
  'var(--color-highlight)',
  'var(--color-accent-dim)',
  'var(--color-highlight-dim)',
  'var(--color-text-muted)',
  'var(--color-text)',
];

const TICK_MS = 1000;

export function SystemFetch() {
  // The probe touches the DOM and WebGL, so it runs once and is kept.
  const staticRows = useMemo(() => collectSystemInfo(), []);

  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), TICK_MS);
    return () => window.clearInterval(timer);
  }, []);

  const liveRows = [
    { key: 'uptime', value: formatUptime(performance.now()) },
    { key: 'time', value: new Date(now).toLocaleTimeString() },
    { key: 'date', value: new Date(now).toLocaleDateString() },
  ];

  const rows = [...staticRows, ...liveRows];
  const user = 'guest';
  const host = 'hackaton';

  return (
    <div className="fetch">
      <pre className="fetch__logo" role="img" aria-label="Hacker Experience">
        {LOGO}
      </pre>

      <div className="fetch__info">
        <p className="fetch__who">
          <b>{user}</b>@<b>{host}</b>
        </p>
        <p className="fetch__rule" aria-hidden="true">
          {'─'.repeat(user.length + host.length + 1)}
        </p>

        <dl className="fetch__rows">
          {rows.map((row) => (
            <div className="fetch__row" key={row.key}>
              <dt>{row.key}</dt>
              <dd>{row.value}</dd>
            </div>
          ))}
        </dl>

        <div className="fetch__swatches" aria-hidden="true">
          {SWATCHES.map((colour) => (
            <span key={colour} style={{ background: colour }} />
          ))}
        </div>

        <p className="fetch__note">
          read locally from {browserName()} · nothing leaves this machine
        </p>
      </div>
    </div>
  );
}
