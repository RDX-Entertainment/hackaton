import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { EmptyNotice, Loading } from '../components/Notice';
import { ProcessProgress } from '../components/ProcessProgress';
import { StatusMessage } from '../components/StatusMessage';
import { arsenalApi, type EvidenceSummary, type MachineAccess, type WebContent } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

/** Consola posterior a la intrusión: el servidor decide permisos y resultados. */
export function AccessesPage() {
  const status = useStatusMessage();
  const [accesses, setAccesses] = useState<MachineAccess[]>();
  const [evidence, setEvidence] = useState<EvidenceSummary[]>([]);
  const [selected, setSelected] = useState<number>();
  const [web, setWeb] = useState<WebContent>();
  const [draft, setDraft] = useState('');
  const [processId, setProcessId] = useState<number>();

  const load = useCallback(() => {
    void arsenalApi.listAccesses().then((items) => { setAccesses(items); setSelected((current) => current ?? items[0]?.ip); }).catch(status.reportError);
    void arsenalApi.listEvidence().then(setEvidence).catch(status.reportError);
  }, [status.reportError]);
  useEffect(() => { load(); }, [load]);
  useEffect(() => { if (selected) void arsenalApi.getWebContent(selected).then((content) => { setWeb(content); setDraft(content.content); }).catch(() => setWeb(undefined)); }, [selected]);

  const modify = async (event: FormEvent) => { event.preventDefault(); if (!selected) return; try { const result = await arsenalApi.modifyWeb(selected, draft); setProcessId(result.process_id); status.report(`Modificación en cola (${result.estimated_chance}).`); } catch (error) { status.reportError(error); } };
  const hide = async (id: number) => { if (!selected) return; try { const result = await arsenalApi.hideEvidence(selected, id); setProcessId(result.process_id); status.report(`Ocultación en cola (${result.estimated_chance}).`); } catch (error) { status.reportError(error); } };
  const forge = async (id: number) => { if (!selected) return; const ip = Number(prompt('IP conocida que quieres aparentar')); if (!Number.isFinite(ip)) return; try { const result = await arsenalApi.forgeEvidence(selected, id, ip); setProcessId(result.process_id); status.report(`Falsificación en cola (${result.estimated_chance}).`); } catch (error) { status.reportError(error); } };
  const analyze = async (id: number) => { try { const result = await arsenalApi.analyzeEvidence(id); setProcessId(result.process_id); status.report(`Análisis forense en cola (${result.estimated_chance}).`); } catch (error) { status.reportError(error); } };
  const discover = async () => { try { const result = await arsenalApi.discoverEvidence(); setProcessId(result.process_id); status.report(`Búsqueda forense en cola (${result.estimated_chance}).`); } catch (error) { status.reportError(error); } };

  if (!accesses) return <Loading />;
  const selectedAccess = accesses.find((access) => access.ip === selected);
  return <><h2>Accesos comprometidos</h2><StatusMessage text={status.message} as="block" />
    {accesses.length ? <><label className="lab__picker">Máquina<select value={selected ?? ''} onChange={(event) => setSelected(Number(event.target.value))}>{accesses.map((access) => <option key={access.ip} value={access.ip}>{access.ip} · {access.identity} · {access.scope}</option>)}</select></label>
    {selectedAccess && <p className="muted">Alcance: {selectedAccess.scope} · privilegio: {selectedAccess.privilege} · permisos: {selectedAccess.permissions.join(', ')}</p>}</> : <EmptyNotice>No tienes sesiones tácticas activas; las alertas forenses siguen apareciendo abajo.</EmptyNotice>}
    {web && <form className="compose" onSubmit={modify}><h3>{web.path} · revisión {web.revision}</h3><textarea rows={10} value={draft} onChange={(event) => setDraft(event.target.value)} /><button disabled={!selectedAccess?.permissions.includes('write_files')}>Modificar HTML</button></form>}
    {accesses.length > 0 && !web && <p className="muted">Este acceso no permite leer contenido web o la máquina no tiene página editable.</p>}
    <section className="arsenal__block"><h3>Evidencia propia o descubierta</h3><button type="button" onClick={() => void discover()}>buscar evidencia oculta</button>{evidence.length ? evidence.map((item) => <article className="forum-post" key={item.id}><b>{item.action}</b><small> · {item.state} · confianza {item.confidence}</small><p>{item.can_tamper && selectedAccess && <><button type="button" onClick={() => void hide(item.id)}>ocultar</button> <button type="button" onClick={() => void forge(item.id)}>falsificar origen</button> </>}<button type="button" onClick={() => void analyze(item.id)}>analizar</button></p></article>) : <EmptyNotice>No hay evidencia visible.</EmptyNotice>}</section>
    {processId && <ProcessProgress processId={processId} onFinished={load} />}
  </>;
}
