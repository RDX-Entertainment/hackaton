import { tr, useIdioma, idiomaActual } from '../i18n/text';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { Link } from 'react-router-dom';

import { FileCard } from '../components/software/FileCard';
import { Migas, Volumenes, type Sitio } from '../components/software/Explorador';
import { Icono } from '../components/iconos/Icono';
import { Meter } from '../components/software/Meter';
import { contextoDe, discoEnUso, ramEnUso, noSePuedeInstalar } from '../components/software/rules';
import { CircuitRail } from '../components/onboarding/CircuitRail';
import { useTutorial } from '../app/tutorial';
import { ProcessProgress } from '../components/ProcessProgress';
import { StatusMessage } from '../components/StatusMessage';
import { Tag } from '../components/Tag';
import {
  hardwareApi,
  processesApi,
  researchApi,
  softwareApi,
  type Machine,
  type OwnedFile,
  type SoftwareType,
  type UnknownRow,
} from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

type Compilation = {
  id: number;
  idealSeconds: number;
  workedSeconds: number;
  completed: boolean;
  status: string;
  resultCode: string | null;
  resultMessage: string | null;
};

/** La cola general conserva también las tareas del legado; aquí solo se ven las 91. */
function lecturaDeCompilacion(row: UnknownRow): Compilation | null {
  if (Number(row.action) !== 91) return null;

  return {
    id: Number(row.id),
    idealSeconds: Number(row.ideal_seconds),
    workedSeconds: Math.max(0, Number(row.ideal_seconds) - Number(row.seconds_left ?? 0)),
    completed: Boolean(row.completed),
    status: String(row.status ?? 'running'),
    resultCode: typeof row.result_code === 'string' ? row.result_code : null,
    resultMessage: typeof row.result_message === 'string' ? row.result_message : null,
  };
}

/**
 * El disco de la máquina.
 *
 * La pantalla se ordena por dónde está cada fichero, porque es lo que decide qué se puede
 * hacer con él: lo cargado en memoria es lo único que funciona, lo del disco está a la
 * espera, y lo oculto no se ve hasta que un Seeker lo saque. Antes era una lista plana con
 * seis botones por fila y ninguna pista de por qué la mitad no hacían nada.
 */
