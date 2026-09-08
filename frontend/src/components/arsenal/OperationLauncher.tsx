import { useMemo, useState, type FormEvent } from 'react';
import { Link } from 'react-router-dom';
import { arsenalApi, type Program, type RankedPlayer, type Machine } from '../../utils/api';
import { useStatusMessage } from '../../utils/useStatusMessage';
import { EmptyNotice } from '../Notice';
import { ProcessProgress } from '../ProcessProgress';
import { StatusMessage } from '../StatusMessage';
import { StatSheet } from './StatSheet';

type Props = {
  programas: Program[];
  jugadores: RankedPlayer[];
  maquinas: Machine[];
  yo: number | null;
  onLanzada: () => void;
};

export function OperationLauncher({ programas, jugadores, maquinas, yo, onLanzada }: Props) {
  const status = useStatusMessage();
  const [selected, setSelected] = useState('');
  const [target, setTarget] = useState('');
  const [processId, setProcessId] = useState<number>();
  const [pending, setPending] = useState(false);
  const weapons = useMemo(() => programas.filter((program) => program.running
    && program.role === 'attacker' && program.capability
    && ['intrude', 'infect'].includes(program.action_kind ?? '')), [programas]);
  const rivals = jugadores.filter((player) => player.user_id !== yo);
  const weapon = weapons.find((program) => String(program.software_id) === selected) ?? weapons[0];
  const opponent = rivals.find((player) => String(player.user_id) === target) ?? rivals[0];
  const machine = maquinas.find((item) => item.id === weapon?.server_id);
  const resources = weapon?.resources;
  const fits = !!weapon && !!resources && resources.cpu >= weapon.stats.cpu
    && resources.ram >= weapon.stats.ram && resources.net >= weapon.stats.net;

  const launch = async (event: FormEvent) => {
    event.preventDefault();
    if (pending || !weapon?.capability || !opponent || !fits) return;
    setPending(true);
    try {
      const started = await arsenalApi.startOperation(weapon.capability, opponent.user_id, weapon.software_id);
      setProcessId(started.process_id);
      status.report(`Operación #${started.process_id} en curso · ${started.seconds} s · probabilidad ${started.estimated_chance}.`);
      onLanzada();
    } catch (error) {
      status.reportError(error);
    } finally {
      setPending(false);
    }
  };

  if (!weapons.length) return <EmptyNotice>Necesitas un programa de intrusión o infección en memoria. Compila un diseño e <Link to="/software">instálalo en Software</Link>.</EmptyNotice>;
  if (!opponent) return <EmptyNotice>No hay otro jugador contra el que operar todavía.</EmptyNotice>;

  return <form className="compose arsenal__operation" onSubmit={launch}>
    <label>Programa y origen
      <select value={weapon.software_id} onChange={(event) => setSelected(event.target.value)}>
        {weapons.map((program) => <option key={program.software_id} value={program.software_id}>
          {program.name} · {program.capability_name} · {maquinas.find((item) => item.id === program.server_id)?.name ?? `Equipo #${program.server_id}`}
        </option>)}
      </select>
    </label>
    <label>Objetivo · máquina principal
      <select value={opponent.user_id} onChange={(event) => setTarget(event.target.value)}>
        {rivals.map((player) => <option key={player.user_id} value={player.user_id}>{player.login}</option>)}
      </select>
    </label>
    <p className="muted">Sale de {machine?.name ?? `equipo #${weapon.server_id}`}. Los recursos incluyen su cluster, si pertenece a uno.</p>
    <StatSheet stats={weapon.stats} capacidad={resources ?? undefined} />
    {!fits && <p role="status">El equipo no dispone de recursos suficientes para ejecutar este programa.</p>}
    {weapon.action_kind === 'infect' && <p className="muted">Requiere acceso previo a la máquina objetivo y espacio en su disco.</p>}
    <button disabled={pending || !fits}>{pending ? 'Lanzando…' : 'Lanzar operación'}</button>
    <StatusMessage text={status.message} />
    {processId && <ProcessProgress processId={processId} onFinished={onLanzada} />}
  </form>;
}
