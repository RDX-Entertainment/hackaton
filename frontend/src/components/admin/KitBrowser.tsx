import { tr, useIdioma, idiomaActual } from '../../i18n/text';
import { useMemo, useState } from 'react';

import type { Kit, KitScope } from '../../utils/api';
import { AdminCard } from './AdminCard';
import { KitCard } from './KitCard';

const COLLAPSED = 4;

const SCOPES: { value: KitScope | 'all'; label: string }[] = [
  { value: 'all', get label() { return tr("todos"); } },
  { value: 'shop', label: 'tienda' },
  { value: 'clan', get label() { return tr("corporación"); } },
  { value: 'marketplace', label: 'mercado' },
  { value: 'hidden', get label() { return tr("oculto"); } },
];

type KitBrowserProps = {
  kits: Kit[];
  busy: boolean;
  onEdit: (kit: Kit) => void;
  onRemove: (kit: Kit) => void;
  onDefault: (kit: Kit) => void;
};

export function KitBrowser({ kits, busy, onEdit, onRemove, onDefault }: KitBrowserProps) {
  useIdioma();
  const [scope, setScope] = useState<KitScope | 'all'>('all');
  const [text, setText] = useState('');
  const [expanded, setExpanded] = useState(false);

  const shown = useMemo(() => {
    const terms = text.trim().toLowerCase().split(/\s+/).filter(Boolean);

    return kits.filter((kit) => {
      if (scope !== 'all' && kit.scope !== scope) {
        return false;
      }

      if (!terms.length) {
        return true;
      }

      const haystack = [
        kit.name,
        kit.description,
        kit.scope,
        ...kit.items.map((item) => item.name),
        ...kit.clans.map((clan) => `${clan.nick} ${clan.name}`),
      ]
        .join(' ')
        .toLowerCase();

      return terms.every((term) => haystack.includes(term));
    });
  }, [kits, scope, text, idiomaActual()]);

  const visible = expanded ? shown : shown.slice(0, COLLAPSED);
  const hidden = shown.length - visible.length;

  const count = (value: KitScope | 'all') =>
    value === 'all' ? kits.length : kits.filter((kit) => kit.scope === value).length;

  return (
    <AdminCard prefix="Kit" brand="Menu" note={tr("{0} de {1} equipos", shown.length, kits.length)}>
      <div className="catalog">
        <div className="filters">
          <div className="filters__search">
            <span className="filters__icon" aria-hidden="true">
              ⌕
            </span>
            <input
              value={text}
              placeholder={tr("buscar por nombre, pieza, corporación…")}
              onChange={(event) => {
                setText(event.target.value);
                setExpanded(false);
              }}
            />
            {text && (
              <button type="button" aria-label={tr("Limpiar búsqueda")} onClick={() => setText('')}>
                ×
              </button>
            )}
          </div>

          <div className="filters__group">
            <span className="filters__legend">{tr("ámbito")}</span>

            <div className="chips">
              {SCOPES.map((option) => (
                <button
                  type="button"
                  key={option.value}
                  className={scope === option.value ? 'chip is-active' : 'chip'}
                  onClick={() => {
                    setScope(option.value);
                    setExpanded(false);
                  }}
                >
                  {option.label} <b>{count(option.value)}</b>
                </button>
              ))}
            </div>
          </div>
        </div>

        {shown.length === 0 ? (
          <p className="catalog__empty">{tr("Ningún kit encaja con esos filtros.")}</p>
        ) : (
          <>
            <div className="kitgrid">
              {visible.map((kit) => (
                <KitCard
                  key={kit.id}
                  kit={kit}
                  busy={busy}
                  onEdit={onEdit}
                  onRemove={onRemove}
                  onDefault={onDefault}
                />
              ))}
            </div>

            {(hidden > 0 || expanded) && (
              <button
                type="button"
                className="catalog__more"
                aria-expanded={expanded}
                onClick={() => setExpanded((current) => !current)}
              >
                <span className={expanded ? 'catalog__chevron is-up' : 'catalog__chevron'}>▾</span>
                {expanded ? tr("ver menos") : tr("ver {0} equipos más", hidden)}
              </button>
            )}
          </>
        )}
      </div>
    </AdminCard>
  );
}
