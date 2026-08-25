import { useEffect, useState, type FormEvent } from 'react';

import { bankApi, financesApi, type BankAccount, type SiteAction } from '../../utils/api';
import { messageFrom } from '../../utils/errors';
import { formatMoney } from '../../utils/format';

type SitePayDialogProps = {
  action: SiteAction | null;
  onClose: () => void;
};

/**
 * Lo que sale cuando un sitio ofrece cobrar.
 *
 * La página solo declara la intención: el importe, la cuenta que recibe y un concepto. Quien
 * pide el PIN es el juego, aquí fuera, y nunca el sitio. Esa separación es lo que convierte
 * un cobro en una decisión del jugador en vez de en un robo automático.
 *
 * Que el concepto sea mentira es asunto de quien lo escribió: eso es el juego.
 */
export function SitePayDialog({ action, onClose }: SitePayDialogProps) {
  const [accounts, setAccounts] = useState<BankAccount[]>([]);
  const [from, setFrom] = useState(0);
  const [pin, setPin] = useState('');
  const [failed, setFailed] = useState('');
  const [done, setDone] = useState('');
  const [sending, setSending] = useState(false);

  useEffect(() => {
    if (!action) {
      return;
    }

    setFailed('');
    setDone('');
    setPin('');
    financesApi
      .listAccounts()
      .then((rows) => {
        const usable = rows.filter((row) => row.pin_set);
        setAccounts(usable);
        setFrom(usable[0]?.account ?? 0);
      })
      .catch(() => setAccounts([]));
  }, [action]);

  if (!action) {
    return null;
  }

  const amount = Number(action.params?.amount ?? 0);
  const concept = action.params?.concept ?? action.label;
  const kind = action.kind === 'charge' ? 'Cobro' : 'Pago';
  const target = Number(action.target);
  const source = accounts.find((one) => one.account === from);
  const short = source ? source.cash < amount : false;

  const pay = async (event: FormEvent) => {
    event.preventDefault();
    setSending(true);
    setFailed('');

    try {
      const movement = await bankApi.transfer({
        account: from,
        pin,
        to_account: target,
        amount,
        concept,
      });

      setDone(`Pagado. Te quedan ${formatMoney(movement.balance)}.`);
      setPin('');
    } catch (error) {
      setFailed(messageFrom(error));
    } finally {
      setSending(false);
    }
  };

  return (
    <div className="site-pay">
      <form onSubmit={pay}>
        <header>
          <b>{kind} · {concept}</b>
          <button type="button" onClick={onClose}>
            cerrar
          </button>
        </header>

        <dl>
          <div>
            <dt>Importe</dt>
            <dd>{formatMoney(amount)}</dd>
          </div>
          <div>
            <dt>A la cuenta</dt>
            <dd>{action.target}</dd>
          </div>
          <div>
            <dt>Concepto</dt>
            <dd>{concept}</dd>
          </div>
        </dl>

        {done ? (
          <p className="site-pay__done">{done}</p>
        ) : (
          <>
            <label>
              Pagar desde
              <select
                value={from}
                onChange={(event) => setFrom(Number(event.target.value))}
                disabled={!accounts.length}
              >
                {accounts.length ? (
                  accounts.map((one) => (
                    <option key={one.id} value={one.account}>
                      {one.account} · {formatMoney(one.cash)}
                    </option>
                  ))
                ) : (
                  <option value={0}>Sin cuentas con PIN</option>
                )}
              </select>
            </label>

            <label>
              PIN
              <input
                type="password"
                value={pin}
                maxLength={6}
                autoComplete="off"
                onChange={(event) => setPin(event.target.value)}
                required
              />
            </label>

            <button className="is-primary" disabled={sending || !from || short || amount <= 0}>
              {sending ? 'Pagando…' : `Pagar ${formatMoney(amount)}`}
            </button>

            {short && source && (
              <small className="error">Solo tienes {formatMoney(source.cash)} en esa cuenta.</small>
            )}
            {failed && <small className="error">{failed}</small>}
          </>
        )}
      </form>
    </div>
  );
}
