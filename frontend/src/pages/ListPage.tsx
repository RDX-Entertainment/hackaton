import { tr, useIdioma } from '../i18n/text';
import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { Icono } from '../components/iconos/Icono';
import { StatusMessage } from '../components/StatusMessage';
import { listsApi, type ListEntry } from '../utils/api';
import { formatIp, parseIp } from '../utils/ip';
import { useStatusMessage } from '../utils/useStatusMessage';

export function ListPage() {
  useIdioma();
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
        status.report(tr("«{0}» no es una dirección válida.", ip));
        return;
      }

      await listsApi.addCompromised(packed, login, password);
      setIp('');
      setLogin('');
      setPassword('');
      status.report(tr("Account added."));
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  const remove = async (id: number) => {
    try {
      await listsApi.removeCompromised(id);
      status.report(tr("Account removed."));
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>{tr("Cuentas que tienes")}</h2>

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
          {tr("Usuario")} <input value={login} onChange={(event) => setLogin(event.target.value)} required />
        </label>

        <label>
          {tr("Contraseña")} <input
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            required
          />
        </label>

        <button>{tr("Apuntar cuenta")}</button>
        <StatusMessage text={status.message} />
      </form>

      {entries.length ? (
        <div className="rejilla-tarjetas">
          {entries.map((entry) => (
            <article className="tarjeta" key={entry.id}>
              <span className="tarjeta__rotulo">
                <Icono nombre="llave" tamano={12} /> {formatIp(entry.ip)}
              </span>

              <div className="tarjeta__cabecera" style={{ marginTop: '0.35rem' }}>
                <span className="tarjeta__titulo">{entry.login}</span>
              </div>

              <p className="tarjeta__texto">
                {entry.virus_id ? tr("La dejó tu virus #{0}. ", entry.virus_id) : ''}
                {entry.hacked_at}
              </p>

              <div className="tarjeta__acciones">
                <span className="micro">{tr("cuenta ajena")}</span>
                <button onClick={() => void remove(entry.id)}>{tr("Olvidar")}</button>
              </div>
            </article>
          ))}
        </div>
      ) : (
        <div className="vacio">
          <div className="vacio__titulo">{tr("No has apuntado ninguna cuenta")}</div>
          <p className="vacio__texto">
            {tr("Aquí se guardan los usuarios y contraseñas de máquinas ajenas: los que consigas entrando, y los que te traiga un virus plantado.")} </p>
        </div>
      )}
    </>
  );
}
