import { tr, useIdioma } from '../i18n/text';
import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { Icono } from '../components/iconos/Icono';
import { StatusMessage } from '../components/StatusMessage';
import { bitcoinApi, type Wallet } from '../utils/api';
import { formatBitcoin, parseBitcoin } from '../utils/format';
import { useStatusMessage } from '../utils/useStatusMessage';

/**
 * Los monederos.
 *
 * El saldo vivía en una tabla de dos columnas, que es la forma de enseñar algo
 * de lo que hay cien; aquí hay dos o tres, y cada uno es un objeto con un
 * número que importa. La dirección va de rótulo y el saldo de cifra.
 *
 * Y la pantalla estaba a medias en inglés, que era justo lo que la decisión de
 * idioma quería evitar.
 */
export function BitcoinPage() {
  useIdioma();
  const status = useStatusMessage();

  const [monederos, setMonederos] = useState<Wallet[]>([]);
  const [desde, setDesde] = useState('');
  const [hacia, setHacia] = useState('');
  const [cantidad, setCantidad] = useState('');

  const cargar = useCallback(() => {
    bitcoinApi.listWallets().then(setMonederos).catch(() => setMonederos([]));
  }, []);

  useEffect(() => {
    cargar();
  }, [cargar]);

  const crear = async () => {
    try {
      await bitcoinApi.createWallet();
      status.report(tr("Monedero creado."));
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  const transferir = async (event: FormEvent) => {
    event.preventDefault();

    const satoshis = parseBitcoin(cantidad);

    if (satoshis === null || satoshis <= 0) {
      status.report(tr("Cantidad no válida. Hasta siete decimales, por ejemplo 1,0000001."));
      return;
    }

    try {
      await bitcoinApi.transferBitcoin(desde, hacia, satoshis);
      setCantidad('');
      status.report(tr("Transferencia completada."));
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Bitcoin</h2>

      {monederos.length ? (
        <div className="rejilla-tarjetas">
          {monederos.map((monedero) => (
            <article className="tarjeta" key={monedero.address}>
              <span className="tarjeta__rotulo">
                <Icono nombre="llave" tamano={12} /> {tr("monedero")} </span>
              <p className="tarjeta__texto" style={{ marginTop: '0.25rem' }}>
                {monedero.address}
              </p>

              <b className="tarjeta__cifra">{formatBitcoin(monedero.amount)}</b>
              <div className="tarjeta__regla" />
              <p className="tarjeta__pie">{tr("BTC disponibles")}</p>
            </article>
          ))}
        </div>
      ) : (
        <div className="vacio">
          <div className="vacio__titulo">{tr("Todavía no tienes monederos")}</div>
          <p className="vacio__texto">
            {tr("Un monedero no es una cuenta de banco: nadie sabe de quién es. Por eso lo que se roba acaba pasando por aquí.")} </p>
          <button onClick={() => void crear()}>{tr("Crear el primero")}</button>
        </div>
      )}

      {monederos.length > 0 && <button onClick={() => void crear()}>{tr("Crear otro monedero")}</button>}

      <h3>{tr("Transferir")}</h3>

      <form className="compose" onSubmit={transferir}>
        <label>
          {tr("Desde")} <select value={desde} onChange={(event) => setDesde(event.target.value)} required>
            <option value="">{tr("Elige un monedero")}</option>
            {monederos.map((monedero) => (
              <option key={monedero.address} value={monedero.address}>
                {monedero.address}
              </option>
            ))}
          </select>
        </label>

        <label>
          {tr("Hacia")} <input value={hacia} onChange={(event) => setHacia(event.target.value)} required />
        </label>

        <label>
          {tr("Cantidad")} <input
            value={cantidad}
            inputMode="decimal"
            onChange={(event) => setCantidad(event.target.value)}
            required
          />
        </label>

        <button>{tr("Transferir")}</button>
      </form>

      <StatusMessage text={status.message} as="block" />
    </>
  );
}
