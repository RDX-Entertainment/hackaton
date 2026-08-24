import { useMemo, useState } from 'react';

import type { ComponentKind, Part } from '../../utils/api';
import { AdminCard } from './AdminCard';
import { CatalogFilters, EMPTY_QUERY, KIND_LABEL, matches, type CatalogQuery } from './CatalogFilters';
import { PartCard } from './PartCard';

const COLLAPSED = 8;

const ORDER: ComponentKind[] = ['mobo', 'cpu', 'ram', 'hdd', 'net'];

type CatalogBrowserProps = {
  parts: Part[];
  busy: boolean;
  onEdit: (part: Part) => void;
  onRemove: (part: Part) => void;
};

export function CatalogBrowser({ parts, busy, onEdit, onRemove }: CatalogBrowserProps) {
  const [query, setQuery] = useState<CatalogQuery>(EMPTY_QUERY);
  const [expanded, setExpanded] = useState(false);

  const shown = useMemo(() => {
    const found = parts.filter((part) => matches(part, query));

    return ORDER.flatMap((kind) => found.filter((part) => part.kind === kind));
  }, [parts, query]);

  const visible = expanded ? shown : shown.slice(0, COLLAPSED);

  const grouped = useMemo(
    () =>
      ORDER.map((kind) => [kind, visible.filter((part) => part.kind === kind)] as const).filter(
        ([, group]) => group.length > 0,
      ),
    [visible],
  );

  const hidden = shown.length - visible.length;

  return (
    <AdminCard
      prefix="Hard"
      brand="Menu"
      note={`${shown.length} de ${parts.length} piezas`}
    >
      <div className="catalog">
        <CatalogFilters
          query={query}
          parts={parts}
          shown={shown.length}
          onChange={(next) => {
            setQuery(next);
            setExpanded(false);
          }}
        />

        {shown.length === 0 ? (
          <p className="catalog__empty">Ninguna pieza encaja con esos filtros.</p>
        ) : (
          <>
            {grouped.map(([kind, group]) => (
              <section className="catalog__group" key={kind}>
                <h4 className="catalog__title">
                  {KIND_LABEL[kind]} <span className="catalog__count">{group.length}</span>
                </h4>

                <div className="catalog__grid">
                  {group.map((part) => (
                    <PartCard
                      key={part.id}
                      part={part}
                      busy={busy}
                      onEdit={onEdit}
                      onRemove={onRemove}
                    />
                  ))}
                </div>
              </section>
            ))}

            {(hidden > 0 || expanded) && (
              <button
                type="button"
                className="catalog__more"
                aria-expanded={expanded}
                onClick={() => setExpanded((current) => !current)}
              >
                <span className={expanded ? 'catalog__chevron is-up' : 'catalog__chevron'}>▾</span>
                {expanded ? 'ver menos' : `ver ${hidden} piezas más`}
              </button>
            )}
          </>
        )}
      </div>
    </AdminCard>
  );
}
