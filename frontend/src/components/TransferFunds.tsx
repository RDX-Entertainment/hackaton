import { useState, type FormEvent } from 'react';

import { financesApi } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';
import { StatusMessage } from './StatusMessage';

type TransferFundsProps = {
  /** Called after a successful transfer so balances can be reloaded. */
  onSent: () => void;
};

export function TransferFunds({ onSent }: TransferFundsProps) {
  const status = useStatusMessage();

  const [from, setFrom] = useState('');
  const [password, setPassword] = useState('');
  const [to, setTo] = useState('');
  const [amount, setAmount] = useState('');

  const send = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await financesApi.transferFunds(Number(from), password, Number(to), Number(amount));
      setPassword('');
      setAmount('');
      status.report('Transfer complete.');
      onSent();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <form className="compose" onSubmit={send}>
      <label>
        From account
        <input
          inputMode="numeric"
          value={from}
          onChange={(event) => setFrom(event.target.value)}
          required
        />
      </label>

      <label>
        Account password
        <input
          type="password"
          value={password}
          onChange={(event) => setPassword(event.target.value)}
          required
        />
      </label>

      <label>
        Destination account
        <input
          inputMode="numeric"
          value={to}
          onChange={(event) => setTo(event.target.value)}
          required
        />
      </label>

      <label>
        Amount
        <input
          inputMode="numeric"
          min="1"
          value={amount}
          onChange={(event) => setAmount(event.target.value)}
          required
        />
      </label>

      <button>Transfer</button>
      <StatusMessage text={status.message} />
    </form>
  );
}
