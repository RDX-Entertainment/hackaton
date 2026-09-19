import { tr, useIdioma, idiomaActual } from '../i18n/text';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { BlueprintForge } from '../components/arsenal/BlueprintForge';
import { AntivirusPanel } from '../components/arsenal/AntivirusPanel';
import { ModuleWorkshop } from '../components/arsenal/ModuleWorkshop';
import { OperationLauncher } from '../components/arsenal/OperationLauncher';
import { StatSheet } from '../components/arsenal/StatSheet';
import { DataTable } from '../components/DataTable';
import { Icono } from '../components/iconos/Icono';
import { EmptyNotice } from '../components/Notice';
import { CircuitRail } from '../components/onboarding/CircuitRail';
import { ProcessProgress } from '../components/ProcessProgress';
import { StatusMessage } from '../components/StatusMessage';
import { Tag } from '../components/Tag';
import { useAppSelector } from '../app/hooks';
import { useAccion, useTutorial } from '../app/tutorial';
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
  type RankedPlayer,
  type ResearchModule,
  type UnknownRow,
} from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';
import { messageFrom } from '../utils/errors';
import { useStableData } from '../utils/useStableData';

/** La tarea de compilar, en la cola general del legado. */
const ACCION_COMPILAR = 91;

type Mesa = 'crear' | 'investigar' | 'mejorar' | 'lanzar';

