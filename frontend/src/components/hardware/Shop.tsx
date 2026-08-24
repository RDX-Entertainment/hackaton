import { useState } from 'react';

import type { ComponentKind, Part } from '../../utils/api';
import { formatCount, formatMoney } from '../../utils/format';

type ShopProps = {
  parts: Part[];
  busy: boolean;
  onBuy: (part: Part) => void;
};

const TABS: { kind: ComponentKind; label: string }[] = [
  { kind: 'mobo', label: 'placas' },
  { kind: 'cpu', label: 'cpu' },
  { kind: 'ram', label: 'memoria' },
  { kind: 'hdd', label: 'disco' },
  { kind: 'net', label: 'red' },
];

export function Shop({ parts, busy, onBuy }: ShopProps) {
  const [kind, setKind] = useState<ComponentKind>('mobo');
  const shown = parts.filter((part) => part.kind === kind);

  return (
    <section className="panel">
      <header className="panel__head">
        <span className="panel__path">~/tienda</span>

        <span className="shop__tabs">
          {TABS.map((tab) => (
            <button
              type="button"
              key={tab.kind}
              className={tab.kind === kind ? 'is-active' : undefined}
              onClick={() => setKind(tab.kind)}
            >
              {tab.label}
            </button>
          ))}
        </span>
      </header>

      <div className="panel__body">
        {shown.length === 0 ? (
          <p className="panel__empty">Nada a la venta de este tipo.</p>
        ) : (
          <table className="shop">
            <thead>
              <tr>
                <th>pieza</th>
                <th>gama</th>
                <th>{kind === 'mobo' ? 'zócalos' : 'capacidad'}</th>
                <th>{kind === 'mobo' ? 'máximos' : 'ddr'}</th>
                <th>precio</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {shown.map((part) => (
                <tr key={part.id}>
                  <td>
                    {part.name}
                    {part.chipset && <small className="shop__chipset"> {part.chipset}</small>}
                  </td>
                  <td>
                    <span className={`tier tier--${part.tier}`}>{part.tier}</span>
                  </td>
                  <td>
                    {part.kind === 'mobo'
                      ? `${part.cpu_slots}c ${part.ram_slots}r ${part.hdd_slots}d ${part.net_slots}n`
                      : formatCount(part.power)}
                  </td>
                  <td>
                    {part.kind === 'mobo'
                      ? `${formatCount(part.max_ram)} MB · DDR${part.generation}`
                      : part.generation
                        ? `DDR${part.generation}`
                        : '—'}
                  </td>
                  <td>{formatMoney(part.price)}</td>
                  <td>
                    <button type="button" disabled={busy} onClick={() => onBuy(part)}>
                      comprar
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </section>
  );
}
