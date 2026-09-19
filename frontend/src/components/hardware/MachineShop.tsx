import { tr, useIdioma } from '../../i18n/text';
import { useState, type FormEvent } from 'react';

import type { MachineQuote } from '../../utils/api';
import { formatMoney } from '../../utils/format';

type MachineShopProps = {
  quote: MachineQuote | null;
  busy: boolean;
  onBuy: (name: string) => void;
};

export function MachineShop({ quote, busy, onBuy }: MachineShopProps) {
  useIdioma();
  const [name, setName] = useState('');

  if (!quote) {
    return null;
  }

  const send = (event: FormEvent) => {
    event.preventDefault();
    onBuy(name.trim());
    setName('');
  };

  return (
    <section className="panel">
      <header className="panel__head">
        <span className="panel__path">~/tienda/torres</span>
        <span className="fleet__count">
          {quote.machines} {tr("de")} {quote.max} {tr("equipos")} </span>
      </header>

      <div className="panel__body">
        <div className="fleet">
          <div className="fleet__bays" aria-hidden="true">
            {Array.from({ length: quote.max }, (_, index) => (
              <span
                key={index}
                className={index < quote.machines ? 'fleet__bay is-taken' : 'fleet__bay'}
              />
            ))}
          </div>

          {quote.can_buy ? (
            <form className="fleet__form" onSubmit={send}>
              <label className="fleet__name">
                {tr("nombre")} <input
                  value={name}
                  maxLength={15}
                  placeholder={`Server #${quote.machines + 1}`}
                  onChange={(event) => setName(event.target.value)}
                />
              </label>

              <span className="fleet__price">{formatMoney(quote.price)}</span>

              <button className="is-primary" disabled={busy}>
                {tr("comprar torre")} </button>
            </form>
          ) : (
            <p className="fleet__full">
              {tr("Tienes los")} {quote.max} {tr("equipos que admite una cuenta. Da de baja alguno para montar otro.")} </p>
          )}

          <p className="fleet__hint">
            {tr("Una torre nace vacía: es un sitio donde poner una placa, no un ordenador. Cada equipo que compras cuesta el doble que el anterior, y todo lo que suman cuenta junto.")} </p>
        </div>
      </div>
    </section>
  );
}
