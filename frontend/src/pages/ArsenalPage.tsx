import { useCallback, useEffect, useMemo, useState } from 'react';
import { Link } from 'react-router-dom';

import { useAppSelector } from '../app/hooks';
import { BlueprintForge } from '../components/arsenal/BlueprintForge';
import { OperationLauncher } from '../components/arsenal/OperationLauncher';
import { StatSheet } from '../components/arsenal/StatSheet';
import { EmptyNotice } from '../components/Notice';
import { Tag } from '../components/Tag';
import { CircuitRail } from '../components/onboarding/CircuitRail';
import { useOnboarding } from '../components/onboarding/useOnboarding';
import { ProcessProgress } from '../components/ProcessProgress';
import { StatusMessage } from '../components/StatusMessage';
import {
  arsenalApi,
  hardwareApi,
  processesApi,
  rankingApi,
  type Blueprint,
  type Capability,
  type Machine,
  type Program,
  type RankedPlayer,
  type ResearchModule,
} from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

/** Cómo se lee un `action_kind` en pantalla. */
const CONSECUENCIA: Record<string, string> = {
  intrude: 'entra en la máquina',
  infect: 'deja algo dentro',
  block: 'cierra la puerta',
  detect: 'se entera',
  remove: 'echa lo que hay dentro',
};

const SKILL_LABELS: Record<string, string> = {
  modify_web: 'Modificar página', hide_evidence: 'Ocultar evidencia',
  forge_evidence: 'Falsificar evidencia', analyze_evidence: 'Analizar evidencia',
  install_persistence: 'Instalar persistencia', remove_persistence: 'Eliminar persistencia',
  escalate_privileges: 'Elevar privilegios',
  manipulation: 'Manipulación', anti_forensics: 'Anti-forense', forensics: 'Forense',
  persistence: 'Persistencia', removal: 'Limpieza', penetration: 'Penetración',
};

/**
 * El arsenal: el sistema modular de software, que el backend sirve bajo `/software-system/`.
 *
 * La pantalla va en el orden del circuito —se diseña, se compila, y con lo compilado se
 * lanza—, y al fondo queda la referencia de lo que existe. Abría por «Operación», que es el
 * último paso: quien llegaba sin nada compilado se encontraba primero un lanzador vacío y
 * tenía que bajar para descubrir por dónde se empieza. Software —la otra pantalla— es la que
 * instala: aquí se construye, allí se carga en memoria.
 */
