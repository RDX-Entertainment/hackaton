import type { ComponentKind, Part, Tier } from '../../utils/api';
import { PartIcon } from './PartIcon';

export type CatalogQuery = {
  text: string;
  kind: ComponentKind | 'all';
  tier: Tier | 'all';
  onlyListed: boolean;
};

export const EMPTY_QUERY: CatalogQuery = {
  text: '',
  kind: 'all',
  tier: 'all',
  onlyListed: false,
};

export const KIND_LABEL: Record<ComponentKind, string> = {
  mobo: 'placas base',
  cpu: 'procesadores',
  ram: 'memorias',
  hdd: 'almacenamiento',
  net: 'red',
};

const KINDS: ComponentKind[] = ['mobo', 'cpu', 'ram', 'hdd', 'net'];

const TIERS: Tier[] = ['domestica', 'gaming', 'server', 'industrial'];

export function matches(part: Part, query: CatalogQuery): boolean {
  if (query.kind !== 'all' && part.kind !== query.kind) {
    return false;
  }

  if (query.tier !== 'all' && part.tier !== query.tier) {
    return false;
  }

  if (query.onlyListed && !part.listed) {
    return false;
  }

  const text = query.text.trim().toLowerCase();

  if (!text) {
    return true;
  }

  const haystack = [
    part.name,
    part.chipset ?? '',
    part.tier,
    KIND_LABEL[part.kind],
    part.generation ? `ddr${part.generation}` : '',
    String(part.id),
  ]
    .join(' ')
    .toLowerCase();

  return text.split(/\s+/).every((term) => haystack.includes(term));
}

type CatalogFiltersProps = {
  query: CatalogQuery;
  parts: Part[];
  shown: number;
  onChange: (query: CatalogQuery) => void;
};

export function CatalogFilters({ query, parts, shown, onChange }: CatalogFiltersProps) {
  const set = <K extends keyof CatalogQuery>(key: K, value: CatalogQuery[K]) =>
    onChange({ ...query, [key]: value });

  const countKind = (kind: ComponentKind) => parts.filter((part) => part.kind === kind).length;
  const countTier = (tier: Tier) => parts.filter((part) => part.tier === tier).length;

  const dirty =
    query.text !== '' || query.kind !== 'all' || query.tier !== 'all' || query.onlyListed;

  return (
    <div className="filters">
      <div className="filters__search">
        <span className="filters__icon" aria-hidden="true">
          ⌕
        </span>
        <input
          value={query.text}
          placeholder="buscar por nombre, chipset, gama, DDR…"
          onChange={(event) => set('text', event.target.value)}
        />
        {query.text && (
          <button type="button" aria-label="Limpiar búsqueda" onClick={() => set('text', '')}>
            ×
          </button>
        )}
      </div>

      <div className="filters__group">
        <span className="filters__legend">categoría</span>

        <div className="chips">
          <button
            type="button"
            className={query.kind === 'all' ? 'chip is-active' : 'chip'}
            onClick={() => set('kind', 'all')}
          >
            todas <b>{parts.length}</b>
          </button>

          {KINDS.map((kind) => (
            <button
              type="button"
              key={kind}
              className={query.kind === kind ? 'chip is-active' : 'chip'}
              onClick={() => set('kind', kind)}
            >
              <PartIcon kind={kind} className="chip__icon" />
              {KIND_LABEL[kind]} <b>{countKind(kind)}</b>
            </button>
          ))}
        </div>
      </div>

      <div className="filters__group">
        <span className="filters__legend">dominio</span>

        <div className="chips">
          <button
            type="button"
            className={query.tier === 'all' ? 'chip is-active' : 'chip'}
            onClick={() => set('tier', 'all')}
          >
            todos
          </button>

          {TIERS.map((tier) => (
            <button
              type="button"
              key={tier}
              className={query.tier === tier ? `chip chip--${tier} is-active` : `chip chip--${tier}`}
              onClick={() => set('tier', tier)}
            >
              {tier} <b>{countTier(tier)}</b>
            </button>
          ))}
        </div>
      </div>

      <div className="filters__foot">
        <label className="filters__toggle">
          <input
            type="checkbox"
            checked={query.onlyListed}
            onChange={(event) => set('onlyListed', event.target.checked)}
          />
          solo a la venta
        </label>

        <span className="filters__count">
          {shown} de {parts.length}
        </span>

        {dirty && (
          <button type="button" className="filters__reset" onClick={() => onChange(EMPTY_QUERY)}>
            limpiar
          </button>
        )}
      </div>
    </div>
  );
}
