import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { StatusMessage } from '../components/StatusMessage';
import { listsApi, type ListEntry } from '../utils/api';
import { formatIp, parseIp } from '../utils/ip';
import { useStatusMessage } from '../utils/useStatusMessage';

export function ListPage() {
  const status = useStatusMessage();

  const [entries, setEntries] = useState<ListEntry[]>([]);
  const [ip, setIp] = useState('');
  const [login, setLogin] = useState('');
  const [password, setPassword] = useState('');

  const load = useCallback(() => {
    listsApi.listCompromised().then(setEntries).catch(() => setEntries([]));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const add = async (event: FormEvent) => {
    event.preventDefault();

    try {
      const packed = parseIp(ip);
      if (packed === null) {
        status.report(`«${ip}» no es una dirección válida.`);
        return;
      }

      await listsApi.addCompromised(packed, login, password);
      setIp('');
      setLogin('');
      setPassword('');
      status.report('Account added.');
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  const remove = async (id: number) => {
    try {
      await listsApi.removeCompromised(id);
      status.report('Account removed.');
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Compromised list</h2>

      <form className="compose" onSubmit={add}>
        <label>
          IP
          <input
            value={ip}
            inputMode="numeric"
            placeholder="0.0.0.0"
            onChange={(event) => setIp(event.target.value)}
            required
          />
        </label>

        <label>
          Login
          <input value={login} onChange={(event) => setLogin(event.target.value)} required />
        </label>

        <label>
          Password
          <input
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            required
          />
        </label>

        <button>Add account</button>
        <StatusMessage text={status.message} />
      </form>

      {entries.map((entry) => (
        <article className="forum-post" key={entry.id}>
          <b>
            {formatIp(entry.ip)} · {entry.login}
          </b>
          <p>
            Virus #{entry.virus_id} · {entry.hacked_at}
          </p>
          <button onClick={() => void remove(entry.id)}>Delete</button>
        </article>
      ))}
    </>
  );
}
