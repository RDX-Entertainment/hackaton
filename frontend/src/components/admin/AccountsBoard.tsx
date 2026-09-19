import { tr, useIdioma } from '../../i18n/text';
import { useEffect, useState, type FormEvent } from 'react';

import { financesApi, type Bank, type BankAccount } from '../../utils/api';
import { formatMoney } from '../../utils/format';

type Props = {
  accounts: BankAccount[];
  busy: boolean;
  emptyMessage: string;
  onOpen: (bankId: number, pin: string) => void;
  onAdjust: (account: BankAccount, amount: number) => void;
};

const PIN_LENGTH = 6;

/**
 * Las cuentas de un titular y lo que el panel puede hacer con ellas: abrir una nueva,
 * ingresar y retirar. Lo usan la pantalla de usuarios y la de organizaciones, que
 * administran lo mismo y solo se diferencian en de quién es el dinero.
 *
 * El importe se teclea una vez y los dos botones deciden el signo: son el mismo gesto.
 */
export function AccountsBoard({ accounts, busy, emptyMessage, onOpen, onAdjust }: Props) {
  useIdioma();
  const [banks, setBanks] = useState<Bank[]>([]);
  const [bankId, setBankId] = useState(0);
  const [pin, setPin] = useState('');
  const [amounts, setAmounts] = useState<Record<number, string>>({});

  useEffect(() => {
    financesApi.listBanks().then(setBanks).catch(() => setBanks([]));
  }, []);

  useEffect(() => {
    setBankId((current) => (banks.some((bank) => bank.id === current) ? current : banks[0]?.id ?? 0));
  }, [banks]);

  const open = (event: FormEvent) => {
    event.preventDefault();
    onOpen(bankId, pin);
    setPin('');
  };

  const move = (account: BankAccount, sign: 1 | -1) => {
    const typed = Number(amounts[account.id] ?? '');

    if (!Number.isFinite(typed) || typed <= 0) {
      return;
    }

    onAdjust(account, Math.trunc(typed) * sign);
    setAmounts((current) => ({ ...current, [account.id]: '' }));
  };

  return (
    <div className="accounts">
      {accounts.map((account) => {
        const typed = Number(amounts[account.id] ?? '');
        const ready = Number.isFinite(typed) && typed > 0;

        return (
          <div className="account" key={account.id}>
            <b>{account.account}</b>
            <span>{account.bank_name}</span>
            <em>{formatMoney(account.cash)}</em>
            <input
              value={amounts[account.id] ?? ''}
              inputMode="numeric"
              placeholder="importe"
              onChange={(event) => setAmounts((current) => ({ ...current, [account.id]: event.target.value }))}
            />
            <button type="button" disabled={busy || !ready} onClick={() => move(account, 1)}>
              {tr("Ingresar")} </button>
            <button
              type="button"
              className="account__take"
              disabled={busy || !ready || typed > account.cash}
              onClick={() => move(account, -1)}
            >
              {tr("Retirar")} </button>
          </div>
        );
      })}

      {!accounts.length && <p className="org-empty">{emptyMessage}</p>}

      <form className="account-create" onSubmit={open}>
        <b>{tr("Abrir cuenta")}</b>
        <select value={bankId} disabled={!banks.length} onChange={(event) => setBankId(Number(event.target.value))}>
          {banks.length ? (
            banks.map((bank) => (
              <option key={bank.id} value={bank.id}>
                {bank.name}
              </option>
            ))
          ) : (
            <option value={0}>{tr("Sin bancos")}</option>
          )}
        </select>
        <input
          required
          value={pin}
          minLength={PIN_LENGTH}
          maxLength={PIN_LENGTH}
          placeholder={tr("PIN de {0}", PIN_LENGTH)}
          onChange={(event) => setPin(event.target.value)}
        />
        <button className="is-primary" disabled={busy || !bankId || pin.length !== PIN_LENGTH}>
          {tr("Abrir")} </button>
        {!banks.length && <small>{tr("Marca «Ofrece cuentas bancarias» en alguna organización para tener dónde abrirla.")}</small>}
      </form>
    </div>
  );
}
