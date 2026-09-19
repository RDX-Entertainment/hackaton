import { tr, useIdioma } from '../../i18n/text';
import { useState, type FormEvent } from 'react';

import type { Remote } from '../../utils/api';
import { useAccion } from '../../app/tutorial';
import { formatIp } from '../../utils/ip';
import { RemoteLogEditor } from './RemoteLogEditor';

type AccessTabProps = {
  remote: Remote;
  log: string;
  onSignIn: (user: string, password: string) => Promise<boolean>;
  onRewriteLog: (text: string) => void;
  onWipeLog: () => void;
};

/**
 * La sesión: entrar, y lo que se puede hacer una vez dentro.
 *
 * Sin acceso solo hay una consola pidiendo credenciales. Con acceso, el disco y el
 * registro — que es lo único que de verdad cambia al haber entrado.
 */
export function AccessTab({
  remote,
  log,
  onSignIn,
  onRewriteLog,
  onWipeLog,
}: AccessTabProps) {
  useIdioma();
  const entrar = useAccion('red.entrar');

  const [user, setUser] = useState('root');
  const [password, setPassword] = useState('');

  const submit = async (event: FormEvent) => {
    event.preventDefault();

    // Una contraseña rechazada se queda en el campo, para poder corregirla.
    if (await onSignIn(user, password)) {
      setPassword('');
    }
  };

  if (remote.down) {
    return (
      <div className="page">
        <p className="page__note">{tr("La máquina no responde. No hay a qué conectarse.")}</p>
      </div>
    );
  }

  if (!remote.access) {
    return (
      <div className="page">
        <form className="ssh" onSubmit={(event) => void submit(event)}>
          <p className="ssh__line">
            <span className="ssh__prompt">$</span> ssh {user || tr("usuario")}@{formatIp(remote.ip)}
          </p>

          <label className="ssh__field">
            <span>{tr("usuario")}</span>
            <input
              value={user}
              spellCheck={false}
              onChange={(event) => setUser(event.target.value)}
              required
            />
          </label>

          <label className="ssh__field">
            <span>{tr("contraseña")}</span>
            <input
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              required
            />
          </label>

          <button
            className={entrar.destacada ? 'is-primary es-foco' : 'is-primary'}
            data-foco="red.entrar"
            disabled={!entrar.permitida}
            title={entrar.motivo ?? undefined}
          >
            {tr("entrar")} </button>

          <p className="ssh__hint">
            {tr("¿No la sabes? En")} <b>force</b> {tr("están las formas de conseguirla.")} </p>
        </form>
      </div>
    );
  }

  return (
    <div className="page">
      <p className="ssh__line ssh__line--in">
        <span className="ssh__prompt">$</span> {tr("sesión abierta como")} <b>{remote.access_as ?? 'root'}</b> {tr("en")}{' '}
        {formatIp(remote.ip)}
      </p>

      <section className="page__block">
        <h4>{tr("Registro")}</h4>
        <RemoteLogEditor log={log} onRewrite={onRewriteLog} onWipe={onWipeLog} />
      </section>
    </div>
  );
}
