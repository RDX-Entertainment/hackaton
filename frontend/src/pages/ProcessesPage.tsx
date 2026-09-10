import { useCallback, useEffect, useState } from 'react';
import { Link } from 'react-router-dom';

import { Icono } from '../components/iconos/Icono';
import { StatusMessage } from '../components/StatusMessage';
import { Tag } from '../components/Tag';
import {
  hardwareApi,
  processesApi,
  type Machine,
  type ProcessCommand,
  type UnknownRow,
} from '../utils/api';
import { formatCell } from '../utils/format';
import { useStatusMessage } from '../utils/useStatusMessage';

/**
 * Los nombres del legado, en castellano y dichos como lo que hacen.
 *
 * La API devuelve `E_LOG`, `D_LOG`, `XHD`: son las constantes del PHP original y
 * no significan nada para quien juega. La que no conozcamos sale con su nombre
 * crudo, que es como se descubre que el worker aprendió a hacer algo nuevo.
 */
const NOMBRES: Record<number, string> = {
  1: 'Descargar un fichero',
  2: 'Subir un fichero',
  3: 'Borrar un fichero',
  4: 'Ocultar un fichero',
  5: 'Buscar lo oculto',
  6: 'Recoger lo del virus',
  7: 'Pasar el antivirus',
  8: 'Falsear el registro',
  9: 'Borrar el registro',
  10: 'Formatear el disco',
  11: 'Entrar en una máquina',
  12: 'Entrar en un banco',
  13: 'Cargar en memoria',
  14: 'Sacar de memoria',
  15: 'Mirar los puertos',
  16: 'Entrar para aprender',
  17: 'Investigar',
  18: 'Subir al disco externo',
  19: 'Bajar del disco externo',
  20: 'Borrar del disco externo',
  22: 'Rastrear la red',
  23: 'Analizar',
  24: 'Instalar el doom',
  25: 'Cambiar de dirección',
  26: 'Cambiar la contraseña',
  27: 'Tumbar a base de peticiones',
  28: 'Levantar el servidor web',
  91: 'Compilar un programa',
};

/** Cada cuánto se vuelve a preguntar. El worker resuelve fuera de la pantalla. */
const REFRESCO_MS = 2_000;

function segundos(total: number) {
  const minutos = Math.floor(total / 60);
  const resto = Math.max(0, total % 60);
  return `${String(minutos).padStart(2, '0')}:${String(resto).padStart(2, '0')}`;
}

/**
 * Las tareas: la cola del worker, mirada desde fuera.
 *
 * Esta pantalla era **un formulario de base de datos**: un desplegable con
 * «HACK (11)», una caja para escribir el id de la víctima y otra de texto libre.
 * Eso no es una pantalla de juego, es el panel de quien escribió el worker: un
 * jugador no tiene a mano el número de una acción ni el id de nadie.
 *
 * Las tareas se lanzan desde donde significan algo --cargar en memoria desde
 * Discos, compilar desde Lab, entrar desde Red-- y aquí se **miran**: cuánto
 * queda, en qué equipo, y cómo acabaron. La cuenta atrás va en ámbar, que en
 * este sistema significa tiempo.
 */
