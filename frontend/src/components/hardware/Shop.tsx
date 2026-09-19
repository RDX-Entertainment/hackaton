import { tr, useIdioma } from '../../i18n/text';
import { useState } from 'react';

import type { ComponentKind, Part } from '../../utils/api';
import { formatCount, formatMoney } from '../../utils/format';

type ShopProps = {
  parts: Part[];
  busy: boolean;
  onBuy: (part: Part) => void;
  canBuyExtras: boolean;
  onBuySwitch: () => void;
};

const TABS: { kind: ComponentKind | 'extras'; label: string }[] = [
  { kind: 'mobo', label: 'placas' },
  { kind: 'cpu', label: 'cpu' },
  { kind: 'ram', get label() { return tr("memoria"); } },
  { kind: 'hdd', get label() { return tr("disco"); } },
  { kind: 'net', get label() { return tr("red"); } },
  { kind: 'extras', label: 'Extras' },
];

export function Shop({ parts, busy, onBuy, canBuyExtras, onBuySwitch }: ShopProps) {
  useIdioma();
  const [kind, setKind] = useState<ComponentKind | 'extras'>('mobo');
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
        {kind === 'extras' ? (
          <div className="shop__extra">
            <div><b>{tr("Switch LAN · 8 puertos")}</b><p>{tr("Conecta equipos en tu red. Aparece directamente en el mapa y no consume una plaza de cómputo.")}</p></div>
            <span>{formatMoney(1000)}</span>
            <button type="button" disabled={busy || !canBuyExtras} onClick={onBuySwitch}>{tr("Comprar switch")}</button>
          </div>
        ) : shown.length === 0 ? (
          <p className="panel__empty">{tr("Nada a la venta de este tipo.")}</p>
        ) : (
          <table className="shop">
            <thead>
              <tr>
                <th>{tr("pieza")}</th>
                <th>{tr("gama")}</th>
                <th>{kind === 'mobo' ? tr("zócalos") : tr("capacidad")}</th>
                <th>{kind === 'mobo' ? tr("máximos") : 'ddr'}</th>
                <th>{tr("precio")}</th>
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
                      {tr("comprar")} </button>
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