export function ArsenalPage() {
  const status = useStatusMessage();
  const { onboarding } = useOnboarding();
  const yo = useAppSelector((state) => state.auth.user?.id ?? null);

  const [capacidades, setCapacidades] = useState<Capability[]>([]);
  const [modulos, setModulos] = useState<ResearchModule[]>([]);
  const [programas, setProgramas] = useState<Program[]>([]);
  const [disenos, setDisenos] = useState<Blueprint[]>([]);
  const [jugadores, setJugadores] = useState<RankedPlayer[]>([]);
  const [maquinas, setMaquinas] = useState<Machine[]>([]);
  const [compilacionActiva, setCompilacionActiva] = useState<{ blueprintId?: number; processId: number }>();
  const [compiling, setCompiling] = useState(false);
  const [machineFilter, setMachineFilter] = useState('');
  const [loadError, setLoadError] = useState('');

  const cargar = useCallback(() => {
    arsenalApi.listCapabilities().then(setCapacidades).catch(() => setCapacidades([]));
    arsenalApi.listModules().then(setModulos).catch(() => setModulos([]));
    arsenalApi.listPrograms().then((programs) => { setProgramas(programs); setLoadError(''); }).catch(() => setLoadError('No se pudo cargar el arsenal. Intenta actualizar.'));
    arsenalApi.listBlueprints().then(setDisenos).catch(() => setDisenos([]));
    rankingApi.listPlayers().then(setJugadores).catch(() => setJugadores([]));
    hardwareApi.listMachines().then(setMaquinas).catch(() => setMaquinas([]));
    processesApi.listProcesses().then((rows) => {
      const active = rows.find((row) => Number(row.action) === 91 && !row.completed);
      if (active) setCompilacionActiva((current) => current?.processId === Number(active.id) ? current : { processId: Number(active.id) });
    }).catch(() => {});
  }, []);

  useEffect(() => {
    cargar();
    const timer = window.setInterval(cargar, 5000);
    return () => window.clearInterval(timer);
  }, [cargar]);

  const visiblePrograms = programas.filter((program) => !machineFilter || String(program.server_id) === machineFilter);

  const nombreDeCapacidad = useMemo(
    () => new Map(capacidades.map((una) => [una.code, una.name])),
    [capacidades],
  );

  const compilar = async (diseno: Blueprint) => {
    if (compiling || compilacionActiva) return;
    setCompiling(true);
    try {
      const hecho = await arsenalApi.compile(diseno.id);
      setCompilacionActiva({ blueprintId: diseno.id, processId: hecho.process_id });
      status.report(
        `Compilación de «${hecho.name}» encolada (${hecho.seconds} s). ` +
          'El fichero aparecerá al terminar; después instálalo desde Software.',
      );
      cargar();
    } catch (error) {
      status.reportError(error);
    } finally {
      setCompiling(false);
    }
  };

  return (
    <>
      <h2>Arsenal</h2>
      <div className="arsenal__summary">
        <span><b>{programas.length}</b> programas</span>
        <span><b>{programas.filter((program) => program.running).length}</b> en memoria</span>
        <span><b>{disenos.length}</b> diseños</span>
        <Link to="/software">Gestionar discos y memoria →</Link>
      </div>
      <CircuitRail onboarding={onboarding} />

      {loadError && <p role="alert">{loadError} <button onClick={cargar}>Actualizar</button></p>}

      <p className="muted">
        Un programa es una capacidad más los módulos que le hayas puesto. Se diseña, se
        compila y se instala; solo entonces dispara.
      </p>

      <section className="arsenal__block">
        <h3>Diseños</h3>
        {compilacionActiva && <ProcessProgress processId={compilacionActiva.processId} onFinished={(process) => {
          status.report(process.result_message || 'La compilación ha terminado.');
          setCompilacionActiva(undefined);
          cargar();
        }} />}

        {disenos.map((diseno) => (
          <article className="forum-post" key={diseno.id}>
            <b>
              {diseno.name} r{diseno.revision}
            </b>
            <small>
              {' '}
              · {nombreDeCapacidad.get(diseno.capability) ?? diseno.capability} ·{' '}
              {diseno.modules.length} módulo{diseno.modules.length === 1 ? '' : 's'}
            </small>

            <StatSheet stats={diseno.projected} />

            <button disabled={compiling || !!compilacionActiva} onClick={() => void compilar(diseno)}>Compilar</button>
          </article>
        ))}

        {!disenos.length && <EmptyNotice>Ningún diseño guardado todavía.</EmptyNotice>}

        <BlueprintForge capacidades={capacidades} modulos={modulos} onCreado={cargar} />
      </section>

      <section className="arsenal__block">
        <h3>Lo compilado</h3>
        <label className="arsenal__filter">Equipo
          <select value={machineFilter} onChange={(event) => setMachineFilter(event.target.value)}>
            <option value="">Todos los equipos</option>
            {maquinas.map((machine) => <option key={machine.id} value={machine.id}>{machine.name}</option>)}
          </select>
        </label>
        <StatusMessage text={status.message} as="block" />

        {visiblePrograms.length ? (
          visiblePrograms.map((programa) => (
            <article className="forum-post arsenal__program" key={programa.software_id}>
              <b>
                {programa.name} v{programa.version}
              </b>
              <small>
                {' '}
                · {programa.capability_name ?? 'sin capacidad'}
                {programa.action_kind && ` · ${CONSECUENCIA[programa.action_kind] ?? programa.action_kind}`}
                {' · '}{maquinas.find((machine) => machine.id === programa.server_id)?.name ?? `Equipo #${programa.server_id}`}
              </small>

              <div className="arsenal__estado">
                <Tag estado={programa.running ? 'memoria' : 'disco'} />
              </div>

              {!programa.running && (
                <p className="muted">Instálalo desde Software o no podrá usarse.</p>
              )}

              <StatSheet stats={programa.stats} capacidad={programa.resources ?? undefined} />
              {!!programa.skills.length && <ul className="arsenal__skills">{programa.skills.map((skill) => <li key={skill.action} className={skill.unlocked ? 'is-ready' : ''}>
                {skill.unlocked ? 'Disponible' : 'Bloqueada'} · {SKILL_LABELS[skill.action] ?? skill.action} · {SKILL_LABELS[skill.attribute] ?? skill.attribute} ≥ {skill.threshold}
              </li>)}</ul>}
            </article>
          ))
        ) : (
          <EmptyNotice>No hay programas en esta selección.</EmptyNotice>
        )}
      </section>


      <section className="arsenal__block">
        <h3>Operación</h3>
        <OperationLauncher
          programas={programas}
          jugadores={jugadores}
          maquinas={maquinas}
          yo={yo}
          onLanzada={cargar}
        />
      </section>


      <section className="arsenal__block">
        <h3>Módulos investigados</h3>

        {modulos.length ? (
          <ul className="arsenal__modules">
            {modulos.map((pieza) => (
              <li key={pieza.id}>
                <b>{pieza.name}</b>
                <small>
                  {' '}
                  · {pieza.capability ? nombreDeCapacidad.get(pieza.capability) ?? pieza.capability : 'cualquier capacidad'}
                </small>
              </li>
            ))}
          </ul>
        ) : (
          <EmptyNotice>Sin módulos. Sin ellos solo se compilan programas pelados.</EmptyNotice>
        )}
      </section>

      <section className="arsenal__block">
        <h3>Capacidades que existen</h3>

        <table className="arsenal__caps">
          <thead>
            <tr>
              <th>Dominio</th>
              <th>Capacidad</th>
              <th>Lado</th>
              <th>Si gana</th>
            </tr>
          </thead>
          <tbody>
            {capacidades.map((una) => (
              <tr key={una.code}>
                <td data-label="Dominio">{una.domain}</td>
                <td data-label="Capacidad">{una.name}</td>
                <td data-label="Lado">{una.role === 'defender' ? 'defensa' : una.role === 'attacker' ? 'ataque' : 'apoyo'}</td>
                <td data-label="Si gana">{CONSECUENCIA[una.action_kind] ?? una.action_kind}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </>
  );
}
