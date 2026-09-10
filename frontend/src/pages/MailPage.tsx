import { useCallback, useEffect, useState } from 'react';

import { ComposeMail } from '../components/ComposeMail';
import { Icono } from '../components/iconos/Icono';
import { StatusMessage } from '../components/StatusMessage';
import { Tag } from '../components/Tag';
import { mailApi, type UnknownRow } from '../utils/api';
import { formatCell } from '../utils/format';
import { useStatusMessage } from '../utils/useStatusMessage';

/**
 * El correo.
 *
 * Cada mensaje era un botón con el asunto y, detrás, «from adm · unread» en
 * inglés. Ahora es una tarjeta: quién escribe de rótulo, el asunto de titular,
 * y lo que no has leído se marca —con etiqueta y con el filo vivo—, que es lo
 * único que hace falta saber de un vistazo en una bandeja.
 */
export function MailPage() {
  const status = useStatusMessage();

  const [mensajes, setMensajes] = useState<UnknownRow[]>([]);
  const [abierto, setAbierto] = useState<UnknownRow>();

  const cargar = useCallback(() => {
    mailApi.listMail().then(setMensajes).catch(() => setMensajes([]));
  }, []);

  useEffect(() => {
    cargar();
  }, [cargar]);

  const abrir = async (id: number) => {
    try {
      setAbierto(await mailApi.readMail(id));
      // Leer apaga la marca de no leído, así que la lista de detrás ya no vale.
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

  const sinLeer = mensajes.filter((mensaje) => !mensaje.read).length;

  return (
    <>
      <h2>Correo</h2>
      <StatusMessage text={status.message} as="block" />

      {abierto && (
        <article className="tarjeta tarjeta--viva" style={{ marginBottom: '1.5rem' }}>
          <span className="tarjeta__rotulo">
            <Icono nombre="aviso" tamano={12} /> de {formatCell(abierto.from)}
          </span>
          <div className="tarjeta__cabecera" style={{ marginTop: '0.5rem' }}>
            <span className="tarjeta__titulo">{formatCell(abierto.subject)}</span>
          </div>
          <pre className="tarjeta__texto" style={{ whiteSpace: 'pre-wrap' }}>
            {formatCell(abierto.text)}
          </pre>
          <div className="tarjeta__acciones">
            <span className="micro">abierto</span>
            <button onClick={() => void borrar(Number(abierto.id))}>Borrar</button>
          </div>
        </article>
      )}

      <ComposeMail onSent={cargar} />

      {mensajes.length ? (
        <>
          <h3>
            Bandeja{sinLeer ? ` · ${sinLeer} sin leer` : ''}
          </h3>
          <div className="rejilla-tarjetas">
            {mensajes.map((mensaje) => {
              const nuevo = !mensaje.read;

              return (
                <article
                  className={nuevo ? 'tarjeta tarjeta--viva' : 'tarjeta'}
                  key={formatCell(mensaje.id)}
                >
                  <span className="tarjeta__rotulo">de {formatCell(mensaje.from)}</span>
                  <div className="tarjeta__cabecera" style={{ marginTop: '0.35rem' }}>
                    <span className="tarjeta__titulo">{formatCell(mensaje.subject)}</span>
                  </div>

                  <div className="tarjeta__acciones">
                    {nuevo ? <Tag estado="memoria">sin leer</Tag> : <Tag estado="off">leído</Tag>}
                    <button onClick={() => void abrir(Number(mensaje.id))}>Abrir</button>
                  </div>
                </article>
              );
            })}
          </div>
        </>
      ) : (
        <div className="vacio">
          <div className="vacio__titulo">La bandeja está vacía</div>
          <p className="vacio__texto">
            Aquí llegan los avisos del sistema y lo que te escriban otros jugadores.
          </p>
        </div>
      )}
    </>
  );
}
