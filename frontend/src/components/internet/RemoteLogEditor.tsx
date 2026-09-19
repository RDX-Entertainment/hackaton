import { tr, useIdioma } from '../../i18n/text';
import { useEffect, useState, type FormEvent } from 'react';

type RemoteLogEditorProps = {
  log: string;
  onRewrite: (text: string) => void;
  onWipe: () => void;
};

/** Reescribir el rastro que dejas es trabajo encolado, no una edición inmediata. */
export function RemoteLogEditor({ log, onRewrite, onWipe }: RemoteLogEditorProps) {
  useIdioma();
  const [draft, setDraft] = useState(log);
  const [editing, setEditing] = useState(false);

  // Otra máquina trae otro registro, así que el borrador la sigue.
  useEffect(() => {
    setDraft(log);
    setEditing(false);
  }, [log]);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    onRewrite(draft);
    setEditing(false);
  };

  return (
    <div className="rlog">
      <pre className="rlog__text">{log || tr("(el registro está vacío)")}</pre>

      {editing ? (
        <form className="rlog__form" onSubmit={submit}>
          <label>
            {tr("reemplazar el registro por")} <textarea
              value={draft}
              rows={5}
              spellCheck={false}
              onChange={(event) => setDraft(event.target.value)}
            />
          </label>

          <div className="rlog__actions">
            <button className="is-primary">{tr("reescribir")}</button>
            <button
              type="button"
              onClick={() => {
                setDraft(log);
                setEditing(false);
              }}
            >
              {tr("cancelar")} </button>
          </div>
        </form>
      ) : (
        <div className="rlog__actions">
          <button type="button" onClick={() => setEditing(true)}>
            {tr("reescribir")} </button>
          <button type="button" className="rlog__wipe" onClick={onWipe}>
            {tr("borrarlo entero")} </button>
        </div>
      )}
    </div>
  );
}
