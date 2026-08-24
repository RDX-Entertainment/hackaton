import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { DataTable } from '../components/DataTable';
import { StatusMessage } from '../components/StatusMessage';
import { bitcoinApi, type Wallet } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

export function BitcoinPage() {
  const status = useStatusMessage();

  const [wallets, setWallets] = useState<Wallet[]>([]);
  const [from, setFrom] = useState('');
  const [to, setTo] = useState('');
  const [amount, setAmount] = useState('');

  const load = useCallback(() => {
    bitcoinApi.listWallets().then(setWallets).catch(() => setWallets([]));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const create = async () => {
    try {
      await bitcoinApi.createWallet();
      status.report('Wallet created.');
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  const transfer = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await bitcoinApi.transferBitcoin(from, to, amount);
      setAmount('');
      status.report('Bitcoin transfer complete.');
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Bitcoin</h2>

      <button onClick={() => void create()}>Create wallet</button>

      <form className="compose" onSubmit={transfer}>
        <label>
          From wallet
          <select value={from} onChange={(event) => setFrom(event.target.value)} required>
            <option value="">Select wallet</option>
            {wallets.map((wallet) => (
              <option key={wallet.address} value={wallet.address}>
                {wallet.address}
              </option>
            ))}
          </select>
        </label>

        <label>
          To wallet
          <input value={to} onChange={(event) => setTo(event.target.value)} required />
        </label>

        <label>
          Amount
          <input
            value={amount}
            inputMode="decimal"
            onChange={(event) => setAmount(event.target.value)}
            required
          />
        </label>

        <button>Transfer</button>
      </form>

      <StatusMessage text={status.message} as="block" />
      <DataTable rows={wallets} />
    </>
  );
}
