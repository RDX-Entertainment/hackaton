import { useCallback, useEffect, useMemo, useState } from 'react';

import { useAppSelector } from '../app/hooks';
import { BlueprintForge } from '../components/arsenal/BlueprintForge';
import { OperationLauncher } from '../components/arsenal/OperationLauncher';
import { StatSheet } from '../components/arsenal/StatSheet';
import { EmptyNotice } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import {
  arsenalApi,
  hardwareApi,
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

/**
 * El arsenal: el sistema modular de software, que el backend sirve bajo `/software-system/`.
 *
 * La pantalla sigue el camino que sigue el jugador y en ese orden: se lanza con lo que ya se
 * tiene, se diseña, se compila, y al fondo queda la referencia de lo que existe. Software
 * —la otra pantalla— es la que instala: aquí se construye, allí se carga en memoria.
 */
export function ArsenalPage() {
  const status = useStatusMessage();
  const yo = useAppSelector((state) => state.auth.user?.id ?? null);

  const [capacidades, setCapacidades] = useState<Capability[]>([]);
  const [modulos, setModulos] = useState<ResearchModule[]>([]);
  const [programas, setProgramas] = useState<Program[]>([]);
  const [disenos, setDisenos] = useState<Blueprint[]>([]);
  const [jugadores, setJugadores] = useState<RankedPlayer[]>([]);
  const [maquinas, setMaquinas] = useState<Machine[]>([]);

  const cargar = useCallback(() => {
    arsenalApi.listCapabilities().then(setCapacidades).catch(() => setCapacidades([]));
    arsenalApi.listModules().then(setModulos).catch(() => setModulos([]));
    arsenalApi.listPrograms().then(setProgramas).catch(() => setProgramas([]));
    arsenalApi.listBlueprints().then(setDisenos).catch(() => setDisenos([]));
    rankingApi.listPlayers().then(setJugadores).catch(() => setJugadores([]));
    hardwareApi.listMachines().then(setMaquinas).catch(() => setMaquinas([]));
  }, []);

  useEffect(() => {
    cargar();
  }, [cargar]);

  /**
   * Lo que el servidor mira al arrancar una operación: la suma de la flota, no la máquina
   * concreta que ejecuta. Se suma igual aquí para que lo que se enseña y lo que se comprueba
   * sean lo mismo, aunque sumar la flota entera sea discutible.
   */
  const capacidad = useMemo(
    () =>
      maquinas.reduce(
        (total, maquina) => ({
          cpu: total.cpu + maquina.cpu,
          ram: total.ram + maquina.ram,
          net: total.net + maquina.net,
        }),
        { cpu: 0, ram: 0, net: 0 },
      ),
    [maquinas],
  );

  const nombreDeCapacidad = useMemo(
    () => new Map(capacidades.map((una) => [una.code, una.name])),
    [capacidades],
  );

  const compilar = async (diseno: Blueprint) => {
    try {
      const hecho = await arsenalApi.compile(diseno.id);
      status.report(
        `«${hecho.name}» compilado como fichero #${hecho.software_id}. ` +
          'Instálalo desde Software para poder usarlo.',
      );
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Arsenal</h2>

      <p className="muted">
        Un programa es una capacidad más los módulos que le hayas puesto. Se diseña, se
        compila y se instala; solo entonces dispara.
      </p>

      <section className="arsenal__block">
        <h3>Operación</h3>
        <OperationLauncher
          programas={programas}
          jugadores={jugadores}
          yo={yo}
          onLanzada={cargar}
        />
      </section>

      <section className="arsenal__block">
        <h3>Lo compilado</h3>
        <StatusMessage text={status.message} as="block" />

        {programas.length ? (
          programas.map((programa) => (
            <article className="forum-post" key={programa.software_id}>
              <b>
                {programa.name} v{programa.version}
              </b>
              <small>
                {' '}
                · {programa.capability_name ?? 'sin capacidad'}
                {programa.action_kind && ` · ${CONSECUENCIA[programa.action_kind] ?? programa.action_kind}`}
                {' · '}
                {programa.running ? 'en memoria' : 'solo en disco'}
              </small>

              {!programa.running && (
                <p className="muted">Instálalo desde Software o no podrá usarse.</p>
              )}

              <StatSheet stats={programa.stats} capacidad={capacidad} />
            </article>
          ))
        ) : (
          <EmptyNotice>Todavía no has compilado nada.</EmptyNotice>
        )}
      </section>

      <section className="arsenal__block">
        <h3>Diseños</h3>

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

            <StatSheet stats={diseno.projected} capacidad={capacidad} />

            <button onClick={() => void compilar(diseno)}>Compilar</button>
          </article>
        ))}

        {!disenos.length && <EmptyNotice>Ningún diseño guardado todavía.</EmptyNotice>}

        <BlueprintForge capacidades={capacidades} modulos={modulos} onCreado={cargar} />
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
                <td>{una.domain}</td>
                <td>{una.name}</td>
                <td>{una.role === 'defender' ? 'defensa' : una.role === 'attacker' ? 'ataque' : 'apoyo'}</td>
                <td>{CONSECUENCIA[una.action_kind] ?? una.action_kind}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </>
  );
}
