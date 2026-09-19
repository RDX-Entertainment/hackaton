import { tr, useIdioma } from '../i18n/text';
import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { Icono } from '../components/iconos/Icono';
import { TransferFunds } from '../components/TransferFunds';
import { financesApi, type BankAccount } from '../utils/api';
import { messageFrom } from '../utils/errors';
import { formatMoney } from '../utils/format';

const PIN_LENGTH = 6;

export function FinancesPage() {
  useIdioma();
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
        <h2>{tr("Banco")}</h2>
        <p className="muted">{tr("Cargando tus cuentas…")}</p>
      </>
    );
  }

  const pending = accounts.filter((account) => !account.pin_set);
  const total = accounts
    .filter((account) => account.pin_set)
    .reduce((sum, account) => sum + account.cash, 0);

  return (
    <>
      <h2>{tr("Banco")}</h2>

      {failed && <p className="error">{failed}</p>}

      {pending.map((account) => (
        <ChoosePin key={account.id} account={account} onDone={load} />
      ))}

      {accounts.length > 1 && (
        <article className="tarjeta tarjeta--viva" style={{ marginBottom: '1rem' }}>
          <span className="tarjeta__rotulo">
            <Icono nombre="dinero" tamano={12} /> {tr("entre todas las cuentas con PIN")} </span>
          <b className="tarjeta__cifra">{formatMoney(total)}</b>
          <div className="tarjeta__regla" />
          <p className="tarjeta__pie">
            {accounts.length} {tr("cuentas &middot; el saldo de una cuenta sin PIN no cuenta.")} </p>
        </article>
      )}

      <div className="rejilla-tarjetas">
        {accounts.map((account) => (
          <article
            className={account.pin_set ? 'tarjeta' : 'tarjeta tarjeta--bloqueada'}
            key={account.id}
          >
            <div className="tarjeta__cabecera">
              <span className="tarjeta__titulo">{account.bank_name}</span>
              <span className="micro">nº {account.account}</span>
            </div>

            <span className="tarjeta__rotulo" style={{ marginTop: '0.75rem' }}>
              <Icono nombre="dinero" tamano={12} /> {tr("saldo")} </span>
            <b className={account.pin_set ? 'tarjeta__cifra' : 'tarjeta__cifra tarjeta__cifra--quieta'}>
              {account.pin_set ? formatMoney(account.cash) : '—'}
            </b>
            <div className="tarjeta__regla" />

            {!account.pin_set && (
              <p className="tarjeta__pie">{tr("Sin PIN todavía: no puede mover dinero.")}</p>
            )}
          </article>
        ))}
      </div>

      {!accounts.length && (
        <p className="muted">
          {tr("No tienes ninguna cuenta. Se abre una sola al registrarte; si no la tienes, es que todavía no había ningún banco en el juego. Pídesela a un administrador.")} </p>
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
  useIdioma();
  const [pin, setPin] = useState('');
  const [repeat, setRepeat] = useState('');
  const [failed, setFailed] = useState('');
  const [saving, setSaving] = useState(false);

  const short = pin.length > 0 && pin.length < PIN_LENGTH;
  const mismatch = repeat.length > 0 && pin !== repeat;
  const ready = pin.length === PIN_LENGTH && pin === repeat && !saving;

  const aviso = failed || (mismatch ? tr("Los dos PIN no coinciden.") : '') ||
    (short ? tr("Faltan {0} caracteres.", PIN_LENGTH - pin.length) : '');

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
        <b>{tr("Elige el PIN de tu cuenta")}</b>
        <small>
          {account.account} · {account.bank_name} {tr("— se abrió contigo al registrarte. Sin PIN no puede mover dinero. Son")} {PIN_LENGTH} {tr("letras o números.")} </small>
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
        {tr("Repítelo")} <input
          type="password"
          value={repeat}
          maxLength={PIN_LENGTH}
          autoComplete="new-password"
          onChange={(event) => setRepeat(event.target.value)}
          required
        />
      </label>

      <button className="is-primary" disabled={!ready}>
        {saving ? tr("Guardando…") : tr("Guardar PIN")}
      </button>

      {aviso && <small className="pin-setup__error">{aviso}</small>}
    </form>
  );
}
