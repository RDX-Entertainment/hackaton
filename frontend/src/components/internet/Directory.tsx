import { useMemo, useState } from 'react';

import type { Target } from '../../utils/api';
import { formatIp } from '../../utils/ip';

type DirectoryProps = {
  targets: Target[];
  current: number | null;
  onVisit: (ip: number) => void;
};

/**
 * El listín, en un raíl a la derecha.
 *
 * Son 36 máquinas y creciendo, así que la lista scrollea dentro de su propia caja en vez
 * de estirar la página: el navegador se queda donde está mientras buscas dónde ir.
 */
export function Directory({ targets, current, onVisit }: DirectoryProps) {
  const [text, setText] = useState('');

  const found = useMemo(() => {
    const terms = text.trim().toLowerCase().split(/\s+/).filter(Boolean);

    if (!terms.length) {
      return targets;
    }

    return targets.filter((one) => {
      const words = `${one.name ?? ''} ${formatIp(one.ip)}`.toLowerCase();
      return terms.every((term) => words.includes(term));
    });
  }, [targets, text]);

  return (
    <aside className="dir">
      <header className="dir__head">
        <h4>Máquinas conocidas</h4>
        <span className="dir__count">{found.length}</span>
      </header>

      <div className="dir__search">
        <span aria-hidden="true">⌕</span>
        <input
          value={text}
          placeholder="filtrar…"
          onChange={(event) => setText(event.target.value)}
        />
        {text && (
          <button type="button" aria-label="Limpiar filtro" onClick={() => setText('')}>
            ×
          </button>
        )}
      </div>

      {found.length === 0 ? (
        <p className="dir__empty">Ninguna encaja.</p>
      ) : (
        <ul className="dir__list">
          {found.map((one) => (
            <li key={one.id}>
              <button
                type="button"
                className={one.ip === current ? 'dir__item is-here' : 'dir__item'}
                onClick={() => onVisit(one.ip)}
              >
                <span className="dir__name">{one.name || 'sin nombre'}</span>
                <span className="dir__ip">{formatIp(one.ip)}</span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </aside>
  );
}
