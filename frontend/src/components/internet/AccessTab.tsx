import { useState, type FormEvent } from 'react';

import type { Remote } from '../../utils/api';
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
        <p className="page__note">La máquina no responde. No hay a qué conectarse.</p>
      </div>
    );
  }

  if (!remote.access) {
    return (
      <div className="page">
        <form className="ssh" onSubmit={(event) => void submit(event)}>
          <p className="ssh__line">
            <span className="ssh__prompt">$</span> ssh {user || 'usuario'}@{formatIp(remote.ip)}
          </p>

          <label className="ssh__field">
            <span>usuario</span>
            <input
              value={user}
              spellCheck={false}
              onChange={(event) => setUser(event.target.value)}
              required
            />
          </label>

          <label className="ssh__field">
            <span>contraseña</span>
            <input
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              required
            />
          </label>

          <button className="is-primary">entrar</button>

          <p className="ssh__hint">
            ¿No la sabes? En <b>force</b> están las formas de conseguirla.
          </p>
        </form>
      </div>
    );
  }

  return (
    <div className="page">
      <p className="ssh__line ssh__line--in">
        <span className="ssh__prompt">$</span> sesión abierta como <b>{remote.access_as ?? 'root'}</b> en{' '}
        {formatIp(remote.ip)}
      </p>

      <section className="page__block">
        <h4>Registro</h4>
        <RemoteLogEditor log={log} onRewrite={onRewriteLog} onWipe={onWipeLog} />
      </section>
    </div>
  );
}
