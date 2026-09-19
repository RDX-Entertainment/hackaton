import { tr, useIdioma } from '../i18n/text';
import { Link } from 'react-router-dom';
import { formatIp } from '../utils/ip';
import { useCallback, useEffect, useRef, useState, type FormEvent } from 'react';

import { EmptyNotice, Loading } from '../components/Notice';
import { ProcessProgress } from '../components/ProcessProgress';
import { StatusMessage } from '../components/StatusMessage';
import { arsenalApi, type EvidenceSummary, type MachineAccess, type WebContent } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

/** Consola posterior a la intrusión: el servidor decide permisos y resultados. */
export function AccessesPage() {
  useIdioma();
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
  useEffect(() => { let current = true; setWeb(undefined); setDraft(''); if (selected) void arsenalApi.getWebContent(selected).then((content) => { if (current) { setWeb(content); setDraft(content.content); } }).catch(() => { if (current) setWeb(undefined); }); return () => { current = false; }; }, [selected]);

  const [publishing, setPublishing] = useState(false);
  const selectedRef = useRef(selected); selectedRef.current = selected;
  const modify = async (event: FormEvent) => {
    event.preventDefault(); if (!selected || publishing) return;
    const ip = selected; setPublishing(true);
    try {
      const result = await arsenalApi.modifyWeb(ip, draft);
      if (selectedRef.current !== ip) return;
      setWeb(result); setDraft(result.content);
      status.report(tr("HTML publicado en {0} · revisión {1}.", formatIp(ip), result.revision));
      load();
    } catch (error) { if (selectedRef.current === ip) status.reportError(error); }
    finally { setPublishing(false); }
  };
  const hide = async (id: number) => { if (!selected) return; try { const result = await arsenalApi.hideEvidence(selected, id); setProcessId(result.process_id); status.report(tr("Ocultación en cola ({0}).", result.estimated_chance)); } catch (error) { status.reportError(error); } };
  const forge = async (id: number) => { if (!selected) return; const ip = Number(prompt(tr("IP conocida que quieres aparentar"))); if (!Number.isFinite(ip)) return; try { const result = await arsenalApi.forgeEvidence(selected, id, ip); setProcessId(result.process_id); status.report(tr("Falsificación en cola ({0}).", result.estimated_chance)); } catch (error) { status.reportError(error); } };
  const analyze = async (id: number) => { try { const result = await arsenalApi.analyzeEvidence(id); setProcessId(result.process_id); status.report(tr("Análisis forense en cola ({0}).", result.estimated_chance)); } catch (error) { status.reportError(error); } };
  const discover = async () => { try { const result = await arsenalApi.discoverEvidence(); setProcessId(result.process_id); status.report(tr("Búsqueda forense en cola ({0}).", result.estimated_chance)); } catch (error) { status.reportError(error); } };

  if (!accesses) return <Loading />;
  const selectedAccess = accesses.find((access) => access.ip === selected);
  return <><h2>{tr("Accesos comprometidos")}</h2><StatusMessage text={status.message} as="block" />
    {accesses.length ? <><label className="lab__picker">{tr("Máquina")}<select value={selected ?? ''} onChange={(event) => setSelected(Number(event.target.value))}>{accesses.map((access) => <option key={access.ip} value={access.ip}>{formatIp(access.ip)} · {access.identity} · {access.scope}</option>)}</select></label>
    {selected && <p><Link to={`/internet?ip=${selected}`}>{tr("Abrir máquina en Internet → Servicios")}</Link></p>}
    {selectedAccess && <p className="muted">{tr("Alcance:")} {selectedAccess.scope} {tr("· privilegio:")} {selectedAccess.privilege} {tr("· permisos:")} {selectedAccess.permissions.join(', ')}</p>}</> : <EmptyNotice>{tr("No tienes sesiones tácticas activas; las alertas forenses siguen apareciendo abajo.")}</EmptyNotice>}
    {web && <form className="compose" onSubmit={modify}><h3>{web.path} {tr("· revisión")} {web.revision}</h3><textarea rows={10} value={draft} onChange={(event) => setDraft(event.target.value)} /><button disabled={publishing || !selectedAccess?.permissions.includes('write_web')}>{publishing ? tr("Publicando…") : tr("Modificar HTML")}</button></form>}
    {accesses.length > 0 && !web && <p className="muted">{tr("Este acceso no permite leer contenido web o la máquina no tiene página editable.")}</p>}
    <section className="arsenal__block"><h3>{tr("Evidencia propia o descubierta")}</h3><button type="button" onClick={() => void discover()}>{tr("buscar evidencia oculta")}</button>{evidence.length ? evidence.map((item) => <article className="forum-post" key={item.id}><b>{item.action}</b><small> · {item.state} {tr("· confianza")} {item.confidence}</small><p>{item.can_tamper && selectedAccess && <><button type="button" onClick={() => void hide(item.id)}>{tr("ocultar")}</button> <button type="button" onClick={() => void forge(item.id)}>{tr("falsificar origen")}</button> </>}<button type="button" onClick={() => void analyze(item.id)}>{tr("analizar")}</button></p></article>) : <EmptyNotice>{tr("No hay evidencia visible.")}</EmptyNotice>}</section>
    {processId && <ProcessProgress processId={processId} onFinished={load} />}
  </>;
}
