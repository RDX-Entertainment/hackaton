import { useEffect, useMemo, useRef, useState, type FormEvent } from 'react';

import type { Target, Visited } from '../../utils/api';
import { formatIp, isIp, parseIp } from '../../utils/ip';

export type Suggestion = {
  ip: number;
  name: string;
  origin: 'marcador' | 'historial' | 'conocida';
};

export function suggestionsFrom(
  bookmarks: Visited[],
  history: Visited[],
  targets: Target[],
): Suggestion[] {
  const seen = new Map<number, Suggestion>();

  const add = (ip: number, name: string, origin: Suggestion['origin']) => {
    if (!seen.has(ip)) {
      seen.set(ip, { ip, name, origin });
    }
  };

  bookmarks.forEach((one) => add(one.ip, one.name, 'marcador'));
  history.forEach((one) => add(one.ip, one.name, 'historial'));
  targets.forEach((one) => add(one.ip, one.name ?? '', 'conocida'));

  return [...seen.values()];
}

type AddressBarProps = {
  address: string;
  known: Suggestion[];
  loading: boolean;
  canBack: boolean;
  canForward: boolean;
  onAddressChange: (address: string) => void;
  onGo: (ip: number) => void;
  onBack: () => void;
  onForward: () => void;
  onReload: () => void;
};

export function AddressBar({
  address,
  known,
  loading,
  canBack,
  canForward,
  onAddressChange,
  onGo,
  onBack,
  onForward,
  onReload,
}: AddressBarProps) {
  const [open, setOpen] = useState(false);
  const [cursor, setCursor] = useState(0);

  const root = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) {
      return;
    }

    const close = (event: MouseEvent) => {
      if (!root.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };

    document.addEventListener('mousedown', close);
    return () => document.removeEventListener('mousedown', close);
  }, [open]);

  const found = useMemo(() => {
    const text = address.trim().toLowerCase();

    const matches = known.filter((one) => {
      if (!text) {
        return true;
      }

      return `${one.name} ${formatIp(one.ip)}`.toLowerCase().includes(text);
    });

    return matches.slice(0, 8);
  }, [known, address]);

  const valid = isIp(address);

  const go = (ip: number) => {
    setOpen(false);
    onAddressChange(formatIp(ip));
    onGo(ip);
  };

  const submit = (event: FormEvent) => {
    event.preventDefault();

    if (valid) {
      go(parseIp(address) as number);
      return;
    }

    // Escribir un nombre y pulsar enter va a la primera que encaje, como una
    // barra de direcciones de verdad: lo que tecleas no tiene que ser la dirección.
    if (found.length) {
      go(found[Math.min(cursor, found.length - 1)].ip);
    }
  };

  return (
    <div className="chrome" ref={root}>
      <div className="chrome__nav">
        <button
          type="button"
          className="chrome__step"
          aria-label="Atrás"
          title="Atrás"
          disabled={!canBack}
          onClick={onBack}
        >
          ◂
        </button>
        <button
          type="button"
          className="chrome__step"
          aria-label="Adelante"
          title="Adelante"
          disabled={!canForward}
          onClick={onForward}
        >
          ▸
        </button>
        <button
          type="button"
          className={loading ? 'chrome__step is-busy' : 'chrome__step'}
          aria-label="Recargar"
          title="Recargar"
          onClick={onReload}
        >
          ⟳
        </button>
      </div>

      <form className="chrome__bar" onSubmit={submit} autoComplete="off">
        <span className={valid ? 'chrome__scheme is-valid' : 'chrome__scheme'}>tcp://</span>

        <input
          value={address}
          placeholder="dirección o nombre de la máquina"
          spellCheck={false}
          onFocus={() => setOpen(true)}
          onChange={(event) => {
            onAddressChange(event.target.value);
            setCursor(0);
            setOpen(true);
          }}
          onKeyDown={(event) => {
            if (event.key === 'Escape') {
              setOpen(false);
            }

            if (event.key === 'ArrowDown') {
              event.preventDefault();
              setOpen(true);
              setCursor((current) => Math.min(current + 1, found.length - 1));
            }

            if (event.key === 'ArrowUp') {
              event.preventDefault();
              setCursor((current) => Math.max(current - 1, 0));
            }
          }}
        />

        <button className="chrome__go" disabled={!valid && found.length === 0}>
          conectar
        </button>
      </form>

      {open && found.length > 0 && (
        <ul className="chrome__hints">
          {found.map((one, index) => (
            <li key={one.ip}>
              <button
                type="button"
                className={index === cursor ? 'chrome__hint is-cursor' : 'chrome__hint'}
                onMouseEnter={() => setCursor(index)}
                onClick={() => go(one.ip)}
              >
                <span className="chrome__hintname">{one.name || 'sin nombre'}</span>
                <span className="chrome__hintip">{formatIp(one.ip)}</span>
                <span className={`chrome__origin chrome__origin--${one.origin}`}>
                  {one.origin}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
