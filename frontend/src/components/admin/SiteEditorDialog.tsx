import { useEffect, useState } from 'react';

type SiteEditorDialogProps = {
  open: boolean;
  title: string;
  initialWeb: string;
  busy?: boolean;
  onClose: () => void;
  onSave: (web: string) => void;
};

export function SiteEditorDialog({ open, title, initialWeb, busy, onClose, onSave }: SiteEditorDialogProps) {
  const [web, setWeb] = useState(initialWeb);
  useEffect(() => { if (open) setWeb(initialWeb); }, [initialWeb, open]);
  if (!open) return null;
  return <div className="site-editor-backdrop" role="presentation" onMouseDown={onClose}>
    <section className="site-editor" role="dialog" aria-modal="true" aria-label={`Editor web: ${title}`} onMouseDown={(event) => event.stopPropagation()}>
      <header><div><small>EDITOR WEB</small><h3>{title}</h3></div><button type="button" onClick={onClose}>Cerrar</button></header>
      <div className="site-editor__split"><label>HTML<textarea value={web} onChange={(event) => setWeb(event.target.value)} spellCheck={false} placeholder="<h1>Servicio</h1>" /></label><label>Previsualización<iframe title={`Vista previa de ${title}`} sandbox="" srcDoc={web} /></label></div>
      <footer><button type="button" onClick={onClose}>Cancelar</button><button className="is-primary" disabled={busy} onClick={() => onSave(web)}>Guardar web</button></footer>
    </section>
  </div>;
}
