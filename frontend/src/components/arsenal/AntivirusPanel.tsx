import { tr, useIdioma } from '../../i18n/text';
import { useState } from 'react';
import { Link } from 'react-router-dom';
import { arsenalApi, type Capability, type Machine, type Program } from '../../utils/api';
import { useStatusMessage } from '../../utils/useStatusMessage';
import { ProcessProgress } from '../ProcessProgress';
import { StatusMessage } from '../StatusMessage';
import { StatSheet } from './StatSheet';

export function AntivirusPanel({ programas, capacidades, maquinas, onFinished }: {
  programas: Program[]; capacidades: Capability[]; maquinas: Machine[]; onFinished: () => void;
}) {
  useIdioma();
  const [selected, setSelected] = useState<number>();
  const [pending, setPending] = useState(false);
  const [process, setProcess] = useState<number>();
  const status = useStatusMessage();
  const programs = programas.filter((p) => p.running && (p.functions ?? [p.capability]).some(f => ['antivirus','antivirus_scan','antivirus_clean'].includes(f ?? '')));
  const program = programs.find((p) => p.software_id === selected) ?? programs[0];
  const fits = !!program?.resources && program.resources.cpu >= program.stats.cpu
    && program.resources.ram >= program.stats.ram && program.resources.net >= program.stats.net;

  async function run(mode: 'scan' | 'clean') {
    if (!program || pending || !fits) return;
    setPending(true);
    try {
      const result = await arsenalApi.runAntivirus(program.software_id, mode);
      setProcess(result.process_id);
      status.report(tr("{0} en curso · {1} s. El resultado aparecerá al terminar.", mode === 'scan' ? tr("Análisis") : tr("Limpieza"), result.seconds));
    } catch (error) { status.reportError(error); }
    finally { setPending(false); }
  }

  return <section className="compose antivirus-panel" aria-label={tr("Antivirus")}>
    <h3>{tr("Antivirus · buscar y eliminar")}</h3>
    <p className="muted">{tr("Detección contra sigilo para encontrar virus. Limpieza contra persistencia para eliminarlos. Solo actúa en el equipo del antivirus; no borra tus diseños de malware originales.")}</p>
    {!program ? <p>{tr("Crea un Antivirus o instala un módulo de detección/limpieza, y")} <Link to="/software">{tr("cárgalo en memoria en Software")}</Link>.</p> : <>
      <label>{tr("Antivirus y equipo")} <select value={program.software_id} onChange={(e) => setSelected(Number(e.target.value))}>
          {programs.map((p) => <option key={p.software_id} value={p.software_id}>{p.name} · {maquinas.find((m) => m.id === p.server_id)?.name ?? tr("Equipo #{0}", p.server_id)}</option>)}
        </select>
      </label>
      <StatSheet stats={program.stats} capability={capacidades.find((c) => c.code === program.capability)} extraAttributes={(program.functions ?? [program.capability]).flatMap(f => f === 'antivirus_scan' ? ['detection'] : f === 'antivirus_clean' ? ['removal'] : [])} capacidad={program.resources ?? undefined} />
      {!fits && <p role="status">{tr("El equipo no dispone de recursos suficientes o está desactivado.")}</p>}
      <div className="tarjeta__acciones">
        <button type="button" disabled={pending || !fits || !(program.functions ?? [program.capability]).some(f => f === 'antivirus' || f === 'antivirus_scan')} onClick={() => void run('scan')}>{tr("Buscar virus · detección")} {program.stats.detection}</button>
        <button type="button" disabled={pending || !fits || !(program.functions ?? [program.capability]).some(f => f === 'antivirus' || f === 'antivirus_clean')} onClick={() => void run('clean')}>{tr("Eliminar detectados · limpieza")} {program.stats.removal}</button>
      </div>
    </>}
    <StatusMessage text={status.message} />
    {process && <ProcessProgress key={process} processId={process} onFinished={onFinished} />}
  </section>;
}