export function ProcessesPage() {
  const status = useStatusMessage();

  const [tareas, setTareas] = useState<UnknownRow[]>([]);
  const [maquinas, setMaquinas] = useState<Machine[]>([]);

  const cargar = useCallback(() => {
    processesApi.listProcesses().then(setTareas).catch(() => setTareas([]));
  }, []);

  useEffect(() => {
    cargar();
    hardwareApi.listMachines().then(setMaquinas).catch(() => setMaquinas([]));

    const reloj = window.setInterval(cargar, REFRESCO_MS);
    return () => window.clearInterval(reloj);
  }, [cargar]);

  const ordenar = async (id: number, orden: ProcessCommand) => {
    const dicho = { pause: 'Pausada.', resume: 'Reanudada.', delete: 'Cancelada.' };

    try {
      await processesApi.commandProcess(id, orden);
      status.report(dicho[orden]);
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  const nombreDe = (fila: UnknownRow) =>
    NOMBRES[Number(fila.action)] ?? formatCell(fila.details || fila.action);

  const equipoDe = (fila: UnknownRow) => {
    const id = Number(fila.origin_server ?? 0);
    return maquinas.find((maquina) => maquina.id === id)?.name;
  };

  const enMarcha = tareas.filter((tarea) => !tarea.completed);
  const terminadas = tareas.filter((tarea) => tarea.completed);

  return (
    <>
      <h2>Tareas</h2>
      <StatusMessage text={status.message} as="block" />

      {enMarcha.length ? (
        <div className="tareas">
          {enMarcha.map((tarea) => {
            const id = Number(tarea.id);
            const total = Number(tarea.ideal_seconds ?? 0);
            const quedan = Math.max(0, Number(tarea.seconds_left ?? 0));
            const hecho = Math.max(0, total - quedan);
            const pausada = Boolean(tarea.paused);
            const parte = total ? Math.min(100, (hecho / total) * 100) : 0;
            const equipo = equipoDe(tarea);

            return (
              <article className={pausada ? 'tarea es-pausada' : 'tarea'} key={id}>
                <div className="tarea__cabecera">
                  <span className="tarea__nombre">{nombreDe(tarea)}</span>
                  <span className="tarea__reloj">{pausada ? '—' : segundos(quedan)}</span>
                </div>

                <div className="tarea__meta">
                  <span className="micro">#{id}</span>
                  {equipo && (
                    <span className="micro">
                      <Icono nombre="equipo" tamano={11} /> {equipo}
                    </span>
                  )}
                  <span className="micro">
                    {hecho} de {total} s
                  </span>
                  {pausada ? <Tag estado="off">pausada</Tag> : <Tag estado="obra">en marcha</Tag>}
                </div>

                <div className="barra barra--tiempo">
                  <i style={{ width: `${parte}%` }} />
                </div>

                <div className="tarea__acciones">
                  {pausada ? (
                    <button onClick={() => void ordenar(id, 'resume')}>Reanudar</button>
                  ) : (
                    <button onClick={() => void ordenar(id, 'pause')}>Pausar</button>
                  )}
                  <button onClick={() => void ordenar(id, 'delete')}>Cancelar</button>
                </div>
              </article>
            );
          })}
        </div>
      ) : (
        <div className="vacio">
          <div className="vacio__titulo">No hay nada en marcha</div>
          <p className="vacio__texto">
            Las tareas se lanzan desde donde significan algo: compilar en{' '}
            <Link to="/lab">Lab</Link>, cargar en memoria en <Link to="/software">Discos</Link>,
            entrar en una máquina en <Link to="/internet">Red</Link>. Aquí se ve cuánto les
            queda y cómo acabaron.
          </p>
        </div>
      )}

      <h3>Lo que ya terminó</h3>

      {terminadas.length ? (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Tarea</th>
                <th>Qué era</th>
                <th>Tardó</th>
                <th>Cómo acabó</th>
              </tr>
            </thead>
            <tbody>
              {terminadas.map((tarea) => {
                const id = Number(tarea.id);
                const fallo = tarea.status === 'failed';

                return (
                  <tr key={id} className={fallo ? 'off' : undefined}>
                    <td>#{id}</td>
                    <td>{nombreDe(tarea)}</td>
                    <td>
                      {Number(tarea.worked_seconds ?? tarea.ideal_seconds ?? 0)} s
                    </td>
                    <td>
                      {fallo ? <Tag estado="mal">falló</Tag> : <Tag estado="memoria">hecha</Tag>}
                      {Boolean(tarea.result_message) && (
                        <div className="micro" style={{ textTransform: 'none', letterSpacing: 0 }}>
                          {formatCell(tarea.result_message)}
                        </div>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      ) : (
        <p className="muted">Todavía no ha terminado ninguna.</p>
      )}
    </>
  );
}
