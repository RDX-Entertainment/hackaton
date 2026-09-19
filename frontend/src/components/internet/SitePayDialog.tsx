import { tr, useIdioma } from '../../i18n/text';
import { useEffect, useState, type FormEvent } from 'react';

import { bankApi, financesApi, type BankAccount, type SiteAction } from '../../utils/api';
import { messageFrom } from '../../utils/errors';
import { formatMoney } from '../../utils/format';

type SitePayDialogProps = {
  action: SiteAction | null;
  onClose: () => void;
};

/**
 * Lo que sale cuando un sitio cualquiera ofrece cobrar.
 *
 * Va **con el aspecto del juego y no con el de la página**, y eso es a propósito: así se ve
 * de un vistazo que quien pide el PIN es el sistema y no la tienda que estás mirando. Un
 * banco sí puede llevarse esta interfaz dentro de su web y vestirla con sus colores, porque
 * para eso hay que ser un banco; una tienda no.
 *
 * La página solo declara la intención: el importe, la cuenta que recibe y un concepto. Que
 * el concepto sea mentira es asunto de quien lo escribió: eso es el juego.
 */
export function SitePayDialog({ action, onClose }: SitePayDialogProps) {
  useIdioma();
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
  const kind = action.kind === 'charge' ? tr("Cobro") : tr("Pago");
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

      setDone(tr("Pagado. Te quedan {0}.", formatMoney(movement.balance)));
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
          <b>
            <span className="site-pay__seal">{tr("SISTEMA")}</span> {tr("Verificando con el banco")} </b>
          <button type="button" onClick={onClose}>
            {tr("cerrar")} </button>
        </header>

        <p className="site-pay__note">
          <b>{concept}</b> {tr("— lo pide")} {kind === 'Cobro' ? tr("un cobro") : tr("un pago")} {tr("de esta página. Comprueba el importe y la cuenta antes de autorizar: el PIN se lo das al juego, nunca al sitio.")} </p>

        <dl>
          <div>
            <dt>{tr("Importe")}</dt>
            <dd>{formatMoney(amount)}</dd>
          </div>
          <div>
            <dt>{tr("A la cuenta")}</dt>
            <dd>{action.target}</dd>
          </div>
          <div>
            <dt>{tr("Concepto")}</dt>
            <dd>{concept}</dd>
          </div>
        </dl>

        {done ? (
          <p className="site-pay__done">{done}</p>
        ) : (
          <>
            <label>
              {tr("Pagar desde")} <select
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
                  <option value={0}>{tr("Sin cuentas con PIN")}</option>
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
              {sending ? tr("Pagando…") : tr("Pagar {0}", formatMoney(amount))}
            </button>

            {short && source && (
              <small className="error">{tr("Solo tienes")} {formatMoney(source.cash)} {tr("en esa cuenta.")}</small>
            )}
            {failed && <small className="error">{failed}</small>}
          </>
        )}
      </form>
    </div>
  );
}
