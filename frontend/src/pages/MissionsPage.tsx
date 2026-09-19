import { tr, useIdioma } from '../i18n/text';
import { useCallback, useEffect, useState } from 'react';
import { Link } from 'react-router-dom';

import { Icono } from '../components/iconos/Icono';
import { StatusMessage } from '../components/StatusMessage';
import { Tag } from '../components/Tag';
import { missionsApi, type Mission, type MissionCommand } from '../utils/api';
import { formatMoney } from '../utils/format';
import { formatIp } from '../utils/ip';
import { useStatusMessage } from '../utils/useStatusMessage';

/** Los códigos del legado: 1 es ofrecido, 2 es aceptado. */
const OFRECIDO = 1;
const ACEPTADO = 2;
const COMPLETADO = 3;

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
  const [encargos, setEncargos] = useState<Mission[]>([]);

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
            const id = encargo.id;
            const estado = encargo.status;
            const premio = encargo.prize;
            const aceptado = estado === ACEPTADO;
            const completado = estado === COMPLETADO;

            return (
              <article className={aceptado ? 'tarjeta tarjeta--viva' : 'tarjeta'} key={id}>
                <div className="tarjeta__cabecera">
                  <span className="tarjeta__titulo">{tr("Encargo #")}{id}</span>
                  <span className="micro">{tr("nivel")} {encargo.level}</span>
                </div>

                <p className="tarjeta__texto">
                  {tr("Consigue acceso a {0}", encargo.target_name)} · {formatIp(encargo.victim)}
                </p>

                <span className="tarjeta__rotulo" style={{ marginTop: '0.75rem' }}>
                  <Icono nombre="dinero" tamano={12} /> {tr("paga")} </span>
                <b className="tarjeta__cifra">{formatMoney(premio)}</b>
                <div className="tarjeta__regla" />

                <div className="tarjeta__acciones">
                  {aceptado ? (
                    <Tag estado="obra">{tr("aceptado")}</Tag>
                  ) : completado ? (
                    <Tag estado="memoria">{tr("completado")}</Tag>
                  ) : estado === OFRECIDO ? (
                    <Tag estado="plano">{tr("libre")}</Tag>
                  ) : (
                    <Tag estado="off">{tr("cerrado")}</Tag>
                  )}

                  {estado === OFRECIDO && (
                    <button onClick={() => void ordenar(id, 'accept')}>{tr("Aceptar")}</button>
                  )}
                  {aceptado && (
                    <>
                      <Link className="boton" to={`/internet?ip=${encargo.victim}`}>{tr("Abrir objetivo")}</Link>
                      <button onClick={() => void ordenar(id, 'abort')}>{tr("Abandonar")}</button>
                    </>
                  )}
                </div>
                {completado && encargo.reward_module_name && (
                  <p className="tarjeta__pie">
                    {tr("Botín: {0} · grado {1}", encargo.reward_module_name, (encargo.reward_module_tier ?? 0) + 1)}
                  </p>
                )}
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
