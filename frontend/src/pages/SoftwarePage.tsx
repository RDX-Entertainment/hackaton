import { useCallback, useEffect, useMemo, useState } from 'react';
import { Link } from 'react-router-dom';

import { FileCard } from '../components/software/FileCard';
import { Meter } from '../components/software/Meter';
import { contextoDe, discoEnUso, ramEnUso } from '../components/software/rules';
import { MachineSelector } from '../components/hardware/MachineSelector';
import { EmptyNotice } from '../components/Notice';
import { ProcessProgress } from '../components/ProcessProgress';
import { StatusMessage } from '../components/StatusMessage';
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
  const status = useStatusMessage();

  const [ficheros, setFicheros] = useState<OwnedFile[]>([]);
  const [tipos, setTipos] = useState<SoftwareType[]>([]);
  const [maquinas, setMaquinas] = useState<Machine[]>([]);
  /**
   * El equipo que se está mirando. Antes esta pantalla enseñaba todo junto porque el software
   * era «del jugador»; ahora vive en un disco, y verlo todo mezclado esconde justo el dato que
   * decide qué puedes hacer con cada cosa.
   */
  const [equipo, setEquipo] = useState(0);
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
    hardwareApi
      .listMachines()
      .then((lista) => {
        setMaquinas(lista);
        setEquipo((actual) =>
          lista.some((maquina) => maquina.id === actual) ? actual : lista[0]?.id ?? 0,
        );
      })
      .catch(() => setMaquinas([]));
  }, [cargar]);

  // La compilación la resuelve el worker: refrescar aquí evita obligar a ir a Procesos para
  // descubrir que ya terminó y hace que el fichero aparezca solo en el disco.
  useEffect(() => {
    const timer = window.setInterval(cargar, 5_000);
    return () => window.clearInterval(timer);
  }, [cargar]);

  const nombreDeTipo = useMemo(
    () => new Map(tipos.map((uno) => [uno.id, uno.name])),
    [tipos],
  );

  // Lo que hay en el equipo que se está mirando, cargado o no. Un programa corre en la
  // máquina donde está guardado y gasta su disco y su memoria, así que todo lo de esta
  // pantalla es de este equipo.
  const aqui = ficheros.filter((uno) => uno.server_id === equipo);

  const elegido = maquinas.find((maquina) => maquina.id === equipo);

  // Las reglas de «¿cabe en memoria?» se miden contra este equipo: un programa corre donde
  // está guardado y gasta la RAM de ahí. El backend hace la misma cuenta al cargarlo.
  const ctx = useMemo(
    () => contextoDe(aqui, elegido?.ram ?? 0),
    [aqui, elegido?.ram],
  );

  const enMemoria = aqui.filter((uno) => uno.running);
  const enDisco = aqui.filter((uno) => !uno.running && !uno.hidden);
  const ocultos = aqui.filter((uno) => uno.hidden);
  const compilacionActiva = compilaciones.find((una) => !una.completed);

  const encolar = async (accion: number, label: string, id: number) => {
    try {
      const tarea = await processesApi.startProcess(accion, undefined, id);
      setProcessId(tarea.id);
      status.report(`${label}: tarea #${tarea.id}, lista en ${tarea.seconds} s.`);
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  const investigar = async (id: number) => {
    try {
      const tarea = await researchApi.startResearch(id);
      setProcessId(tarea.process_id);
      status.report(`Investigar: tarea #${tarea.process_id}, lista en ${tarea.seconds} s.`);
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
      status.report('Fichero movido.');
      await cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  const pintar = (lista: OwnedFile[]) =>
    lista.map((fichero) => (
      <FileCard
        key={fichero.id}
        fichero={fichero}
        tipo={nombreDeTipo.get(fichero.type) ?? `tipo ${fichero.type}`}
        ctx={ctx}
        onTarea={(accion, label) => void encolar(accion, label, fichero.id)}
        onInvestigar={() => void investigar(fichero.id)}
        destinos={maquinas.filter((maquina) => maquina.id !== fichero.server_id)}
        onMover={(destino) => void mover(fichero.id, destino)}
      />
    ));

  return (
    <>
      <h2>Software</h2>

      <p className="muted">
        Un fichero en el disco no hace nada: hay que cargarlo en memoria, y la memoria es lo
        que limita cuántos programas puedes tener a la vez. Cada verbo de esta pantalla encola
        una tarea, y el resultado se ve en <Link to="/processes">Procesos</Link>.
      </p>

      {/*
        El disco del equipo que se está mirando, no la suma ni uno por máquina: el selector ya
        dice cuál es, y repetir el resto llenaba la pantalla de números que no decidían nada.
        La memoria sigue siendo una sola: aún es un depósito común.
      */}
      <MachineSelector machines={maquinas} selected={equipo} onSelect={setEquipo} />

      <div className="swmeters">
        <Meter
          label={`Disco · ${elegido?.name ?? '—'}`}
          usado={discoEnUso(aqui)}
          total={elegido?.hdd ?? 0}
          unidad=""
          nota="Lo que hay guardado en este equipo, corra o no."
        />
        <Meter
          label={`Memoria · ${elegido?.name ?? '—'}`}
          usado={ramEnUso(aqui)}
          total={elegido?.ram ?? 0}
          unidad=""
          nota="Solo la ocupa lo cargado en este equipo."
        />
      </div>

      <StatusMessage text={status.message} as="block" />
      {processId && <ProcessProgress processId={processId} onFinished={cargar} />}

      <section className="swgroup">
        <h3>
          Compilación <small>{compilaciones.length}</small>
        </h3>

        {compilacionActiva ? (
          <article className="swnote">
            <p className="swnote__head">
              <b>Compilando programa</b>
              <span className="swfile__type">en curso</span>
            </p>
            <p className="swnote__meta">
              Tarea #{compilacionActiva.id} · {compilacionActiva.workedSeconds}/
              {compilacionActiva.idealSeconds} s. Al terminar, el fichero aparecerá abajo en
              el disco.
            </p>
          </article>
        ) : compilaciones.length ? (
          <article className="swnote">
            <p className="swnote__head">
              <b>Última compilación</b>
              <span className="swfile__type">
                {compilaciones[0].status === 'succeeded' ? 'terminada' : compilaciones[0].status}
              </span>
            </p>
            <p className="swnote__meta">
              {compilaciones[0].resultMessage ?? 'La compilación no dejó un mensaje.'}
            </p>
          </article>
        ) : (
          <EmptyNotice>
            Diseña un programa en <Link to="/arsenal">Arsenal</Link>. La compilación se verá
            y terminará aquí.
          </EmptyNotice>
        )}
      </section>

      <section className="swgroup">
        <h3>
          En memoria <small>{enMemoria.length}</small>
        </h3>

        {enMemoria.length ? (
          pintar(enMemoria)
        ) : (
          <EmptyNotice>
            Nada cargado. Sin esto no funciona ningún programa, ni el tuyo ni el que te
            defiende.
          </EmptyNotice>
        )}
      </section>

      <section className="swgroup">
        <h3>
          En el disco <small>{enDisco.length}</small>
        </h3>

        {enDisco.length ? (
          pintar(enDisco)
        ) : (
          <EmptyNotice>
            {elegido?.name ?? 'Este equipo'} no tiene nada guardado. Compílalo en{' '}
            <Link to="/arsenal">Arsenal</Link>, róbalo por ahí o tráelo de otro equipo.
          </EmptyNotice>
        )}
      </section>

      {ocultos.length > 0 && (
        <section className="swgroup">
          <h3>
            Ocultos <small>{ocultos.length}</small>
          </h3>

          <p className="muted">
            Escondidos con un Hidder. Solo los saca un Seeker de versión igual o mejor que la
            que los escondió.
          </p>

          {pintar(ocultos)}
        </section>
      )}

    </>
  );
}
