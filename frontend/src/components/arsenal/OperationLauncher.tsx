import { useMemo, useState, type FormEvent } from 'react';

import { arsenalApi, type Program, type RankedPlayer } from '../../utils/api';
import { useStatusMessage } from '../../utils/useStatusMessage';
import { EmptyNotice } from '../Notice';
import { ProcessProgress } from '../ProcessProgress';
import { StatusMessage } from '../StatusMessage';

type OperationLauncherProps = {
  /** Todo el arsenal; aquí se filtra lo que de verdad puede disparar. */
  programas: Program[];
  jugadores: RankedPlayer[];
  /** Id del jugador, para no ofrecerse a sí mismo como objetivo. */
  yo: number | null;
  onLanzada: () => void;
};

/**
 * Lanzar una operación.
 *
 * El servidor exige tres cosas y las tres se comprueban aquí antes de dejar pulsar: un
 * programa atacante **cargado en memoria**, un objetivo que no seas tú, y hardware que
 * aguante el consumo. Lo tercero lo sigue decidiendo el servidor con la máquina delante.
 */
export function OperationLauncher({ programas, jugadores, yo, onLanzada }: OperationLauncherProps) {
  const status = useStatusMessage();

  const [capacidad, setCapacidad] = useState('');
  const [objetivo, setObjetivo] = useState('');
  const [processId, setProcessId] = useState<number>();

  // Una operación sale de un programa corriendo, y solo los atacantes salen a por alguien.
  const armas = useMemo(
    () => programas.filter((uno) => uno.running && uno.role === 'attacker' && uno.capability),
    [programas],
  );

  // Dos copias de la misma capacidad no son dos entradas del selector: el servidor dispara
  // una sola, la de mayor eficacia + penetración de las que corren. Se elige aquí con ese
  // mismo criterio para que el nombre que se enseña sea el del programa que va a salir.
  const disponibles = useMemo(() => {
    const mejores = new Map<string, Program>();

    for (const arma of armas) {
      const code = arma.capability as string;
      const anterior = mejores.get(code);

      if (!anterior || fuerza(arma) > fuerza(anterior)) {
        mejores.set(code, arma);
      }
    }

    return [...mejores.values()];
  }, [armas]);

  const rivales = useMemo(
    () => jugadores.filter((jugador) => jugador.user_id !== yo),
    [jugadores, yo],
  );

  const elegida = capacidad || disponibles[0]?.capability || '';
  const arma = disponibles.find((uno) => uno.capability === elegida);
  const contra = objetivo || String(rivales[0]?.user_id ?? '');

  const lanzar = async (event: FormEvent) => {
    event.preventDefault();

    try {
      const empezada = await arsenalApi.startOperation(elegida, Number(contra));
      setProcessId(empezada.process_id);
      status.report(
        `Encolada como tarea #${empezada.process_id}; termina en ${empezada.seconds} s. ` +
          'El resultado sale en Procesos cuando el worker la resuelva.',
      );
      onLanzada();
    } catch (error) {
      status.reportError(error);
    }
  };

  if (!disponibles.length) {
    return (
      <EmptyNotice>
        No tienes ningún programa de ataque cargado en memoria. Compila un diseño aquí abajo y
        luego instálalo desde Software: un fichero en el disco no dispara.
      </EmptyNotice>
    );
  }

  if (!rivales.length) {
    return <EmptyNotice>No hay otro jugador contra el que operar todavía.</EmptyNotice>;
  }

  return (
    <form className="compose" onSubmit={lanzar}>
      <label>
        Capacidad
        <select value={elegida} onChange={(event) => setCapacidad(event.target.value)}>
          {disponibles.map((uno) => (
            <option key={uno.software_id} value={uno.capability as string}>
              {uno.capability_name} · {uno.name} v{uno.version}
            </option>
          ))}
        </select>
      </label>

      <label>
        Objetivo
        <select value={contra} onChange={(event) => setObjetivo(event.target.value)}>
          {rivales.map((jugador) => (
            <option key={jugador.user_id} value={jugador.user_id}>
              {jugador.login}
            </option>
          ))}
        </select>
      </label>

      <button>Lanzar</button>
      <StatusMessage text={status.message} />
      {processId && <ProcessProgress processId={processId} onFinished={onLanzada} />}

      {arma?.action_kind === 'infect' && (
        <p className="muted">
          Infectar exige haber entrado antes en esa máquina. Si no está abierta, la operación
          se resolverá como fallida.
        </p>
      )}
    </form>
  );
}

/** El desempate del servidor entre dos copias de la misma capacidad. */
function fuerza(programa: Program): number {
  return programa.stats.effectiveness + programa.stats.penetration;
}
