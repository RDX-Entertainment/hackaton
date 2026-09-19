import { tr, useIdioma } from '../../i18n/text';
import { useState } from 'react';

import type { MachineDetail } from '../../utils/api';
import { formatIp } from '../../utils/ip';

type Props = {
  detail: MachineDetail;
  machines: MachineDetail[];
  busy: boolean;
  onChange: (id: number) => void;
};

export function GatewayControl({ detail, machines, busy, onChange }: Props) {
  useIdioma();
  const [chosen, setChosen] = useState('');
  const eligible = ({ machine, components }: MachineDetail) =>
    machine.node_kind === 'computer' && machine.operational && machine.public_ip !== null &&
    components.some((part) => part.kind === 'net');
  const alternatives = machines.filter((item) => !item.machine.is_gateway && eligible(item));
  const target = alternatives.find((item) => String(item.machine.id) === chosen) ?? alternatives[0];
  const { machine } = detail;

  return (
    <section className="panel gateway-control" aria-label={tr("Máquina principal")}>
      {machine.is_gateway ? (
        <>
          <b>{tr("★ Máquina principal")}</b>
          <p>{tr("Por este equipo te alcanzan. Antes de quitar su última tarjeta de red, nombra principal a otro.")}</p>
          {target ? (
            <div className="gateway-control__actions">
              <label>{tr("Otra máquina principal")} <select value={target.machine.id} disabled={busy} onChange={(event) => setChosen(event.target.value)}>
                  {alternatives.map(({ machine: candidate }) => (
                    <option key={candidate.id} value={candidate.id}>{candidate.name} · {formatIp(candidate.public_ip)}</option>
                  ))}
                </select>
              </label>
              <button disabled={busy} onClick={() => onChange(target.machine.id)}>{tr("Nombrar principal a esta máquina")}</button>
            </div>
          ) : <p>{tr("No hay otro equipo apto todavía: prepara uno activo con una tarjeta de red instalada.")}</p>}
        </>
      ) : (
        <>
          <button disabled={busy || !eligible(detail)} onClick={() => onChange(machine.id)}>{tr("Nombrar como principal")}</button>
          {!eligible(detail) && <p>{tr("Necesita estar activo y tener una tarjeta de red con salida pública.")}</p>}
        </>
      )}
      <small>{tr("Al cambiar, la IP pública del nuevo principal será la IP de tu cuenta. Los equipos conservan sus IP y archivos.")}</small>
    </section>
  );
}
