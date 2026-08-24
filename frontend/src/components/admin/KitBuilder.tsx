import { useMemo } from 'react';

import type { ComponentKind, Part } from '../../utils/api';
import { formatCount, formatMoney } from '../../utils/format';
import { KIND_UNIT } from '../../utils/kitBuild';
import { SearchSelect, type SearchOption } from '../SearchSelect';
import { PartIcon } from './PartIcon';

export type BuildLine = {
  part: number;
  amount: number;
};

const ORDER: ComponentKind[] = ['mobo', 'cpu', 'ram', 'hdd', 'net'];

function specOf(part: Part): string {
  if (part.kind === 'mobo') {
    return `${part.chipset ?? 'sin chipset'} · DDR${part.generation} · ${part.cpu_slots}c ${part.ram_slots}r ${part.hdd_slots}d ${part.net_slots}n`;
  }

  return `${formatCount(part.power)} ${KIND_UNIT[part.kind]}${part.kind === 'ram' ? ` · DDR${part.generation}` : ''}${part.kind === 'cpu' && part.chipset ? ` · ${part.chipset}` : ''}`;
}

type KitBuilderProps = {
  parts: Part[];
  lines: BuildLine[];
  onChange: (lines: BuildLine[]) => void;
};

export function KitBuilder({ parts, lines, onChange }: KitBuilderProps) {
  const byId = useMemo(() => new Map(parts.map((part) => [part.id, part])), [parts]);

  const taken = new Set(lines.map((line) => line.part));

  const options: SearchOption[] = useMemo(
    () =>
      ORDER.flatMap((kind) => parts.filter((part) => part.kind === kind))
        .filter((part) => !taken.has(part.id))
        .map((part) => ({
          id: part.id,
          label: part.name,
          hint: specOf(part),
          terms: `${part.kind} ${part.tier} ${part.chipset ?? ''} ddr${part.generation} ${formatMoney(part.price)}`,
          icon: <PartIcon kind={part.kind} className="ssel__icon" />,
          badge: <span className={`tier tier--${part.tier}`}>{part.tier}</span>,
        })),
    [parts, lines],
  );

  const add = (id: number | null) => {
    if (id === null || taken.has(id)) {
      return;
    }

    onChange([...lines, { part: id, amount: 1 }]);
  };

  const setAmount = (part: number, amount: number) =>
    onChange(
      lines.map((line) =>
        line.part === part ? { ...line, amount: Math.min(32, Math.max(1, amount)) } : line,
      ),
    );

  const drop = (part: number) => onChange(lines.filter((line) => line.part !== part));

  const sorted = [...lines].sort((left, right) => {
    const a = byId.get(left.part);
    const b = byId.get(right.part);

    return ORDER.indexOf(a?.kind ?? 'net') - ORDER.indexOf(b?.kind ?? 'net');
  });

  return (
    <div className="kitbuild">
      <SearchSelect
        label="añadir pieza"
        options={options}
        value={null}
        onChange={add}
        placeholder="nombre, gama, chipset, DDR…"
        emptyLabel="elige del catálogo"
        noResults="ninguna pieza encaja"
      />

      {sorted.length === 0 ? (
        <p className="kitbuild__empty">
          Todavía no hay piezas. Empieza por la placa: es la que decide qué entra después.
        </p>
      ) : (
        <ul className="kitbuild__list">
          {sorted.map((line) => {
            const part = byId.get(line.part);

            if (!part) {
              return null;
            }

            const board = part.kind === 'mobo';

            return (
              <li key={line.part} className={board ? 'kitline is-board' : 'kitline'}>
                <PartIcon kind={part.kind} className="kitline__icon" />

                <span className="kitline__text">
                  <span className="kitline__name">{part.name}</span>
                  <span className="kitline__spec">{specOf(part)}</span>
                </span>

                <span className={`tier tier--${part.tier}`}>{part.tier}</span>

                <span className="kitline__price">{formatMoney(part.price * line.amount)}</span>

                {board ? (
                  <span className="kitline__single">×1</span>
                ) : (
                  <span className="kitline__amount">
                    <button
                      type="button"
                      aria-label={`Una menos de ${part.name}`}
                      disabled={line.amount <= 1}
                      onClick={() => setAmount(line.part, line.amount - 1)}
                    >
                      −
                    </button>
                    <b>{line.amount}</b>
                    <button
                      type="button"
                      aria-label={`Una más de ${part.name}`}
                      disabled={line.amount >= 32}
                      onClick={() => setAmount(line.part, line.amount + 1)}
                    >
                      +
                    </button>
                  </span>
                )}

                <button
                  type="button"
                  className="kitline__drop"
                  aria-label={`Quitar ${part.name}`}
                  onClick={() => drop(line.part)}
                >
                  ×
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
