import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { Icono } from '../components/iconos/Icono';
import { TransferFunds } from '../components/TransferFunds';
import { financesApi, type BankAccount } from '../utils/api';
import { messageFrom } from '../utils/errors';
import { formatMoney } from '../utils/format';

const PIN_LENGTH = 6;

export function FinancesPage() {
  const [accounts, setAccounts] = useState<BankAccount[]>();
  const [failed, setFailed] = useState('');

  const load = useCallback(() => {
    financesApi
      .listAccounts()
      .then((rows) => {
        setAccounts(rows);
        setFailed('');
      })
      .catch((error) => {
        setAccounts([]);
        setFailed(messageFrom(error));
      });
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  if (!accounts) {
    return (
      <>
        <h2>Dinero</h2>
        <p className="muted">Cargando tus cuentas…</p>
      </>
    );
  }

  const pending = accounts.filter((account) => !account.pin_set);
  const total = accounts
    .filter((account) => account.pin_set)
    .reduce((sum, account) => sum + account.cash, 0);

  return (
    <>
      <h2>Dinero</h2>

      {failed && <p className="error">{failed}</p>}

      {pending.map((account) => (
        <ChoosePin key={account.id} account={account} onDone={load} />
      ))}

      <div className="rejilla-tarjetas">
        {accounts.map((account) => (
          <article
            className={account.pin_set ? 'tarjeta' : 'tarjeta tarjeta--bloqueada'}
            key={account.id}
          >
            <div className="tarjeta__cabecera">
              <span className="tarjeta__titulo">{account.bank_name}</span>
              <span className="micro">{account.account}</span>
            </div>

            <span className="tarjeta__rotulo" style={{ marginTop: '0.75rem' }}>
              <Icono nombre="dinero" tamano={12} /> saldo
            </span>
            <b className={account.pin_set ? 'tarjeta__cifra' : 'tarjeta__cifra tarjeta__cifra--quieta'}>
              {account.pin_set ? formatMoney(account.cash) : '—'}
            </b>
            <div className="tarjeta__regla" />

            {!account.pin_set && (
              <p className="tarjeta__pie">Sin PIN todavía: no puede mover dinero.</p>
            )}
          </article>
        ))}
      </div>

      {!accounts.length && (
        <p className="muted">
          No tienes ninguna cuenta. Se abre una sola al registrarte; si no la tienes, es que
          todavía no había ningún banco en el juego. Pídesela a un administrador.
        </p>
      )}

      {accounts.length > 1 && (
        <p className="muted">
          Entre todas las cuentas con PIN: <b className="dato">{formatMoney(total)}</b>
        </p>
      )}

      <TransferFunds accounts={accounts} onSent={load} />
    </>
  );
}

/**
 * La cuenta con la que nace el jugador no trae PIN: lo elige aquí, la primera vez. Hasta que
 * lo haga la cuenta existe y no se puede usar, así que esto va arriba del todo.
 *
 * El aviso de error vive dentro del formulario y no en la barra de estado de la página: el
 * fallo ocurre aquí y es aquí donde se está mirando.
 */
function ChoosePin({ account, onDone }: { account: BankAccount; onDone: () => void }) {
  const [pin, setPin] = useState('');
  const [repeat, setRepeat] = useState('');
  const [failed, setFailed] = useState('');
  const [saving, setSaving] = useState(false);

  const short = pin.length > 0 && pin.length < PIN_LENGTH;
  const mismatch = repeat.length > 0 && pin !== repeat;
  const ready = pin.length === PIN_LENGTH && pin === repeat && !saving;

  const aviso = failed || (mismatch ? 'Los dos PIN no coinciden.' : '') ||
    (short ? `Faltan ${PIN_LENGTH - pin.length} caracteres.` : '');

  const save = async (event: FormEvent) => {
    event.preventDefault();
    setSaving(true);
    setFailed('');

    try {
      await financesApi.setAccountPin(account.id, pin);
      setPin('');
      setRepeat('');
      onDone();
    } catch (error) {
      setFailed(messageFrom(error));
      // Si el desacuerdo era de la pantalla —el PIN ya estaba puesto— recargar lo corrige.
      onDone();
    } finally {
      setSaving(false);
    }
  };

  return (
    <form className="pin-setup" onSubmit={save}>
      <div className="pin-setup__lead">
        <b>Elige el PIN de tu cuenta</b>
        <small>
          {account.account} · {account.bank_name} — se abrió contigo al registrarte. Sin PIN no
          puede mover dinero. Son {PIN_LENGTH} letras o números.
        </small>
      </div>

      <label>
        PIN
        <input
          type="password"
          value={pin}
          maxLength={PIN_LENGTH}
          autoComplete="new-password"
          onChange={(event) => setPin(event.target.value)}
          required
        />
      </label>

      <label>
        Repítelo
        <input
          type="password"
          value={repeat}
          maxLength={PIN_LENGTH}
          autoComplete="new-password"
          onChange={(event) => setRepeat(event.target.value)}
          required
        />
      </label>

      <button className="is-primary" disabled={!ready}>
        {saving ? 'Guardando…' : 'Guardar PIN'}
      </button>

      {aviso && <small className="pin-setup__error">{aviso}</small>}
    </form>
  );
}
