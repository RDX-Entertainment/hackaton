import { tr, useIdioma } from '../i18n/text';
import { useCallback, useEffect, useState } from 'react';

import { Icono } from '../components/iconos/Icono';
import { StatusMessage } from '../components/StatusMessage';
import { Tag } from '../components/Tag';
import { missionsApi, type MissionCommand, type UnknownRow } from '../utils/api';
import { formatMoney } from '../utils/format';
import { useStatusMessage } from '../utils/useStatusMessage';

/** Los códigos del legado: 1 es ofrecido, 2 es aceptado. */
const OFRECIDO = 1;
const ACEPTADO = 2;

/**
 * Los encargos.
 *
 * Cada uno era un párrafo con «Prize: $2500 · status 1» dentro: el número que
 * decide si lo coges —el premio— iba escondido en mitad de una frase, y el
 * estado era un dígito sin traducir. Ahora el premio es la cifra grande, el
 * estado usa el vocabulario del sistema y cada encargo lleva una sola acción:
 * la que toca.
 */
export function MissionsPage() {
  useIdioma();
  const status = useStatusMessage();
  const [encargos, setEncargos] = useState<UnknownRow[]>([]);

  const cargar = useCallback(() => {
    missionsApi.listMissions().then(setEncargos).catch(() => setEncargos([]));
  }, []);

  useEffect(() => {
    cargar();
  }, [cargar]);

  const ordenar = async (id: number, accion: MissionCommand) => {
    try {
      await missionsApi.commandMission(id, accion);
      status.report(accion === 'accept' ? tr("Encargo aceptado.") : tr("Encargo abandonado."));
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>{tr("Encargos")}</h2>
      <StatusMessage text={status.message} as="block" />

      {encargos.length ? (
        <div className="rejilla-tarjetas">
          {encargos.map((encargo) => {
            const id = Number(encargo.id);
            const estado = Number(encargo.status);
            const premio = Number(encargo.prize);
            const aceptado = estado === ACEPTADO;

            return (
              <article className={aceptado ? 'tarjeta tarjeta--viva' : 'tarjeta'} key={id}>
                <div className="tarjeta__cabecera">
                  <span className="tarjeta__titulo">{tr("Encargo #")}{id}</span>
                  <span className="micro">{tr("nivel")} {String(encargo.level ?? '—')}</span>
                </div>

                <span className="tarjeta__rotulo" style={{ marginTop: '0.75rem' }}>
                  <Icono nombre="dinero" tamano={12} /> {tr("paga")} </span>
                <b className="tarjeta__cifra">{formatMoney(premio)}</b>
                <div className="tarjeta__regla" />

                <div className="tarjeta__acciones">
                  {aceptado ? (
                    <Tag estado="obra">{tr("aceptado")}</Tag>
                  ) : estado === OFRECIDO ? (
                    <Tag estado="plano">{tr("libre")}</Tag>
                  ) : (
                    <Tag estado="off">{tr("cerrado")}</Tag>
                  )}

                  {estado === OFRECIDO && (
                    <button onClick={() => void ordenar(id, 'accept')}>{tr("Aceptar")}</button>
                  )}
                  {aceptado && (
                    <button onClick={() => void ordenar(id, 'abort')}>{tr("Abandonar")}</button>
                  )}
                </div>
              </article>
            );
          })}
        </div>
      ) : (
        <div className="vacio">
          <div className="vacio__titulo">{tr("No hay encargos en el tablón")}</div>
          <p className="vacio__texto">
            {tr("El tablón se repone solo cada cierto tiempo. Mientras tanto, el dinero también entra por la Academia y por lo que consigas por tu cuenta.")} </p>
        </div>
      )}
    </>
  );
}
