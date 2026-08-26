import { useCallback, useEffect, useMemo, useState } from 'react';
import { Link } from 'react-router-dom';

import { FileCard } from '../components/software/FileCard';
import { Meter } from '../components/software/Meter';
import { contextoDe, discoEnUso, ramEnUso } from '../components/software/rules';
import { WriteFileForm } from '../components/software/WriteFileForm';
import { EmptyNotice } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import {
  hardwareApi,
  processesApi,
  researchApi,
  softwareApi,
  type Machine,
  type OwnedFile,
  type SoftwareType,
} from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

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

  const cargar = useCallback(() => {
    softwareApi.listSoftware().then(setFicheros).catch(() => setFicheros([]));
  }, []);

  useEffect(() => {
    cargar();
    softwareApi.listSoftwareTypes().then(setTipos).catch(() => setTipos([]));
    hardwareApi.listMachines().then(setMaquinas).catch(() => setMaquinas([]));
  }, [cargar]);

  // El worker suma la flota entera para el disco y para la memoria, así que aquí igual.
  const capacidad = useMemo(
    () =>
      maquinas.reduce(
        (total, maquina) => ({ hdd: total.hdd + maquina.hdd, ram: total.ram + maquina.ram }),
        { hdd: 0, ram: 0 },
      ),
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

  const encolar = async (accion: number, label: string, id: number) => {
    try {
      const tarea = await processesApi.startProcess(accion, undefined, id);
      status.report(`${label}: tarea #${tarea.id}, lista en ${tarea.seconds} s.`);
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  const investigar = async (id: number) => {
    try {
      const tarea = await researchApi.startResearch(id);
      status.report(`Investigar: tarea #${tarea.process_id}, lista en ${tarea.seconds} s.`);
      cargar();
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

      <div className="swmeters">
        <Meter
          label="Disco"
          usado={discoEnUso(ficheros)}
          total={capacidad.hdd}
          unidad=""
          nota="Lo ocupa todo lo que tienes, corra o no."
        />
        <Meter
          label="Memoria"
          usado={ramEnUso(ficheros)}
          total={capacidad.ram}
          unidad=""
          nota="Solo la ocupa lo cargado. Es lo que decide si un programa entra."
        />
      </div>

      <StatusMessage text={status.message} as="block" />

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
          tipos={tipos}
          discoLibre={Math.max(0, capacidad.hdd - discoEnUso(ficheros))}
          onEscrito={cargar}
        />
      </section>
    </>
  );
}
