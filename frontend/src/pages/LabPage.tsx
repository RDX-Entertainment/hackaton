import { tr, useIdioma, idiomaActual } from '../i18n/text';
import { useCallback, useEffect, useMemo, useState, type FormEvent } from 'react';

import { BlueprintForge } from '../components/arsenal/BlueprintForge';
import { OperationLauncher } from '../components/arsenal/OperationLauncher';
import { StatSheet } from '../components/arsenal/StatSheet';
import { DataTable } from '../components/DataTable';
import { Icono } from '../components/iconos/Icono';
import { EmptyNotice, Loading } from '../components/Notice';
import { CircuitRail } from '../components/onboarding/CircuitRail';
import { useOnboarding } from '../components/onboarding/useOnboarding';
import { ProcessProgress } from '../components/ProcessProgress';
import { StatusMessage } from '../components/StatusMessage';
import { Tag } from '../components/Tag';
import { useAppSelector } from '../app/hooks';
import { useAccion } from '../app/tutorial';
import {
  arsenalApi,
  hardwareApi,
  processesApi,
  rankingApi,
  researchApi,
  softwareApi,
  type Blueprint,
  type Capability,
  type Machine,
  type OwnedFile,
  type Program,
  type ProgramStats,
  type RankedPlayer,
  type ResearchModule,
  type UnknownRow,
} from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

/** La tarea de compilar, en la cola general del legado. */
const ACCION_COMPILAR = 91;

type Mesa = 'crear' | 'investigar' | 'mejorar' | 'lanzar';

const MESAS: { id: Mesa; nombre: string; que: string }[] = [
  { id: 'crear', get nombre() { return tr("Crear"); }, get que() { return tr("diseñar un programa y compilarlo"); } },
  { id: 'investigar', get nombre() { return tr("Investigar"); }, get que() { return tr("subirle la versión a uno que ya tienes"); } },
  { id: 'mejorar', get nombre() { return tr("Mejorar"); }, get que() { return tr("ajustar sus propiedades"); } },
  { id: 'lanzar', get nombre() { return tr("Lanzar"); }, get que() { return tr("usarlo contra una máquina"); } },
];

const PROPIEDADES: { key: keyof ProgramStats; label: string }[] = [
  { key: 'effectiveness', label: 'eficacia' },
  { key: 'penetration', get label() { return tr("penetración"); } },
  { key: 'stealth', label: 'sigilo' },
  { key: 'detection', get label() { return tr("detección"); } },
  { key: 'removal', get label() { return tr("eliminación"); } },
  { key: 'stability', label: 'estabilidad' },
];

const TACTICOS: { key: keyof ProgramStats; label: string }[] = [
  { key: 'resistance', label: 'resistencia' },
  { key: 'hardening', label: 'endurecimiento' },
  { key: 'manipulation', get label() { return tr("manipulación"); } },
  { key: 'integrity', label: 'integridad' },
  { key: 'anti_forensics', label: 'antiforense' },
  { key: 'forensics', label: 'forense' },
  { key: 'persistence', label: 'persistencia' },
];

const COSTE: { key: keyof ProgramStats; label: string }[] = [
  { key: 'cpu', label: 'CPU' },
  { key: 'ram', label: 'RAM' },
  { key: 'net', get label() { return tr("red"); } },
  { key: 'weight', get label() { return tr("peso en disco"); } },
];

/**
 * El laboratorio: donde un programa se hace y se afina.
 *
 * Antes esto estaban repartido en tres pantallas —Arsenal diseñaba y compilaba,
 * Investigación subía versiones, Lab tocaba atributos— y las tres hablaban del
 * mismo objeto en momentos distintos. Un jugador no distingue «arsenal» de
 * «laboratorio»: distingue **crear**, **investigar**, **mejorar** y **lanzar**,
 * que es como se llaman ahora las cuatro mesas.
 *
 * Lo que **no** vive aquí es dónde está guardado cada fichero: eso es Discos, y
 * es un sitio, no un taller.
 */
