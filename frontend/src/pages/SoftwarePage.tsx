import { useCallback, useEffect, useMemo, useState } from 'react';
import { Link } from 'react-router-dom';

import { FileCard } from '../components/software/FileCard';
import { Meter } from '../components/software/Meter';
import { contextoDe, discoEnUso, ramEnUso } from '../components/software/rules';
import { WriteFileForm } from '../components/software/WriteFileForm';
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

  // El worker suma la flota entera para el disco y para la memoria, así que aquí igual.
  /**
   * La memoria, sumada. Ya no se suma el disco: cada fichero vive en un equipo y lo que
   * decide si cabe es ese disco y solo ese. La RAM sigue siendo un depósito común hasta que
   * la fase 3 la baje también a la máquina.
   */
  const capacidad = useMemo(
    () => ({ ram: maquinas.reduce((total, maquina) => total + maquina.ram, 0) }),
    [maquinas],
  );

  const ctx = useMemo(() => contextoDe(ficheros, capacidad.ram), [ficheros, capacidad.ram]);

  const nombreDeTipo = useMemo(
    () => new Map(tipos.map((uno) => [uno.id, uno.name])),
    [tipos],
  );

  const enMemoria = ficheros.filter((uno) => uno.running);
  const enDisco = ficheros.filter((uno) => !uno.running && !uno.hidden);
  const ocultos = ficheros.filter((uno) => uno.hidden);
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
        Un medidor de disco por equipo, y no uno solo.
        Sumarlos daria un numero que no decide nada: un fichero cabe o no cabe en **un**
        disco, y con el total en pantalla se vería sitio de sobra justo cuando el equipo
        donde intentas escribir está lleno.
        La memoria sigue siendo una sola: todavía es un depósito común.
      */}
      <div className="swmeters">
        {maquinas.map((maquina) => (
          <Meter
            key={maquina.id}
            label={`Disco · ${maquina.name}`}
            usado={discoEnUso(ficheros.filter((fichero) => fichero.server_id === maquina.id))}
            total={maquina.hdd}
            unidad=""
            nota="Lo que hay guardado en este equipo, corra o no."
          />
        ))}
        <Meter
          label="Memoria"
          usado={ramEnUso(ficheros)}
          total={capacidad.ram}
          unidad=""
          nota="Solo la ocupa lo cargado. Es lo que decide si un programa entra."
        />
      </div>

      <StatusMessage text={status.message} as="block" />
      {processId && <ProcessProgress processId={processId} onFinished={cargar} />}

      <section className="swgroup">
        <h3>
          Compilación <small>{compilaciones.length}</small>
        </h3>

        {compilacionActiva ? (
          <article className="swfile">
            <p className="swfile__head">
              <b>Compilando programa</b>
              <span className="swfile__type">en curso</span>
            </p>
            <p className="swfile__meta">
              Tarea #{compilacionActiva.id} · {compilacionActiva.workedSeconds}/
              {compilacionActiva.idealSeconds} s. Al terminar, el fichero aparecerá abajo en
              el disco.
            </p>
          </article>
        ) : compilaciones.length ? (
          <article className="swfile">
            <p className="swfile__head">
              <b>Última compilación</b>
              <span className="swfile__type">
                {compilaciones[0].status === 'succeeded' ? 'terminada' : compilaciones[0].status}
              </span>
            </p>
            <p className="swfile__meta">
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

      {/*
        Guardado, y en qué equipo. Antes era una lista sola porque el software era «del
        jugador»; ahora cada fichero está en un disco y saber en cuál es la mitad de lo que
        hay que saber para moverlo o para atacar con él.
      */}
      <section className="swgroup">
        <h3>
          En el disco <small>{enDisco.length}</small>
        </h3>

        {enDisco.length ? (
          maquinas.map((maquina) => {
            const suyos = enDisco.filter((fichero) => fichero.server_id === maquina.id);

            return (
              <div className="swmachine" key={maquina.id}>
                <h4 className="swmachine__head">
                  {maquina.name}
                  <small>
                    {suyos.length ? `${discoEnUso(suyos)} de ${maquina.hdd}` : 'vacío'}
                  </small>
                </h4>

                {suyos.length ? (
                  pintar(suyos)
                ) : (
                  <p className="muted swmachine__empty">
                    Sin nada guardado. Cabe todo su disco.
                  </p>
                )}
              </div>
            );
          })
        ) : (
          <EmptyNotice>
            El disco está vacío. Compila algo en <Link to="/arsenal">Arsenal</Link>, róbalo por
            ahí, o escríbelo a mano aquí abajo.
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

      <section className="swgroup">
        <h3>Escribir un fichero a mano</h3>

        <p className="muted">
          La vía del legado: eliges tipo, tamaño y RAM y aparece el programa. Para armarte, lo
          que quieres es <Link to="/arsenal">Arsenal</Link>; esto es para las herramientas que
          el sistema modular todavía no cubre, como el Hidder y el Seeker.
        </p>

        <WriteFileForm
          maquinas={maquinas}
          ficheros={ficheros}
          tipos={tipos}
          onEscrito={cargar}
        />
      </section>
    </>
  );
}
