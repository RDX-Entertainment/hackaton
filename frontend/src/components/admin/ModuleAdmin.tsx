import { tr, useIdioma } from '../../i18n/text';
import { useEffect, useState, type FormEvent } from 'react';
import { arsenalApi, type Capability, type ProgramStats } from '../../utils/api';
import { moduleCatalog, saveModule, statNames, type ModuleTemplate } from '../../utils/api/workshop';
import { StatusMessage } from '../StatusMessage';
import { useStatusMessage } from '../../utils/useStatusMessage';
import '../../styles/arsenal/modules.css';

const blank = (): ModuleTemplate => ({ code:'',name:'',config:{allowed_capabilities:[],allowed_slots:[1,2,3,4,5,6],min_level:1,max_copies:1,excludes:[],grants:[],active:true,collectable:true,deltas:{}} });
export function ModuleAdmin() {
  useIdioma();
  const [items,setItems]=useState<ModuleTemplate[]>([]);
  const [caps,setCaps]=useState<Capability[]>([]);
  const [draft,setDraft]=useState<ModuleTemplate>(blank);
  const [busy,setBusy]=useState(false);
  const status=useStatusMessage();
  useEffect(()=>{ moduleCatalog().then(setItems).catch(status.reportError); arsenalApi.listCapabilities().then(setCaps).catch(status.reportError); },[status.reportError]);
  const config=<K extends keyof ModuleTemplate['config']>(k:K,v:ModuleTemplate['config'][K])=>setDraft(d=>({...d,config:{...d.config,[k]:v}}));
  function toggle(key:'grants'|'allowed_capabilities'|'excludes',value:string) { const current=draft.config[key]; config(key,current.includes(value)?current.filter(v=>v!==value):[...current,value]); }
  async function submit(e:FormEvent) {
    e.preventDefault();setBusy(true);
    try { await saveModule(draft);setItems(await moduleCatalog());setDraft(blank());status.report(tr("Módulo guardado. Las nuevas instalaciones usan esta configuración; las ya instaladas conservan su versión.")); }
    catch(e){status.reportError(e);} finally{setBusy(false);}
  }
  const functions=[...caps.map(c=>({code:c.code,name:c.name})),{code:'antivirus_scan',name:tr("Solo detectar virus")},{code:'antivirus_clean',name:tr("Solo eliminar virus")}];
  return <section className="module-admin">
    <h2>{tr("Catálogo de módulos")}</h2><p className="muted">{tr("Define atributos, funciones y lugares de instalación. Desactivar impide nuevas instalaciones y su obtención; no destruye copias existentes. El código es permanente.")}</p>
    <StatusMessage text={status.message} as="block" />
    <label>{tr("Editar módulo")}<select value={draft.id ?? ''} onChange={e=>setDraft(items.find(m=>m.id===Number(e.target.value)) ?? blank())}><option value="">{tr("Crear módulo nuevo")}</option>{items.map(m=><option key={m.id} value={m.id}>{m.name}{!m.config.active?tr("· desactivado"):''}</option>)}</select></label>
    <form className="compose" onSubmit={submit}>
      <h3>{draft.id?tr("Editar"):tr("Crear")} {tr("módulo")}</h3>
      <label>{tr("Código")}<input required maxLength={48} pattern="[a-z0-9_]+" disabled={!!draft.id} value={draft.code} onChange={e=>setDraft(d=>({...d,code:e.target.value}))} /></label>
      <label>{tr("Nombre")}<input required maxLength={60} value={draft.name} onChange={e=>setDraft(d=>({...d,name:e.target.value}))} /></label>
      <fieldset><legend>{tr("Disponibilidad")}</legend><label className="module-check"><input type="checkbox" checked={draft.config.active} onChange={e=>config('active',e.target.checked)} />{tr("Activo")}</label><label className="module-check"><input type="checkbox" checked={draft.config.collectable} onChange={e=>config('collectable',e.target.checked)} />{tr("Disponible al recabar")}</label></fieldset>
      <fieldset><legend>{tr("Restricciones")}</legend><label>{tr("Nivel mínimo")}<input type="number" min={1} max={9999} required value={draft.config.min_level} onChange={e=>config('min_level',Number(e.target.value))} /></label><label>{tr("Máximo por programa")}<input type="number" min={1} max={6} required value={draft.config.max_copies} onChange={e=>config('max_copies',Number(e.target.value))} /></label></fieldset>
      <fieldset><legend>{tr("Posiciones permitidas")}</legend>{[1,2,3,4,5,6].map(n=><label className="module-check" key={n}><input type="checkbox" checked={draft.config.allowed_slots.includes(n)} onChange={()=>config('allowed_slots',draft.config.allowed_slots.includes(n)?draft.config.allowed_slots.filter(v=>v!==n):[...draft.config.allowed_slots,n])} />{tr("Ranura")} {n}</label>)}</fieldset>
      <fieldset><legend>{tr("Capacidades de destino · ninguna marcada = todas")}</legend>{caps.map(c=><label className="module-check" key={c.code}><input type="checkbox" checked={draft.config.allowed_capabilities.includes(c.code)} onChange={()=>toggle('allowed_capabilities',c.code)} />{c.name}</label>)}</fieldset>
      <fieldset><legend>{tr("Funciones que concede")}</legend>{functions.map(c=><label className="module-check" key={c.code}><input type="checkbox" checked={draft.config.grants.includes(c.code)} onChange={()=>toggle('grants',c.code)} />{c.name}</label>)}</fieldset>
      <fieldset><legend>{tr("Atributos del grado Estándar · incrementos, no valores finales")}</legend>{Object.entries(statNames).map(([key,label])=>{const k=key as keyof ProgramStats;const max=['cpu','ram','net','weight'].includes(k)?10000:100;return <label key={k}>{label}<input type="number" min={-max} max={max} value={draft.config.deltas[k] ?? 0} onChange={e=>config('deltas',{...draft.config.deltas,[k]:Number(e.target.value)})} /></label>;})}</fieldset>
      <fieldset><legend>{tr("Incompatible con")}</legend>{items.filter(m=>m.id!==draft.id).map(m=><label className="module-check" key={m.code}><input type="checkbox" checked={draft.config.excludes.includes(m.code)} onChange={()=>toggle('excludes',m.code)} />{m.name}</label>)}</fieldset>
      <p className="muted">{tr("Multiplicadores de grado: ×1 / ×1,35 / ×1,8 / ×2,4 / ×3,2. Se aplican también a costes. Las restricciones se comprueban en el servidor.")}</p>
      <button disabled={busy || !draft.config.allowed_slots.length}>{busy?tr("Guardando…"):tr("Guardar módulo")}</button><button type="button" disabled={busy} onClick={()=>setDraft(blank())}>{tr("Nuevo / cancelar")}</button>
    </form>
  </section>;
}