export function LabPage() {
  useIdioma();
  const status = useStatusMessage();
  const { onboarding } = useOnboarding();
  const yo = useAppSelector((state) => state.auth.user?.id ?? null);

  const [mesa, setMesa] = useState<Mesa>('crear');

  // Lo que el tutorial deja hacer aqui. El servidor lo veta igualmente; esto es para que el
  // boton lo diga antes de que el jugador lo pulse.
  const compilar_ = useAccion('lab.compilar');
  const investigar_ = useAccion('lab.investigar');

  const [capacidades, setCapacidades] = useState<Capability[]>([]);
  const [modulos, setModulos] = useState<ResearchModule[]>([]);
  const [programas, setProgramas] = useState<Program[]>([]);
  const [disenos, setDisenos] = useState<Blueprint[]>([]);
  const [jugadores, setJugadores] = useState<RankedPlayer[]>([]);
  const [maquinas, setMaquinas] = useState<Machine[]>([]);
  const [ficheros, setFicheros] = useState<OwnedFile[]>([]);
  const [historial, setHistorial] = useState<UnknownRow[]>([]);

  const [compilando, setCompilando] = useState<number>();
  const [tarea, setTarea] = useState<number>();
  const [elegido, setElegido] = useState<number>();
  const [borrador, setBorrador] = useState<ProgramStats>();
  const [ocupado, setOcupado] = useState(false);

  const cargar = useCallback(() => {
    arsenalApi.listCapabilities().then(setCapacidades).catch(() => setCapacidades([]));
    arsenalApi.listModules().then(setModulos).catch(() => setModulos([]));
    arsenalApi.listBlueprints().then(setDisenos).catch(() => setDisenos([]));
    arsenalApi
      .listPrograms()
      .then((lista) => {
        setProgramas(lista);
        setElegido((actual) => actual ?? lista[0]?.software_id);
      })
      .catch(() => setProgramas([]));
    softwareApi.listSoftware().then(setFicheros).catch(() => setFicheros([]));
    researchApi.listResearch().then(setHistorial).catch(() => setHistorial([]));
    processesApi
      .listProcesses()
      .then((filas) => {
        const viva = filas.find((fila) => Number(fila.action) === ACCION_COMPILAR && !fila.completed);
        setCompilando(viva ? Number(viva.id) : undefined);
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    cargar();
    rankingApi.listPlayers().then(setJugadores).catch(() => setJugadores([]));
    hardwareApi.listMachines().then(setMaquinas).catch(() => setMaquinas([]));

    const reloj = window.setInterval(cargar, 5_000);
    return () => window.clearInterval(reloj);
  }, [cargar]);

  const programa = useMemo(
    () => programas.find((uno) => uno.software_id === elegido),
    [programas, elegido, idiomaActual()],
  );

  useEffect(() => {
    if (programa) {
      setBorrador(programa.stats);
    }
  }, [programa]);

  const nombreDeCapacidad = useMemo(
    () => new Map(capacidades.map((una) => [una.code, tr(una.name)])),
    [capacidades, idiomaActual()],
  );

  const compilar = async (diseno: Blueprint) => {
    if (compilando) {
      return;
    }

    try {
      const hecho = await arsenalApi.compile(diseno.id);
      setCompilando(hecho.process_id);
      status.report(
        tr("Compilando «{0}» ({1} s). El fichero aparece en Discos al terminar.", hecho.name, hecho.seconds),
      );
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  const investigar = async (id: number) => {
    try {
      const encolado = await researchApi.startResearch(id);
      setTarea(encolado.process_id);
      status.report(tr("Investigando: tarea #{0} ({1} s).", encolado.process_id, encolado.seconds));
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  const guardarPerfil = async (event: FormEvent) => {
    event.preventDefault();

    if (!programa || !borrador) {
      return;
    }

    setOcupado(true);

    try {
      await arsenalApi.tuneProgram(programa.software_id, borrador);
      status.report(tr("Perfil guardado."));
      cargar();
    } catch (error) {
      status.reportError(error);
    } finally {
      setOcupado(false);
    }
  };

  const poner = (key: keyof ProgramStats, valor: number) =>
    setBorrador((actual) => actual && { ...actual, [key]: valor });

  const investigables = ficheros.filter((uno) => !uno.folder);

  return (
    <>
      <h2>Lab</h2>

      <CircuitRail onboarding={onboarding} />

      <nav className="mesas">
        {MESAS.map((una) => (
          <button
            key={una.id}
            type="button"
            className={mesa === una.id ? 'mesas__mesa is-current' : 'mesas__mesa'}
            onClick={() => setMesa(una.id)}
          >
            <b>{una.nombre}</b>
            <span>{una.que}</span>
          </button>
        ))}
      </nav>

      <StatusMessage text={status.message} as="block" />
      {tarea && <ProcessProgress processId={tarea} onFinished={cargar} />}

      {mesa === 'crear' && (
        <>
          <BlueprintForge capacidades={capacidades} modulos={modulos} onCreado={cargar} />

          <h3>{tr("Diseños guardados")}</h3>
          {disenos.length ? (
            disenos.map((diseno) => (
              <article className="tarjeta" key={diseno.id} style={{ marginBottom: '0.75rem' }}>
                <div className="tarjeta__cabecera">
                  <span className="tarjeta__titulo">
                    {diseno.name} <small>r{diseno.revision}</small>
                  </span>
                  <span className="micro">
                    {nombreDeCapacidad.get(diseno.capability) ?? diseno.capability} ·{' '}
                    {diseno.modules.length} {tr("módulo")}{diseno.modules.length === 1 ? '' : 's'}
                  </span>
                </div>

                <StatSheet stats={diseno.projected} />

                <div className="tarjeta__acciones">
                  {compilando ? (
                    <Tag estado="obra">{tr("hay una compilación en marcha")}</Tag>
                  ) : (
                    <Tag estado="plano" />
                  )}
                  <button
                    data-foco="lab.compilar"
                    className={compilar_.destacada ? 'es-foco' : undefined}
                    disabled={!!compilando || !compilar_.permitida}
                    title={compilar_.motivo ?? undefined}
                    onClick={() => void compilar(diseno)}
                  >
                    {tr("Compilar")} </button>
                </div>
              </article>
            ))
          ) : (
            <div className="vacio">
              <div className="vacio__titulo">{tr("Ningún diseño todavía")}</div>
              <p className="vacio__texto">
                {tr("Un programa es una capacidad más los módulos que le pongas. Se diseña arriba, se compila aquí, y el fichero aparece en Discos.")} </p>
            </div>
          )}

          {compilando && <ProcessProgress processId={compilando} onFinished={cargar} />}
        </>
      )}

      {mesa === 'investigar' && (
        <>
          <p className="muted">
            {tr("Investigar sube la versión de un programa que ya tienes. Tarda, y hasta que no termina la versión sigue siendo la de antes.")} </p>

          {investigables.length ? (
            <div className="rejilla-tarjetas">
              {investigables.map((fichero) => (
                <article className="tarjeta" key={fichero.id}>
                  <span className="tarjeta__rotulo">
                    <Icono nombre="disco" tamano={12} />{' '}
                    {maquinas.find((maquina) => maquina.id === fichero.server_id)?.name ?? tr("equipo")}
                  </span>

                  <div className="tarjeta__cabecera" style={{ marginTop: '0.35rem' }}>
                    <span className="tarjeta__titulo">{fichero.name}</span>
                    <span className="micro">v{fichero.version}</span>
                  </div>

                  <div className="tarjeta__acciones">
                    {fichero.running ? <Tag estado="memoria" /> : <Tag estado="disco" />}
                    <button
                      data-foco="lab.investigar"
                      className={investigar_.destacada ? 'es-foco' : undefined}
                      disabled={!investigar_.permitida}
                      title={investigar_.motivo ?? undefined}
                      onClick={() => void investigar(fichero.id)}
                    >
                      {tr("Investigar")} </button>
                  </div>
                </article>
              ))}
            </div>
          ) : (
            <EmptyNotice>
              {tr("No tienes ningún programa que investigar. Crea uno en la mesa de al lado.")} </EmptyNotice>
          )}

          <h3>{tr("Lo investigado")}</h3>
          <DataTable rows={historial} />
        </>
      )}

      {mesa === 'mejorar' && (
        <>
          {!programas.length ? (
            <EmptyNotice>
              {tr("Todavía no hay ningún programa con perfil. Compila uno primero.")} </EmptyNotice>
          ) : !borrador || !programa ? (
            <Loading />
          ) : (
            <>
              <label className="lab__picker">
                {tr("Programa")} <select
                  value={elegido ?? ''}
                  onChange={(event) => setElegido(Number(event.target.value))}
                >
                  {programas.map((uno) => (
                    <option key={uno.software_id} value={uno.software_id}>
                      {uno.name} · {tr(uno.capability_name ?? tr("sin capacidad"))}
                    </option>
                  ))}
                </select>
              </label>

              <form className="compose lab__form" onSubmit={guardarPerfil}>
                <h3>
                  {programa.name} · {tr(programa.capability_name ?? '')}
                </h3>
                <p className="muted">
                  {programa.running
                    ? tr("En memoria: lo que guardes aquí lo leen las operaciones que empiecen después.")
                    : tr("En disco: cárgalo en memoria antes de lanzar nada con él.")}
                </p>

                <fieldset>
                  <legend>{tr("Propiedades (0–100)")}</legend>
                  {PROPIEDADES.map((campo) => (
                    <label key={campo.key}>
                      {campo.label}
                      <input
                        type="number"
                        min={0}
                        max={100}
                        value={borrador[campo.key]}
                        onChange={(event) => poner(campo.key, Number(event.target.value))}
                      />
                    </label>
                  ))}
                </fieldset>

                <fieldset>
                  <legend>{tr("Atributos tácticos (0–100)")}</legend>
                  {TACTICOS.map((campo) => (
                    <label key={campo.key}>
                      {campo.label}
                      <input
                        type="number"
                        min={0}
                        max={100}
                        value={borrador[campo.key]}
                        onChange={(event) => poner(campo.key, Number(event.target.value))}
                      />
                    </label>
                  ))}
                </fieldset>

                <fieldset>
                  <legend>{tr("Coste de ejecución")}</legend>
                  {COSTE.map((campo) => (
                    <label key={campo.key}>
                      {campo.label}
                      <input
                        type="number"
                        min={1}
                        value={borrador[campo.key]}
                        onChange={(event) => poner(campo.key, Number(event.target.value))}
                      />
                    </label>
                  ))}
                </fieldset>

                <p className="muted">
                  {tr("Pares tácticos: eficacia ↔ resistencia · penetración ↔ endurecimiento · sigilo ↔ detección · manipulación ↔ integridad · antiforense ↔ forense · persistencia ↔ eliminación.")} </p>

                <button disabled={ocupado}>{tr("Guardar perfil")}</button>
              </form>
            </>
          )}
        </>
      )}

      {mesa === 'lanzar' && (
        <>
          <p className="muted">
            {tr("Solo dispara lo que esté cargado en memoria. Lo demás está en el disco, esperando.")} </p>

          <OperationLauncher
            programas={programas}
            jugadores={jugadores}
            maquinas={maquinas}
            yo={yo}
            onLanzada={cargar}
          />
        </>
      )}
    </>
  );
}
