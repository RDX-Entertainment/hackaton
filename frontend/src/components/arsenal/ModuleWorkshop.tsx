import { tr, useIdioma } from '../../i18n/text';
import { memo, useCallback, useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { type Capability, type Program, type ProgramStats } from '../../utils/api';
import { equipModule, unequipModule, fuseModule, loadWorkshop, moduleDelta, multipliers, type Workshop, type ModuleTemplate } from '../../utils/api/workshop';
import { useStatusMessage } from '../../utils/useStatusMessage';
import { StatusMessage } from '../StatusMessage';
import { Icono } from '../iconos/Icono';
import { StatSheet } from './StatSheet';
import { visibleAttributes } from './attributes';
import '../../styles/arsenal/modules.css';

export const ModuleWorkshop = memo(function ModuleWorkshop({ programs, capabilities, onChanged }: { programs: Program[]; capabilities: Capability[]; onChanged: () => void }) {
  useIdioma();
  const [data, setData] = useState<Workshop>();
  const [selected, setSelected] = useState<number>();
  const [slot, setSlot] = useState(1);
  const [choice, setChoice] = useState('');
  const [busy, setBusy] = useState(false);
  const status = useStatusMessage();
  const reload = useCallback(async () => { setData(await loadWorkshop()); }, []);
  useEffect(() => { void reload().catch(status.reportError); }, [reload, status.reportError]);
  const program = programs.find(p => p.software_id === selected) ?? programs[0];
  const installed = data?.equipped.filter(m => m.software_id === program?.software_id) ?? [];
  const selectedStock = data?.stock.find(s => `${s.module_id}:${s.tier}` === choice);
  const candidate = data?.catalog.find(m => m.id === selectedStock?.module_id);
  function reason(m: ModuleTemplate): string {
    if (!program) return tr("Compila un programa primero");
    const c=m.config;
    if (!c.active) return tr("Desactivado por administración");
    if (program.version<c.min_level) return tr("Requiere nivel {0}", c.min_level);
    if (c.allowed_capabilities.length && !c.allowed_capabilities.includes(program.capability ?? '')) return tr("Otra capacidad de destino");
    if (!c.allowed_slots.includes(slot)) return tr("Solo ranuras {0}", c.allowed_slots.join(', '));
    if (installed.filter(i => i.module_id === m.id).length>=c.max_copies) return tr("Máximo {0} por programa", c.max_copies);
    if (installed.some(i => c.excludes.includes(i.snapshot.code) || i.snapshot.excludes.includes(m.code))) return tr("Conflicto con un módulo instalado");
    return '';
  }
  const available = data?.stock.filter(s => s.quantity > 0) ?? [];
  const occupied = installed.some(m => m.slot === slot);
  const preview: ProgramStats | undefined = program && { ...program.stats };
  if (preview && candidate && selectedStock && !reason(candidate) && !occupied) {
    for (const [key,value] of Object.entries(candidate.config.deltas)) {
      const k=key as keyof ProgramStats;
      const delta=Math.sign(value!) * Math.round(Math.abs(value!) * multipliers[selectedStock.tier]);
      const cost=['cpu','ram','net','weight'].includes(key);
      preview[k]=Math.max(cost ? 1 : 0,Math.min(cost ? Infinity : 100,preview[k]+delta));
    }
  }
  const grants=[...(program?.functions ?? []),...(!occupied && candidate && !reason(candidate) ? candidate.config.grants : [])];
  const extra=grants.flatMap(g => g==='antivirus_scan' ? ['detection'] : g==='antivirus_clean' ? ['removal'] : visibleAttributes(capabilities.find(c => c.code===g)));
  const functionName=(g:string)=>g==='antivirus_scan'?tr("Detectar virus"):g==='antivirus_clean'?tr("Eliminar virus"):capabilities.find(c=>c.code===g)?.name ?? g;
  async function act(work:()=>Promise<unknown>, message:string) {
    if (busy) return;
    setBusy(true);
    try { await work(); setChoice(''); await reload(); onChanged(); status.report(message); }
    catch (e) { status.reportError(e); }
    finally { setBusy(false); }
  }
  return <section className="compose module-workshop">
    <h3>{tr("Módulos · taller de capacidades")}</h3>
    <p className="muted">{tr("Investiga el programa para desbloquear ranuras en niveles 1, 3, 6, 10, 15 y 21. Tres copias libres del mismo módulo y grado se fusionan en una superior.")}</p>
    <StatusMessage text={status.message} as="block" />
    {!data ? <button onClick={() => void reload().catch(status.reportError)}>{tr("Cargar inventario")}</button> : <>
      {program ? <>
        <label>{tr("Programa")}<select value={program.software_id} onChange={e => { setSelected(Number(e.target.value)); setSlot(1); setChoice(''); }}>{programs.map(p => <option key={p.software_id} value={p.software_id}>{p.name} {tr("· nivel")} {p.version} · {p.module_slots} {tr("ranuras")}</option>)}</select></label>
        <div className="aforge__workbench">
          <div className="aforge__design">
            <div className="aforge__hologram"><div className="aforge__orb" aria-hidden="true"><Icono nombre="memoria" tamano={88} /></div><strong>{program.name}</strong><span>{program.capability_name} {tr("· nivel")} {program.version}</span></div>
            <p>{program.running ? <>{tr("Retíralo de memoria en")} <Link to="/software">Software</Link> {tr("antes de cambiar módulos.")}</> : tr("En disco · listo para modificar si no tiene procesos pendientes.")}</p>
            <div className="module-slots">{data.slot_levels.map((level,i) => {
              const item=installed.find(m => m.slot===i+1);
              return <button type="button" className={slot===i+1?'is-selected':''} key={level} disabled={busy || program.module_slots<i+1} onClick={() => { setSlot(i+1); setChoice(''); }}><b>{tr("Ranura")} {i+1}</b><span>{program.module_slots<i+1 ? tr("Bloqueada · nivel {0}", level) : item ? `${item.snapshot.name} · ${data.grades[item.tier]}` : tr("Disponible")}</span></button>;
            })}</div>
            {installed.filter(m => m.slot===slot).map(m => <div className="module-card" key={m.slot}><b>{m.snapshot.name} · {data.grades[m.tier]}</b><p>{moduleDelta(m.snapshot.deltas)}</p><p>{m.snapshot.grants.map(functionName).join(' · ')}</p><button disabled={busy || program.running} onClick={() => void act(() => unequipModule(program.software_id,slot),tr("Módulo retirado y devuelto al inventario."))}>{tr("Retirar módulo")}</button></div>)}
            {!occupied && <><label>{tr("Módulo para la ranura")} {slot}<select value={choice} onChange={e => setChoice(e.target.value)}><option value="">{tr("Selecciona una copia libre")}</option>{available.map(s => {
              const m=data.catalog.find(m=>m.id===s.module_id)!;
              return <option key={`${s.module_id}:${s.tier}`} value={`${s.module_id}:${s.tier}`} disabled={!!reason(m)}>{m.name} · {data.grades[s.tier]} ×{s.quantity}{reason(m) ? ` — ${reason(m)}` : ''}</option>;
            })}</select></label>
            {candidate && selectedStock && <p>{moduleDelta(candidate.config.deltas,selectedStock.tier)}<br />{candidate.config.grants.map(functionName).join(' · ')}</p>}
            <button disabled={busy || program.running || !candidate || !!reason(candidate)} onClick={() => selectedStock && void act(() => equipModule(program.software_id,slot,selectedStock.module_id,selectedStock.tier),tr("Módulo instalado. Sus atributos y funciones ya se aplican."))}>{tr("Instalar módulo")}</button></>}
          </div>
          <aside className="aforge__preview"><h4>{candidate && !occupied ? tr("Vista previa al instalar") : tr("Estadísticas efectivas")}</h4>{preview && <StatSheet stats={preview} capability={capabilities.find(c=>c.code===program.capability)} extraAttributes={extra} capacidad={program.resources ?? undefined} />}<p>{tr("Funciones:")} {[...new Set(grants)].map(functionName).join(' · ')}</p><p className="muted">{tr("Los costes también crecen con el grado. Los atributos se limitan a 100. Los perfiles y diseños anteriores conservan sus mejoras integradas.")}</p></aside>
        </div>
      </> : <p>{tr("Crea y compila un programa para instalar módulos. Los encargos completados entregan nuevas copias.")}</p>}
      <div className="module-collector"><h4>{tr("Cómo conseguir módulos")}</h4><p>{tr("Completa encargos: cada objetivo pagado entrega un módulo, y los de mayor nivel pueden entregar grados superiores.")}</p></div>
      <h4>{tr("Inventario y fusión")}</h4><p className="muted">{tr("Estándar → Optimizado → Experimental → Clasificado → Zero-day. La fusión consume 3 y entrega 1; no se puede deshacer. Las copias instaladas no se consumen.")}</p>
      <div className="module-inventory">{available.map(s => {
        const m=data.catalog.find(m=>m.id===s.module_id)!;
        return <article className={`module-card grade-${s.tier}`} key={`${s.module_id}:${s.tier}`}><b>{m.name}</b><span>{data.grades[s.tier]} · {s.quantity} {tr("libres")}</span><p>{moduleDelta(m.config.deltas,s.tier)}</p><p>{m.config.grants.map(functionName).join(' · ') || tr("Modificador de atributos")}</p><small>{tr("Nivel")} {m.config.min_level} {tr("· ranuras")} {m.config.allowed_slots.join(', ')} {tr("· máx.")} {m.config.max_copies}<br />{m.config.allowed_capabilities.map(functionName).join(', ') || tr("Cualquier capacidad")}{!m.config.active && tr("· desactivado")}</small>{s.tier<4 && <button disabled={busy || s.quantity<3} onClick={() => void act(() => fuseModule(s.module_id,s.tier),tr("Fusión completada: {0}.", data.grades[s.tier+1]))}>{tr("Fusionar 3 → 1")} {data.grades[s.tier+1]}</button>}</article>;
      })}</div>
      {!available.length && <p>{tr("No hay copias libres. Completa un encargo o retira un módulo instalado.")}</p>}
    </>}
  </section>;
});