export function SoftwarePage() {
  useIdioma();
  const status = useStatusMessage();
  const { onboarding } = useTutorial();

  const [ficheros, setFicheros] = useState<OwnedFile[]>([]);
  const [tipos, setTipos] = useState<SoftwareType[]>([]);
  const [maquinas, setMaquinas] = useState<Machine[]>([]);
  /**
   * El equipo que se está mirando. Antes esta pantalla enseñaba todo junto porque el software
   * era «del jugador»; ahora vive en un disco, y verlo todo mezclado esconde justo el dato que
   * decide qué puedes hacer con cada cosa.
   */
  /**
   * Dónde estás mirando. `null` es la raíz: la lista de volúmenes, que hoy son
   * tus equipos y mañana puede incluir la nube.
   */
  const [sitio, setSitio] = useState<Sitio>(null);
  const [nuevaCarpeta, setNuevaCarpeta] = useState('');
  const [abierto, setAbierto] = useState<number>();
  const [compilaciones, setCompilaciones] = useState<Compilation[]>([]);
  const [processId, setProcessId] = useState<number>();

  const cargar = useCallback(() => {
    softwareApi.listSoftware().then(setFicheros).catch(() => setFicheros([]));
    processesApi
      .listProcesses()
      .then((tareas) => tareas.map(lecturaDeCompilacion).filter((tarea): tarea is Compilation => tarea !== null))
      .then(setCompilaciones)
      .catch(() => setCompilaciones([]));
  }, []);

  useEffect(() => {
    cargar();
    softwareApi.listSoftwareTypes().then(setTipos).catch(() => setTipos([]));
    hardwareApi.listMachines().then(setMaquinas).catch(() => setMaquinas([]));
  }, [cargar]);

  // La compilación la resuelve el worker: refrescar aquí evita obligar a ir a Procesos para
  // descubrir que ya terminó y hace que el fichero aparezca solo en el disco.
  useEffect(() => {
    const timer = window.setInterval(cargar, 5_000);
    return () => window.clearInterval(timer);
  }, [cargar]);

  const nombreDeTipo = useMemo(
    () => new Map(tipos.map((uno) => [uno.id, tr(uno.name)])),
    [tipos, idiomaActual()],
  );

  // Lo que hay en el equipo que se está mirando, cargado o no. Un programa corre en la
  // máquina donde está guardado y gasta su disco y su memoria, así que todo lo de esta
  // pantalla es de este equipo.
  const equipo = sitio?.volumen ?? 0;

  /** Todo lo que hay en este equipo, en cualquier carpeta: las cuentas son suyas. */
  const enElEquipo = ficheros.filter((uno) => uno.server_id === equipo);

  /** Y lo que hay **en esta carpeta**, que es lo que se enseña. */
  const aqui = enElEquipo.filter((uno) => (uno.parent_id ?? null) === (sitio?.carpeta ?? null));

  const carpetas = aqui.filter((uno) => uno.folder);
  const archivos = aqui.filter((uno) => !uno.folder);

  const elegido = maquinas.find((maquina) => maquina.id === equipo);

  // Las reglas de «¿cabe en memoria?» se miden contra este equipo: un programa corre donde
  // está guardado y gasta la RAM de ahí. El backend hace la misma cuenta al cargarlo.
  const ctx = useMemo(
    () => contextoDe(enElEquipo, elegido?.ram ?? 0),
    [enElEquipo, elegido?.ram, idiomaActual()],
  );

  const compilacionActiva = compilaciones.find((una) => !una.completed);

  const crearCarpeta = async () => {
    if (!sitio || !nuevaCarpeta.trim()) {
      return;
    }

    try {
      await softwareApi.createFolder(nuevaCarpeta.trim(), sitio.volumen, sitio.carpeta);
      setNuevaCarpeta('');
      status.report(tr("Carpeta creada."));
      await cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  const borrarCarpeta = async (id: number) => {
    try {
      await softwareApi.deleteFolder(id);
      status.report(tr("Carpeta borrada."));
      await cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  /** Meter algo en otra carpeta del mismo equipo, o sacarlo a la raíz. */
  const meterEn = async (id: number, destino: number | null) => {
    try {
      await softwareApi.moveIntoFolder(id, destino);
      status.report(tr("Movido."));
      await cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  const encolar = async (accion: number, label: string, id: number) => {
    try {
      const tarea = await processesApi.startProcess(accion, undefined, id);
      setProcessId(tarea.id);
      status.report(tr("{0}: tarea #{1}, lista en {2} s.", label, tarea.id, tarea.seconds));
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  const investigar = async (id: number) => {
    try {
      const tarea = await researchApi.startResearch(id);
      setProcessId(tarea.process_id);
      status.report(tr("Investigar: tarea #{0}, lista en {1} s.", tarea.process_id, tarea.seconds));
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  /**
   * Llevar un fichero a otro equipo.
   *
   * Recarga entera al terminar en vez de tocar la lista a mano: al moverse cambian dos
   * medidores de disco a la vez -- el de origen y el de destino -- y actualizarlos por
   * separado es la clase de cuenta que se desincroniza a la primera.
   */
  const mover = async (fichero: number, destino: number) => {
    try {
      await softwareApi.moveSoftware(fichero, destino);
      status.report(tr("Fichero movido."));
      await cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>{tr("Discos")}</h2>

      <CircuitRail onboarding={onboarding} />

      <Migas sitio={sitio} maquinas={maquinas} ficheros={ficheros} onIr={setSitio} />
      <StatusMessage text={status.message} as="block" />

      {!sitio ? (
        <>
          <p className="muted">
            {tr("En la raíz no hay ficheros: hay volúmenes. Un fichero vive dentro de una máquina, igual que en un ordenador de verdad no hay nada flotando por encima de los discos.")} </p>

          <Volumenes
            maquinas={maquinas}
            ficheros={ficheros}
            onEntrar={(id) => setSitio({ volumen: id, carpeta: null })}
          />
        </>
      ) : (
        <>
          <div className="swmeters">
            <Meter
              label={tr("Disco · {0}", elegido?.name ?? '—')}
              usado={discoEnUso(enElEquipo)}
              total={elegido?.hdd ?? 0}
              unidad=""
              nota="Todo lo guardado en este equipo, esté en la carpeta que esté."
            />
            <Meter
              label={tr("Memoria · {0}", elegido?.name ?? '—')}
              usado={ramEnUso(enElEquipo)}
              total={elegido?.ram ?? 0}
              unidad=""
              nota="Solo la ocupa lo cargado en este equipo."
            />
          </div>

          {processId && <ProcessProgress processId={processId} onFinished={cargar} />}

          {compilacionActiva && (
            <article className="swnote">
              <p className="swnote__head">
                <b>{tr("Compilando programa")}</b>
                <span className="swfile__type">{tr("en curso")}</span>
              </p>
              <p className="swnote__meta">
                {tr("Al terminar, el fichero aparece en la raíz del equipo donde quepa.")} </p>
              <ProcessProgress processId={compilacionActiva.id} title={tr("Compilando programa")} onFinished={cargar} />
            </article>
          )}

          <div className="explorador__barra">
            <input
              value={nuevaCarpeta}
              placeholder={tr("nombre de la carpeta")}
              maxLength={30}
              onChange={(event) => setNuevaCarpeta(event.target.value)}
            />
            <button disabled={!nuevaCarpeta.trim()} onClick={() => void crearCarpeta()}>
              {tr("Nueva carpeta")} </button>
          </div>

          <div className="explorador__lista">
            {carpetas.map((carpeta) => (
              <div className="explorador__fila es-carpeta" key={carpeta.id}>
                <button
                  type="button"
                  className="explorador__abrir"
                  onClick={() => setSitio({ volumen: equipo, carpeta: carpeta.id })}
                >
                  <Icono nombre="disco" tamano={18} />
                  <span className="explorador__nombre">{carpeta.name}</span>
                </button>

                <span className="micro">
                  {enElEquipo.filter((uno) => uno.parent_id === carpeta.id).length} {tr("dentro")} </span>

                <button type="button" onClick={() => void borrarCarpeta(carpeta.id)}>
                  {tr("Borrar")} </button>
              </div>
            ))}

            {archivos.map((fichero) => (
              <div className="explorador__entrada" key={fichero.id}>
                <div className="explorador__fila">
                  <button
                    type="button"
                    className="explorador__abrir"
                    data-guia={!noSePuedeInstalar(fichero, ctx) ? 'discos.fichero' : undefined}
                    onClick={() =>
                      setAbierto((actual) => (actual === fichero.id ? undefined : fichero.id))
                    }
                  >
                    <Icono nombre={fichero.running ? 'memoria' : 'disco'} tamano={18} />
                    <span className="explorador__nombre">
                      {fichero.name}
                      {fichero.version ? <small> v{fichero.version}</small> : null}
                    </span>
                  </button>

                  <span className="micro">{nombreDeTipo.get(fichero.type) ?? tr("tipo {0}", fichero.type)}</span>
                  <span className="micro">{fichero.size} MB</span>

                  {fichero.running ? (
                    <Tag estado="memoria" />
                  ) : fichero.hidden ? (
                    <Tag estado="off">{tr("oculto")}</Tag>
                  ) : (
                    <Tag estado="disco" />
                  )}

                  <select
                    value={fichero.parent_id ?? ''}
                    onChange={(event) =>
                      void meterEn(fichero.id, event.target.value ? Number(event.target.value) : null)
                    }
                  >
                    <option value="">{tr("raíz del equipo")}</option>
                    {enElEquipo
                      .filter((uno) => uno.folder)
                      .map((carpeta) => (
                        <option key={carpeta.id} value={carpeta.id}>
                          {carpeta.name}
                        </option>
                      ))}
                  </select>
                </div>

                {abierto === fichero.id && (
                  <FileCard
                    fichero={fichero}
                    tipo={nombreDeTipo.get(fichero.type) ?? tr("tipo {0}", fichero.type)}
                    ctx={ctx}
                    onTarea={(accion, label) => void encolar(accion, label, fichero.id)}
                    onInvestigar={() => void investigar(fichero.id)}
                    destinos={maquinas.filter((maquina) => maquina.id !== fichero.server_id)}
                    onMover={(destino) => void mover(fichero.id, destino)}
                  />
                )}
              </div>
            ))}

            {!aqui.length && (
              <p className="correo__nada">
                {tr("Esta carpeta está vacía. Lo que compiles en")} <Link to="/lab">Lab</Link> {tr("aparece en la raíz del equipo, y desde ahí lo puedes ordenar.")} </p>
            )}
          </div>
        </>
      )}
    </>
  );
}
