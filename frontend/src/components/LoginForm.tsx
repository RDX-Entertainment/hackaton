import { tr, useIdioma } from '../i18n/text';
import { useState, type FormEvent } from 'react';

import { authenticated } from '../app/authSlice';
import { useAppDispatch } from '../app/hooks';
import { authApi } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';
import { useNavigate } from 'react-router-dom';
import { idiomaActual } from '../i18n';

const MIN_PASSWORD_LENGTH = 8;

type LoginFormProps = {
  /** Opens on the register fields rather than the sign-in ones. */
  startRegistering?: boolean;
};

/**
 * The plain way in, for when you do not feel like typing commands. Same two
 * calls the shell makes, wrapped in a form instead of a prompt.
 */
export function LoginForm({ startRegistering = false }: LoginFormProps) {
  useIdioma();
  const dispatch = useAppDispatch();
  const navigate = useNavigate();
  const status = useStatusMessage();

  const [registering, setRegistering] = useState(startRegistering);
  const [identifier, setIdentifier] = useState('');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);

    try {
      const session = registering
        ? await authApi.register(username, password, identifier, idiomaActual())
        : await authApi.login(identifier, password);

      dispatch(authenticated(session));
      navigate('/');
    } catch (error) {
      status.reportError(error);
    } finally {
      setBusy(false);
    }
  };

  const swap = () => {
    setRegistering((current) => !current);
    status.clear();
  };

  return (
    <div className="login">
      <p className="login__brand">
        Hackaton<b>V-Rustup</b>
      </p>
      <p className="login__lead">
        {registering ? tr("Crea una cuenta para empezar.") : tr("Entra con tu cuenta.")}
      </p>

      <form className="login__form" onSubmit={submit}>
        {registering && (
          <label>
            {tr("Username")} <input
              value={username}
              onChange={(event) => setUsername(event.target.value)}
              autoComplete="username"
              required
            />
          </label>
        )}

        <label>
          {registering ? tr("Email") : tr("Email o usuario")}
          <input
            type={registering ? 'email' : 'text'}
            value={identifier}
            onChange={(event) => setIdentifier(event.target.value)}
            autoComplete={registering ? 'email' : 'username'}
            required
          />
        </label>

        <label>
          {tr("Password")} <input
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            autoComplete={registering ? 'new-password' : 'current-password'}
            minLength={registering ? MIN_PASSWORD_LENGTH : undefined}
            required
          />
          {registering && <small>{tr("Mínimo")} {MIN_PASSWORD_LENGTH} {tr("caracteres.")}</small>}
        </label>

        {status.message && <output className="login__error">{status.message}</output>}

        <button className="login__submit" disabled={busy}>
          {busy ? tr("espera…") : registering ? tr("crear cuenta") : tr("entrar")}
        </button>
      </form>

      <p className="login__swap">
        {registering ? tr("¿Ya tienes cuenta?") : tr("¿No tienes cuenta?")}{' '}
        <button type="button" className="login__link" onClick={swap}>
          {registering ? tr("entrar") : tr("crear una")}
        </button>
      </p>
    </div>
  );
}