const MESAS: { id: Mesa; nombre: string; que: string }[] = [
  { id: 'crear', get nombre() { return tr("Crear"); }, get que() { return tr("diseñar un programa y compilarlo"); } },
  { id: 'investigar', get nombre() { return tr("Investigar"); }, get que() { return tr("subirle la versión a uno que ya tienes"); } },
  { id: 'mejorar', get nombre() { return tr("Mejorar"); }, get que() { return tr("instalar módulos y fusionar copias"); } },
  { id: 'lanzar', get nombre() { return tr("Lanzar"); }, get que() { return tr("usarlo contra una máquina"); } },
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
  const { onboarding } = useTutorial();
  const yo = useAppSelector((state) => state.auth.user?.id ?? null);

  const [mesa, setMesa] = useState<Mesa>('crear');

  // Lo que el tutorial deja hacer aqui. El servidor lo veta igualmente; esto es para que el
  // boton lo diga antes de que el jugador lo pulse.
  const compilar_ = useAccion('lab.compilar');
  const investigar_ = useAccion('lab.investigar');

  const [capacidades, setCapacidades] = useStableData<Capability[]>([]);
  const [modulos, setModulos] = useStableData<ResearchModule[]>([]);
  const [moduleError, setModuleError] = useState('');
  const [programas, setProgramas] = useStableData<Program[]>([]);
  const [disenos, setDisenos] = useStableData<Blueprint[]>([]);
  const [jugadores, setJugadores] = useStableData<RankedPlayer[]>([]);
  const [maquinas, setMaquinas] = useStableData<Machine[]>([]);
  const [ficheros, setFicheros] = useStableData<OwnedFile[]>([]);
  const [historial, setHistorial] = useStableData<UnknownRow[]>([]);

  const [compilando, setCompilando] = useState<number>();
  const [tarea, setTarea] = useState<number>();
  const [researchHere, setResearchHere] = useState<number>();
  const [compilationHere, setCompilationHere] = useState<{ id: number; blueprint: number; seconds: number }>();
  const [startingCompilation, setStartingCompilation] = useState(false);
  const compilationRequest = useRef(0);

  const loadCompilation = useCallback(async () => {
    const request = ++compilationRequest.current;
    const rows = await processesApi.listProcesses();
    if (request !== compilationRequest.current) return;
    const live = rows.find(row => Number(row.action) === ACCION_COMPILAR && !row.completed);
    setCompilando(live ? Number(live.id) : undefined);
  }, []);
  useEffect(() => () => { ++compilationRequest.current; }, []);

  const cargarModulos = useCallback(async () => {
    try {
      setModulos(await arsenalApi.listModules());
      setModuleError('');
    } catch (error) {
      setModuleError(messageFrom(error));
      throw error;
    }
  }, []);

  const cargar = useCallback(() => {
    arsenalApi.listCapabilities().then(setCapacidades).catch(status.reportError);
    void cargarModulos().catch(() => {});
    arsenalApi.listBlueprints().then(setDisenos).catch(status.reportError);
    arsenalApi
      .listPrograms()
      .then((lista) => {
        setProgramas(lista);
      })
      .catch(status.reportError);
    softwareApi.listSoftware().then(setFicheros).catch(status.reportError);
    researchApi.listResearch().then(setHistorial).catch(status.reportError);
    void loadCompilation().catch(() => {});
  }, [cargarModulos, loadCompilation]);

  useEffect(() => {
    cargar();
    rankingApi.listPlayers().then(setJugadores).catch(status.reportError);
    hardwareApi.listMachines().then(setMaquinas).catch(status.reportError);
  }, [cargar]);

  // Poll only live data for the visible desk. Catalogs/designs reload after actions.
  // Preserve previous data on failure, and never overlap slow polling batches.
  useEffect(() => {
    let active = true;
    let reading = false;
    const refresh = async () => {
      if (document.hidden || reading || !active) return;
      reading = true;
      const jobs: Promise<unknown>[] = [
        arsenalApi.listPrograms().then(rows => { if (active) setProgramas(rows); }),
        loadCompilation(),
      ];
      if (mesa === 'investigar') {
        jobs.push(softwareApi.listSoftware().then(rows => { if (active) setFicheros(rows); }));
        jobs.push(researchApi.listResearch().then(rows => { if (active) setHistorial(rows); }));
      }
      const results = await Promise.allSettled(jobs);
      if (active) {
        const failed = results.find(result => result.status === 'rejected');
        if (failed?.status === 'rejected') status.reportError(failed.reason);
      }
      reading = false;
    };
    const timer = window.setInterval(() => { void refresh(); }, 5000);
    const visible = () => { if (!document.hidden) void refresh(); };
    document.addEventListener('visibilitychange', visible);
    return () => { active = false; window.clearInterval(timer); document.removeEventListener('visibilitychange', visible); };
  }, [mesa, setProgramas, setFicheros, setHistorial, status.reportError, loadCompilation]);

  const nombreDeCapacidad = useMemo(
    () => new Map(capacidades.map((una) => [una.code, tr(una.name)])),
    [capacidades, idiomaActual()],
  );

  const compilar = async (diseno: Blueprint) => {
    if (compilando || startingCompilation) {
      return;
    }

    setStartingCompilation(true);
    try {
      const hecho = await arsenalApi.compile(diseno.id);
      ++compilationRequest.current;
      setCompilando(hecho.process_id);
      setCompilationHere({ id: hecho.process_id, blueprint: diseno.id, seconds: hecho.seconds });
      status.report(
        tr("Compilando «{0}» ({1} s). El fichero aparece en Discos al terminar.", hecho.name, hecho.seconds),
      );
      cargar();
    } catch (error) {
      status.reportError(error);
    } finally {
      setStartingCompilation(false);
    }
  };

  const investigar = async (id: number) => {
    try {
      const encolado = await researchApi.startResearch(id);
      setTarea(encolado.process_id);
      setResearchHere(id);
      status.report(tr("Investigando: tarea #{0} ({1} s).", encolado.process_id, encolado.seconds));
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  const investigables = ficheros.filter((uno) => !uno.folder);
  const localCompilation = compilationHere && (!compilando || compilationHere.id === compilando) ? compilationHere : undefined;

  return (
    <>
      <h2>Lab</h2>

      <CircuitRail onboarding={onboarding} />

      <nav className="mesas">
        {MESAS.map((una) => (
          <button
            key={una.id}
            type="button"
            data-guia={`lab.${una.id}`}
            className={mesa === una.id ? 'mesas__mesa is-current' : 'mesas__mesa'}
            onClick={() => setMesa(una.id)}
          >
            <b>{una.nombre}</b>
            <span>{una.que}</span>
          </button>
        ))}
      </nav>

      <StatusMessage text={status.message} as="block" />

      {mesa === 'crear' && (
        <>
          {moduleError && (
            <div role="alert" className="vacio">
              <p>{tr("No se pudieron cargar tus módulos:")} {moduleError}</p>
              <button type="button" onClick={() => void cargarModulos().catch(() => {})}>{tr("Reintentar carga de módulos")}</button>
            </div>
          )}
          <BlueprintForge capacidades={capacidades} modulos={modulos} onCreado={cargar} modulesUnavailable={!!moduleError} />

          <h3>{tr("Diseños guardados")}</h3>
          {compilando && (!localCompilation || !disenos.some(d => d.id === localCompilation.blueprint)) && <ProcessProgress processId={compilando} title={tr("Compilando programa")} onFinished={cargar} />}
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

                <StatSheet stats={diseno.projected} capability={capacidades.find((c) => c.code === diseno.capability)} />

                <div className="tarjeta__acciones">
                  {compilando ? (
                    <Tag estado="obra">{tr("hay una compilación en marcha")}</Tag>
                  ) : (
                    <Tag estado="plano" />
                  )}
                  <button
                    data-foco="lab.compilar"
                    className={compilar_.destacada ? 'es-foco' : undefined}
                    disabled={startingCompilation || !!compilando || !compilar_.permitida}
                    title={compilar_.motivo ?? undefined}
                    onClick={() => void compilar(diseno)}
                  >
                    {tr("Compilar")} </button>
                </div>
                {localCompilation?.blueprint === diseno.id && <ProcessProgress processId={localCompilation.id} initialSeconds={localCompilation.seconds} title={tr("Compilando «{0}»", diseno.name)} onFinished={cargar} />}
              </article>
            ))
          ) : (
            <div className="vacio">
              <div className="vacio__titulo">{tr("Ningún diseño todavía")}</div>
              <p className="vacio__texto">
                {tr("Un programa es una capacidad más los módulos que le pongas. Se diseña arriba, se compila aquí, y el fichero aparece en Discos.")} </p>
            </div>
          )}

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
                  {tarea && researchHere === fichero.id && <ProcessProgress processId={tarea} title={tr("Investigando «{0}»", fichero.name)} onFinished={cargar} />}
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

      {mesa === 'mejorar' && <ModuleWorkshop programs={programas} capabilities={capacidades} onChanged={cargar} />}

      {mesa === 'lanzar' && (
        <>
          <p className="muted">
            {tr("Solo dispara lo que esté cargado en memoria. Lo demás está en el disco, esperando.")} </p>

          <AntivirusPanel programas={programas} capacidades={capacidades} maquinas={maquinas} onFinished={cargar} />
          <OperationLauncher
            programas={programas}
            capacidades={capacidades}
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
