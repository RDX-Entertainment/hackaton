import { useState } from 'react';

import type { Kit } from '../../utils/api';
import { formatCount, formatMoney } from '../../utils/format';

type KitShopProps = {
  kits: Kit[];
  busy: boolean;
  onBuy: (kit: Kit) => void;
};

export function KitShop({ kits, busy, onBuy }: KitShopProps) {
  const [open, setOpen] = useState<number | null>(null);

  if (kits.length === 0) {
    return null;
  }

  return (
    <section className="panel">
      <header className="panel__head">
        <span className="panel__path">~/tienda/equipos</span>
        <span className="panel__hint">montados y listos, en una máquina nueva</span>
      </header>

      <div className="panel__body">
        <ul className="kitshop">
          {kits.map((kit) => {
            const board = kit.items.find((item) => item.kind === 'mobo');
            const saving =
              kit.totals.parts_price > 0
                ? Math.round((1 - kit.price / kit.totals.parts_price) * 100)
                : 0;

            return (
              <li key={kit.id} className={open === kit.id ? 'kitoffer is-open' : 'kitoffer'}>
                <button
                  type="button"
                  className="kitoffer__head"
                  aria-expanded={open === kit.id}
                  onClick={() => setOpen(open === kit.id ? null : kit.id)}
                >
                  <span className="kitoffer__chevron" aria-hidden="true">
                    ▸
                  </span>

                  <span className="kitoffer__name">{kit.name}</span>

                  {board && <span className={`tier tier--${board.tier}`}>{board.tier}</span>}

                  {kit.scope === 'clan' && <span className="kitoffer__clan">corporación</span>}

                  <span className="kitoffer__specs">
                    {formatCount(kit.totals.cpu)} MHz · {formatCount(kit.totals.ram)} MB ·{' '}
                    {formatCount(kit.totals.hdd)} MB · {formatCount(kit.totals.net)} Mbit
                  </span>

                  <span className="kitoffer__price">{formatMoney(kit.price)}</span>

                  {saving > 0 && <span className="kitoffer__saving">−{saving}%</span>}
                </button>

                {open === kit.id && (
                  <div className="kitoffer__body">
                    {kit.description && <p className="kitoffer__desc">{kit.description}</p>}

                    <ul className="kitoffer__parts">
                      {kit.items.map((item) => (
                        <li key={item.part}>
                          {item.amount > 1 && <b>{item.amount}×</b>} {item.name}
                          <small>
                            {item.kind === 'mobo'
                              ? `${item.chipset ?? 'sin chipset'} · DDR${item.generation}`
                              : formatCount(item.power)}
                          </small>
                        </li>
                      ))}
                    </ul>

                    <p className="kitoffer__foot">
                      <span>{formatMoney(kit.totals.parts_price)} comprando suelto</span>

                      <button
                        type="button"
                        className="is-primary"
                        disabled={busy}
                        onClick={() => onBuy(kit)}
                      >
                        comprar montado
                      </button>
                    </p>
                  </div>
                )}
              </li>
            );
          })}
        </ul>
      </div>
    </section>
  );
}
