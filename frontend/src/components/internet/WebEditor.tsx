import { tr, useIdioma } from '../../i18n/text';
import { useEffect, useState } from 'react';
import { arsenalApi, type Remote } from '../../utils/api';
import { SiteFrame } from './SiteFrame';
import { messageFrom } from '../../utils/errors';

type Props = { remote: Remote; busy: boolean; onPublish: (content: string) => Promise<void> };
const PREVIEW_POLICY = '<meta http-equiv="Content-Security-Policy" content="default-src \'none\'; style-src \'unsafe-inline\'; img-src data:; form-action \'none\'; base-uri \'none\'">';

export function WebEditor({ remote, busy, onPublish }: Props) {
  useIdioma();
  const [draft, setDraft] = useState('');
  const [original, setOriginal] = useState('');
  const [preview, setPreview] = useState('');
  const [revision, setRevision] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [saving, setSaving] = useState(false);
  const [publishError, setPublishError] = useState('');
  useEffect(() => {
    let current = true;
    arsenalApi.getWebContent(remote.ip).then((web) => {
      if (!current) return;
      setDraft(web.content); setOriginal(web.content); setPreview(web.content); setRevision(web.revision);
    }).catch(() => { if (current) setError(tr("No se pudo leer el sitio. Vuelve a abrir el editor para reintentar.")); })
      .finally(() => { if (current) setLoading(false); });
    return () => { current = false; };
  }, [remote.ip]);
  const bytes = new TextEncoder().encode(draft).length;
  return <section className="page webedit">
    <h3>{tr("Editor del sitio · /index.html")}</h3>
    <p className="page__note">{tr("Revisión")} {revision}{tr(". Edita HTML y CSS o las páginas del documento Webscript. Con tu permiso de escritura, Publicar modificación guarda el HTML en este servidor y actualiza su sitio público. La vista previa no guarda cambios.")}</p>
    {loading ? <p>{tr("Cargando HTML…")}</p> : error ? <p role="alert">{error}</p> : <>
      <div className="webedit__grid"><label>{tr("HTML del sitio")}<textarea spellCheck={false} value={draft} onChange={(e) => setDraft(e.target.value)} /></label>
        <div><p>{tr("Vista previa aislada · las actions no se ejecutan aquí")}</p><SiteFrame ip={remote.ip} html={PREVIEW_POLICY + preview} name="vista previa" down={false} /></div>
      </div>
      <div className="webedit__actions">
        <button type="button" onClick={() => setPreview(draft)}>{tr("Actualizar vista previa")}</button>
        <button type="button" disabled={busy || saving || remote.down || !remote.permissions.includes('write_web') || draft === original || !draft.trim() || bytes > 16384} onClick={() => {
          setSaving(true); setPublishError('');
          void onPublish(draft).catch((error) => setPublishError(messageFrom(error))).finally(() => setSaving(false));
        }}>{saving ? tr("Publicando…") : tr("Publicar modificación")}</button>
        <small>{bytes.toLocaleString()} {tr("/ 16.384 bytes · sin JavaScript ni recursos externos")}</small>
      </div>
      {publishError && <p role="alert">{publishError}</p>}
    </>}
  </section>;
}
