import { useCallback, useEffect, useMemo, useState } from 'react';

import { ComposeMail } from '../components/ComposeMail';
import { Icono } from '../components/iconos/Icono';
import { StatusMessage } from '../components/StatusMessage';
import { mailApi, rankingApi, type RankedPlayer, type UnknownRow } from '../utils/api';
import { formatCell } from '../utils/format';
import { useStatusMessage } from '../utils/useStatusMessage';

/**
 * El correo, y **parece un correo**.
 *
 * Era una rejilla de tarjetas, que es la forma de enseñar cosas que se comparan
 * entre sí —encargos, cuentas, programas—. Un buzón no se compara: se recorre.
 * Por eso esto es lo que es en cualquier cliente desde hace treinta años: la
 * lista a un lado, el mensaje abierto al otro, y el cuerpo tal cual lo
 * escribieron.
 *
 * El remitente llega como número de jugador, que a la cara de nadie significa
 * nada, así que se cruza con el ranking para enseñar su nombre. Si no está en
 * la lista —un aviso del sistema— se queda con su `#id`, que sigue siendo mejor
 * que nada.
 */
export function MailPage() {
  const status = useStatusMessage();

  const [mensajes, setMensajes] = useState<UnknownRow[]>([]);
  const [jugadores, setJugadores] = useState<RankedPlayer[]>([]);
  const [abierto, setAbierto] = useState<UnknownRow>();
  const [redactando, setRedactando] = useState(false);
  const [respuestaA, setRespuestaA] = useState({ para: '', asunto: '' });

  const cargar = useCallback(() => {
    mailApi.listMail().then(setMensajes).catch(() => setMensajes([]));
  }, []);

  useEffect(() => {
    cargar();
    rankingApi.listPlayers().then(setJugadores).catch(() => setJugadores([]));
  }, [cargar]);

  const nombreDe = useMemo(() => {
    const porId = new Map(jugadores.map((jugador) => [jugador.user_id, jugador.login]));
    return (id: unknown) => porId.get(Number(id)) ?? `#${formatCell(id)}`;
  }, [jugadores]);

  const abrir = async (id: number) => {
    setRedactando(false);

    try {
      setAbierto(await mailApi.readMail(id));
      // Leer apaga la marca, así que la lista de detrás ya no vale.
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  const borrar = async (id: number) => {
    try {
      await mailApi.removeMail(id);
      setAbierto(undefined);
      status.report('Mensaje borrado.');
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  const responder = () => {
    if (!abierto) {
      return;
    }

    const asunto = formatCell(abierto.subject);
    setRespuestaA({
      para: nombreDe(abierto.from),
      asunto: asunto.startsWith('Re:') ? asunto : `Re: ${asunto}`,
    });
    setRedactando(true);
  };

  const escribir = () => {
    setRespuestaA({ para: '', asunto: '' });
    setRedactando(true);
  };

  const sinLeer = mensajes.filter((mensaje) => !mensaje.read).length;

  return (
    <>
      <h2>Correo</h2>
      <StatusMessage text={status.message} as="block" />

      <div className={`correo${abierto || redactando ? ' correo--leyendo' : ''}`}>
        <div className="correo__lista">
          <div className="correo__barra">
            <span className="micro">
              bandeja{sinLeer ? ` · ${sinLeer} sin leer` : ''}
            </span>
            <button onClick={escribir}>Escribir</button>
          </div>

          {mensajes.length ? (
            <ul className="correo__mensajes">
              {mensajes.map((mensaje) => {
                const id = Number(mensaje.id);
                const nuevo = !mensaje.read;
                const elegido = Number(abierto?.id) === id;

                return (
                  <li key={id}>
                    <button
                      type="button"
                      className={`correo__fila${nuevo ? ' es-nuevo' : ''}${
                        elegido ? ' es-elegido' : ''
                      }`}
                      onClick={() => void abrir(id)}
                    >
                      <span className="correo__punto" aria-hidden="true" />
                      <span className="correo__de">{nombreDe(mensaje.from)}</span>
                      <span className="correo__asunto">{formatCell(mensaje.subject)}</span>
                      <span className="correo__avance">
                        {formatCell(mensaje.text).split('\n')[0]}
                      </span>
                    </button>
                  </li>
                );
              })}
            </ul>
          ) : (
            <p className="correo__nada">La bandeja está vacía.</p>
          )}
        </div>

        <div className="correo__panel">
          {redactando ? (
            <>
              <div className="correo__cabecera">
                <span className="micro">mensaje nuevo</span>
              </div>
              <ComposeMail
                onSent={cargar}
                onCerrar={() => setRedactando(false)}
                para={respuestaA.para}
                asunto={respuestaA.asunto}
              />
            </>
          ) : abierto ? (
            <>
              <div className="correo__cabecera">
                <h3 className="correo__titulo">{formatCell(abierto.subject)}</h3>
                <div className="correo__meta">
                  <span>
                    <span className="micro">de</span> {nombreDe(abierto.from)}
                  </span>
                  <span>
                    <span className="micro">para</span> {nombreDe(abierto.to)}
                  </span>
                </div>
              </div>

              <pre className="correo__cuerpo">{formatCell(abierto.text)}</pre>

              <div className="correo__acciones">
                <button type="button" onClick={() => setAbierto(undefined)}>
                  Volver
                </button>
                <button type="button" onClick={responder}>
                  Responder
                </button>
                <button type="button" onClick={() => void borrar(Number(abierto.id))}>
                  Borrar
                </button>
              </div>
            </>
          ) : (
            <div className="correo__vacio">
              <Icono nombre="aviso" tamano={24} />
              <p>Elige un mensaje de la izquierda.</p>
              <span className="micro">
                aquí llegan los avisos del sistema y lo que te escriban otros jugadores
              </span>
            </div>
          )}
        </div>
      </div>
    </>
  );
}
