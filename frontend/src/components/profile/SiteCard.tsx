import { tr, useIdioma } from '../../i18n/text';
import { useState } from 'react';

import { accountApi, type Profile } from '../../utils/api';
import { useStatusMessage } from '../../utils/useStatusMessage';
import { SiteEditor } from '../internet/SiteEditor';
import { SiteFrame } from '../internet/SiteFrame';

type SiteCardProps = {
  profile: Profile;
  onSaved: () => void;
};

/**
 * Tu página, la que ve cualquiera que navegue a tu dirección.
 *
 * Se lee cerrada y se edita a propósito, igual que la biografía: la pantalla es un perfil,
 * no un formulario abierto para siempre.
 */
export function SiteCard({ profile, onSaved }: SiteCardProps) {
  useIdioma();
  const status = useStatusMessage();

  const [editing, setEditing] = useState(false);
  const [busy, setBusy] = useState(false);

  const save = async (web: string) => {
    setBusy(true);
    status.clear();

    try {
      await accountApi.saveProfile(profile.biography, web);
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
        <span className="panel__path">~/public_html/index.html</span>

        <button
          type="button"
          className="panel__action"
          onClick={() => {
            status.clear();
            setEditing((one) => !one);
          }}
        >
          {editing ? tr("cerrar") : tr("editar")}
        </button>
      </header>

      <div className="panel__body">
        {editing ? (
          <SiteEditor
            ip={profile.game_ip}
            name={profile.login}
            web={profile.web}
            busy={busy}
            error={status.message}
            onSave={(web) => void save(web)}
          />
        ) : (
          <SiteFrame ip={profile.game_ip} html={profile.web} name={profile.login} down={false} />
        )}
      </div>
    </section>
  );
}
