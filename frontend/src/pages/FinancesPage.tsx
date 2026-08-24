import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { DataTable } from '../components/DataTable';
import { StatusMessage } from '../components/StatusMessage';
import { TransferFunds } from '../components/TransferFunds';
import { financesApi, type UnknownRow } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

const PIN_LENGTH = 6;

export function FinancesPage() {
  const status = useStatusMessage();

  const [accounts, setAccounts] = useState<UnknownRow[]>([]);
  const [bankId, setBankId] = useState('1');
  const [pin, setPin] = useState('');

  const load = useCallback(() => {
    financesApi.listAccounts().then(setAccounts).catch(() => setAccounts([]));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const openAccount = async (event: FormEvent) => {
    event.preventDefault();

    try {
      const opened = await financesApi.openAccount(Number(bankId), pin);
      setPin('');
      status.report(`Account ${opened.account} opened.`);
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Finances</h2>

      <form className="compose" onSubmit={openAccount}>
        <label>
          Bank ID
          <input
            value={bankId}
            inputMode="numeric"
            onChange={(event) => setBankId(event.target.value)}
            required
          />
        </label>

        <label>
          New {PIN_LENGTH}-character PIN
          <input
            value={pin}
            minLength={PIN_LENGTH}
            maxLength={PIN_LENGTH}
            onChange={(event) => setPin(event.target.value)}
            required
          />
        </label>

        <button>Open account</button>
        <StatusMessage text={status.message} />
      </form>

      <TransferFunds onSent={load} />
      <DataTable rows={accounts} />
    </>
  );
}
