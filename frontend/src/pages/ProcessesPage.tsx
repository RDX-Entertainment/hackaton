import { tr, useIdioma } from '../i18n/text';
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
  get 1() { return tr("Descargar un fichero"); },
  get 2() { return tr("Subir un fichero"); },
  get 3() { return tr("Borrar un fichero"); },
  get 4() { return tr("Ocultar un fichero"); },
  get 5() { return tr("Buscar lo oculto"); },
  get 6() { return tr("Recoger lo del virus"); },
  get 7() { return tr("Pasar el antivirus"); },
  get 8() { return tr("Falsear el registro"); },
  get 9() { return tr("Borrar el registro"); },
  get 10() { return tr("Formatear el disco"); },
  get 11() { return tr("Entrar en una máquina"); },
  get 12() { return tr("Entrar en un banco"); },
  get 13() { return tr("Cargar en memoria"); },
  get 14() { return tr("Sacar de memoria"); },
  get 15() { return tr("Mirar los puertos"); },
  get 16() { return tr("Entrar para aprender"); },
  get 17() { return tr("Investigar"); },
  get 18() { return tr("Subir al disco externo"); },
  get 19() { return tr("Bajar del disco externo"); },
  get 20() { return tr("Borrar del disco externo"); },
  get 22() { return tr("Rastrear la red"); },
  get 23() { return tr("Analizar"); },
  get 24() { return tr("Instalar el doom"); },
  get 25() { return tr("Cambiar de dirección"); },
  get 26() { return tr("Cambiar la contraseña"); },
  get 27() { return tr("Tumbar a base de peticiones"); },
  get 28() { return tr("Levantar el servidor web"); },
  get 91() { return tr("Compilar un programa"); },
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
  useIdioma();
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
    const dicho = { pause: tr("Pausada."), resume: tr("Reanudada."), delete: tr("Cancelada.") };

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
      <h2>{tr("Tareas")}</h2>
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
                    {hecho} {tr("de")} {total} s
                  </span>
                  {pausada ? <Tag estado="off">{tr("pausada")}</Tag> : <Tag estado="obra">{tr("en marcha")}</Tag>}
                </div>

                <div className="barra barra--tiempo">
                  <i style={{ width: `${parte}%` }} />
                </div>

                <div className="tarea__acciones">
                  {pausada ? (
                    <button onClick={() => void ordenar(id, 'resume')}>{tr("Reanudar")}</button>
                  ) : (
                    <button onClick={() => void ordenar(id, 'pause')}>{tr("Pausar")}</button>
                  )}
                  <button onClick={() => void ordenar(id, 'delete')}>{tr("Cancelar")}</button>
                </div>
              </article>
            );
          })}
        </div>
      ) : (
        <div className="vacio">
          <div className="vacio__titulo">{tr("No hay nada en marcha")}</div>
          <p className="vacio__texto">
            {tr("Las tareas se lanzan desde donde significan algo: compilar en")}{' '}
            <Link to="/lab">Lab</Link>{tr(", cargar en memoria en")} <Link to="/software">{tr("Discos")}</Link>{tr(", entrar en una máquina en")} <Link to="/internet">{tr("Red")}</Link>{tr(". Aquí se ve cuánto les queda y cómo acabaron.")} </p>
        </div>
      )}

      <h3>{tr("Lo que ya terminó")}</h3>

      {terminadas.length ? (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{tr("Tarea")}</th>
                <th>{tr("Qué era")}</th>
                <th>{tr("Tardó")}</th>
                <th>{tr("Cómo acabó")}</th>
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
                      {fallo ? <Tag estado="mal">{tr("falló")}</Tag> : <Tag estado="memoria">{tr("hecha")}</Tag>}
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
        <p className="muted">{tr("Todavía no ha terminado ninguna.")}</p>
      )}
    </>
  );
}
