import { tr, useIdioma } from '../i18n/text';
import { useCallback, useEffect, useState, type FormEvent } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { authenticated } from '../app/authSlice';
import type { RootState } from '../app/store';
import { Loading } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import { IDIOMAS, NOMBRE_DEL_IDIOMA, esIdioma } from '../i18n';
import { accountApi, authApi, type Settings } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';
import { formatDateTime } from '../utils/format';

const MIN_PASSWORD_LENGTH = 8;

export function SettingsPage() {
  useIdioma();
  const status = useStatusMessage();
  const dispatch = useDispatch();
  const session = useSelector((state: RootState) => state.auth);

  const [settings, setSettings] = useState<Settings>();
  const [email, setEmail] = useState('');
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');

  const load = useCallback(() => {
    accountApi.getSettings().then((loaded) => {
      setSettings(loaded);
      setEmail(loaded.email);
    });
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const save = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await accountApi.saveSettings(email, currentPassword, newPassword);
      setCurrentPassword('');
      setNewPassword('');
      status.report(tr("Ajustes guardados."));
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  /**
   * El idioma se guarda solo al elegirlo, sin boton: es una preferencia, no un formulario, y
   * el efecto se ve en el acto. Se vuelve a meter la sesion entera en el store porque la
   * lengua viaja dentro del usuario, y de ahi la recoge el catalogo.
   */
  const cambiarIdioma = async (lang: string) => {
    if (!session.token || !esIdioma(lang)) {
      return;
    }

    try {
      const user = await authApi.setLanguage(lang);
      dispatch(authenticated({ token: session.token, user }));
      status.report(tr("Idioma guardado."));
    } catch (error) {
      status.reportError(error);
    }
  };

  if (!settings) {
    return <Loading />;
  }

  return (
    <>
      <h2>{tr("Ajustes")}</h2>

      <div className="ajustes">
        <section className="ajustes__grupo">
          <div className="ajustes__cabecera">
            <span className="ajustes__titulo">{tr("Quién eres")}</span>
            <span className="micro">{tr("no se cambia")}</span>
          </div>
          <div className="ajustes__cuerpo">
            <div className="clan__miembro" style={{ border: 0, padding: 0 }}>
              <span className="nombre">{settings.login}</span>
              <span className="puesto">{settings.premium ? 'premium' : tr("cuenta normal")}</span>
            </div>
            <p className="ajustes__nota">
              {tr("Última entrada:")} {formatDateTime(settings.last_login)}
            </p>
          </div>
        </section>

        <section className="ajustes__grupo">
          <div className="ajustes__cabecera">
            <span className="ajustes__titulo">{tr("Idioma")}</span>
            <span className="micro">{tr("se guarda solo")}</span>
          </div>
          <div className="ajustes__cuerpo">
            <div className="idiomas">
              {IDIOMAS.map((lang) => (
                <button
                  key={lang}
                  type="button"
                  className={(session.user?.lang ?? 'es') === lang ? 'is-primary' : ''}
                  onClick={() => void cambiarIdioma(lang)}
                >
                  {NOMBRE_DEL_IDIOMA[lang]}
                </button>
              ))}
            </div>
            <p className="ajustes__nota">
              {tr("Viaja contigo: lo eliges aquí y lo tienes en cualquier navegador.")} </p>
          </div>
        </section>

        <section className="ajustes__grupo">
          <div className="ajustes__cabecera">
            <span className="ajustes__titulo">{tr("Correo y contraseña")}</span>
            <span className="micro">{tr("hace falta la de ahora")}</span>
          </div>
          <div className="ajustes__cuerpo">
            <form className="compose" onSubmit={save}>
              <label>
                {tr("Correo electrónico")} <input
                  type="email"
                  value={email}
                  onChange={(event) => setEmail(event.target.value)}
                  required
                />
              </label>

              <label>
                {tr("Contraseña actual")} <input
                  type="password"
                  value={currentPassword}
                  onChange={(event) => setCurrentPassword(event.target.value)}
                  required
                />
              </label>

              <label>
                {tr("Contraseña nueva (solo si la cambias)")} <input
                  type="password"
                  value={newPassword}
                  minLength={MIN_PASSWORD_LENGTH}
                  onChange={(event) => setNewPassword(event.target.value)}
                />
              </label>

              <button>{tr("Guardar")}</button>
              <StatusMessage text={status.message} />
            </form>
          </div>
        </section>
      </div>
    </>
  );
}
