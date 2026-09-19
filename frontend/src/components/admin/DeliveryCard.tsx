import { tr, useIdioma, idiomaActual } from '../../i18n/text';
import { useMemo, useState, type FormEvent } from 'react';

import type { AdminUser, Part } from '../../utils/api';
import { formatCount, formatMoney } from '../../utils/format';
import { formatIp } from '../../utils/ip';
import { SearchSelect, type SearchOption } from '../SearchSelect';
import { AdminCard } from './AdminCard';
import { PartIcon } from './PartIcon';

const UNIT: Record<string, string> = {
  cpu: 'MHz',
  ram: 'MB',
  hdd: 'MB',
  net: 'Mbit',
};

function specOf(part: Part): string {
  if (part.kind === 'mobo') {
    return `${part.chipset ?? tr("sin chipset")} · DDR${part.generation}`;
  }

  return `${formatCount(part.power)} ${UNIT[part.kind] ?? ''}`;
}

type DeliveryCardProps = {
  parts: Part[];
  users: AdminUser[];
  busy: boolean;
  onDeliver: (part: number, user: number, amount: number) => void;
};

export function DeliveryCard({ parts, users, busy, onDeliver }: DeliveryCardProps) {
  useIdioma();
  const [part, setPart] = useState<number | null>(null);
  const [user, setUser] = useState<number | null>(null);
  const [amount, setAmount] = useState('1');

  const chosen = parts.find((option) => option.id === part);
  const target = users.find((option) => option.id === user);

  const partOptions: SearchOption[] = useMemo(
    () =>
      parts.map((option) => ({
        id: option.id,
        label: option.name,
        hint: specOf(option),
        terms: `${option.kind} ${option.tier} ${option.chipset ?? ''} ddr${option.generation}`,
        icon: <PartIcon kind={option.kind} className="ssel__icon" />,
        badge: <span className={`tier tier--${option.tier}`}>{option.tier}</span>,
      })),
    [parts, idiomaActual()],
  );

  const userOptions: SearchOption[] = useMemo(
    () =>
      users.map((option) => ({
        id: option.id,
        label: option.login,
        hint: tr("{0} piezas · {1}", option.components, formatIp(option.game_ip)),
        terms: `${option.email} ${option.is_admin ? 'admin' : 'jugador'} ${option.id}`,
        badge: option.is_admin ? <span className="role role--admin">admin</span> : undefined,
      })),
    [users, idiomaActual()],
  );

  const ready = Boolean(chosen && target);

  const send = (event: FormEvent) => {
    event.preventDefault();

    if (!chosen || !target) {
      return;
    }

    onDeliver(chosen.id, target.id, Number(amount) || 1);
  };

  return (
    <AdminCard
      prefix="Hard"
      brand="Delivery"
      accent="lime"
      note={ready ? `${amount} × ${chosen?.name} → ${target?.login}` : tr("entrega directa")}
    >
      <form className="delivery" onSubmit={send}>
        <div className="delivery__fields">
          <SearchSelect
            label={tr("pieza")}
            options={partOptions}
            value={part}
            onChange={setPart}
            placeholder={tr("nombre, gama, chipset, DDR…")}
            emptyLabel={tr("elige una pieza")}
            noResults="ninguna pieza encaja"
          />

          <SearchSelect
            label="destinatario"
            options={userOptions}
            value={user}
            onChange={setUser}
            placeholder={tr("login, email, id…")}
            emptyLabel={tr("elige un jugador")}
            noResults="ningún jugador encaja"
          />

          <label className="delivery__amount">
            {tr("cantidad")} <input
              inputMode="numeric"
              value={amount}
              onChange={(event) => setAmount(event.target.value)}
              required
            />
          </label>

          <button className="is-primary" disabled={busy || !ready}>
            {tr("entregar")} </button>
        </div>

        <div className={ready ? 'delivery__slip is-ready' : 'delivery__slip'}>
          {chosen ? (
            <>
              <PartIcon kind={chosen.kind} className="delivery__icon" />

              <div className="delivery__detail">
                <p className="delivery__part">{chosen.name}</p>
                <p className="delivery__spec">
                  {specOf(chosen)} · {formatMoney(chosen.price)}
                </p>
              </div>

              <span className={`tier tier--${chosen.tier}`}>{chosen.tier}</span>
            </>
          ) : (
            <p className="delivery__idle">{tr("Sin pieza seleccionada.")}</p>
          )}
        </div>

        <p className="delivery__hint">
          {tr("Las piezas entregadas van al cajón del jugador sin instalar y sin cobrarle nada.")} </p>
      </form>
    </AdminCard>
  );
}
