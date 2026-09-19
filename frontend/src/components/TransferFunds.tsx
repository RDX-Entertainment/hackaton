import { tr, useIdioma } from '../i18n/text';
import { useEffect, useState, type FormEvent } from 'react';

import { financesApi, type BankAccount } from '../utils/api';
import { messageFrom } from '../utils/errors';
import { formatMoney } from '../utils/format';

type TransferFundsProps = {
  /** Las cuentas del jugador: de una de ellas sale el dinero. */
  accounts: BankAccount[];
  /** Se llama tras una transferencia correcta, para recargar los saldos. */
  onSent: () => void;
};

/** Lo que responde el servidor, traducido a algo que se pueda leer sin saber inglés. */
function explain(error: unknown): string {
  const raw = messageFrom(error);

  if (raw.includes('insufficient funds')) return tr("No tienes tanto en esa cuenta.");
  if (raw.includes('invalid transfer')) return tr("Revisa el importe y que las cuentas sean distintas.");
  if (raw.includes('not found')) return tr("Esa cuenta de destino no existe.");
  // El servidor responde 401 «authentication required» cuando la clave no cuadra: aquí, con
  // la sesión ya abierta, eso solo puede ser el PIN de la cuenta.
  if (raw.includes('authentication required')) return tr("El PIN de la cuenta no es correcto.");

  return raw;
}

export function TransferFunds({ accounts, onSent }: TransferFundsProps) {
  useIdioma();
  const [from, setFrom] = useState(0);
  const [password, setPassword] = useState('');
  const [to, setTo] = useState('');
  const [amount, setAmount] = useState('');
  const [failed, setFailed] = useState('');
  const [done, setDone] = useState('');
  const [sending, setSending] = useState(false);

  const usable = accounts.filter((account) => account.pin_set);
  const source = usable.find((account) => account.account === from);
  const wanted = Number(amount);
  const overdraft = source && Number.isFinite(wanted) && wanted > source.cash;

  useEffect(() => {
    setFrom((current) => (usable.some((account) => account.account === current) ? current : usable[0]?.account ?? 0));
    // Comparar por contenido y no por identidad: `usable` se rehace en cada render.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [accounts]);

  const send = async (event: FormEvent) => {
    event.preventDefault();
    setSending(true);
    setFailed('');
    setDone('');

    try {
      await financesApi.transferFunds(from, password, Number(to), wanted);
      setPassword('');
      setAmount('');
      setDone(tr("Enviados {0} a la cuenta {1}.", formatMoney(wanted), to));
      onSent();
    } catch (error) {
      setFailed(explain(error));
    } finally {
      setSending(false);
    }
  };

  if (!usable.length) {
    return null;
  }

  return (
    <form className="compose" onSubmit={send}>
      <h3>{tr("Transferir")}</h3>

      <label>
        {tr("Cuenta de origen")} <select value={from} onChange={(event) => setFrom(Number(event.target.value))} required>
          {usable.map((account) => (
            <option key={account.id} value={account.account}>
              {account.account} · {account.bank_name} · {formatMoney(account.cash)}
            </option>
          ))}
        </select>
      </label>

      <label>
        {tr("PIN de la cuenta")} <input
          type="password"
          value={password}
          maxLength={6}
          autoComplete="off"
          onChange={(event) => setPassword(event.target.value)}
          required
        />
      </label>

      <label>
        {tr("Cuenta de destino")} <input
          inputMode="numeric"
          value={to}
          placeholder={tr("9 dígitos")}
          onChange={(event) => setTo(event.target.value)}
          required
        />
      </label>

      <label>
        {tr("Importe")} <input
          inputMode="numeric"
          min="1"
          value={amount}
          onChange={(event) => setAmount(event.target.value)}
          required
        />
      </label>

      <button disabled={sending || !from || overdraft || wanted <= 0}>
        {sending ? tr("Enviando…") : tr("Transferir")}
      </button>

      {overdraft && source && (
        <small className="error">
          {tr("Solo tienes")} {formatMoney(source.cash)} {tr("en esa cuenta.")} </small>
      )}
      {failed && <small className="error">{failed}</small>}
      {done && <small className="muted">{done}</small>}
    </form>
  );
}
