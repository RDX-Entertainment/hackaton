import { tr, useIdioma } from '../../i18n/text';
import { useState, type FormEvent } from 'react';

import { accountApi } from '../../utils/api';
import { formatCount } from '../../utils/format';
import { useStatusMessage } from '../../utils/useStatusMessage';

const MAX_LENGTH = 3000;

type BiographyCardProps = {
  biography: string;
  onSaved: () => void;
};

/**
 * The biography, read-only until you ask to change it. Editing is a mode you
 * enter deliberately rather than a textarea sitting open forever, so the page
 * reads as a profile and not as a form.
 */
export function BiographyCard({ biography, onSaved }: BiographyCardProps) {
  useIdioma();
  const status = useStatusMessage();

  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(biography);
  const [busy, setBusy] = useState(false);

  const open = () => {
    setDraft(biography);
    status.clear();
    setEditing(true);
  };

  const cancel = () => {
    setDraft(biography);
    status.clear();
    setEditing(false);
  };

  const save = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);

    try {
      await accountApi.saveProfile(draft);
      setEditing(false);
      onSaved();
    } catch (error) {
      status.reportError(error);
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel">
      <header className="panel__head">
        <span className="panel__path">~/.profile</span>

        {!editing && (
          <button type="button" className="panel__action" onClick={open}>
            {tr("editar")} </button>
        )}
      </header>

      {editing ? (
        <form className="panel__body" onSubmit={save}>
          <textarea
            className="panel__editor"
            value={draft}
            maxLength={MAX_LENGTH}
            autoFocus
            onChange={(event) => setDraft(event.target.value)}
          />

          <div className="panel__foot">
            <span className="panel__count">
              {formatCount(MAX_LENGTH - draft.length)} {tr("restantes")} </span>

            <span className="panel__buttons">
              <button type="button" onClick={cancel} disabled={busy}>
                {tr("cancelar")} </button>
              <button className="is-primary" disabled={busy}>
                {busy ? tr("guardando…") : tr("guardar")}
              </button>
            </span>
          </div>

          {status.message && <p className="panel__error">{status.message}</p>}
        </form>
      ) : (
        <div className="panel__body">
          {biography ? (
            <pre className="panel__text">{biography}</pre>
          ) : (
            <p className="panel__empty">
              <span aria-hidden="true">$ </span>{tr("cat ~/.profile: fichero vacío")} </p>
          )}
        </div>
      )}
    </section>
  );
}
