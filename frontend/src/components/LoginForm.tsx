import { useState, type FormEvent } from 'react';

import { authenticated } from '../app/authSlice';
import { useAppDispatch } from '../app/hooks';
import { authApi } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';
import { useNavigate } from 'react-router-dom';

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
        ? await authApi.register(username, password, identifier)
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
        {registering ? 'Crea una cuenta para empezar.' : 'Entra con tu cuenta.'}
      </p>

      <form className="login__form" onSubmit={submit}>
        {registering && (
          <label>
            Username
            <input
              value={username}
              onChange={(event) => setUsername(event.target.value)}
              autoComplete="username"
              required
            />
          </label>
        )}

        <label>
          {registering ? 'Email' : 'Email o usuario'}
          <input
            type={registering ? 'email' : 'text'}
            value={identifier}
            onChange={(event) => setIdentifier(event.target.value)}
            autoComplete={registering ? 'email' : 'username'}
            required
          />
        </label>

        <label>
          Password
          <input
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            autoComplete={registering ? 'new-password' : 'current-password'}
            minLength={registering ? MIN_PASSWORD_LENGTH : undefined}
            required
          />
          {registering && <small>Mínimo {MIN_PASSWORD_LENGTH} caracteres.</small>}
        </label>

        {status.message && <output className="login__error">{status.message}</output>}

        <button className="login__submit" disabled={busy}>
          {busy ? 'espera…' : registering ? 'crear cuenta' : 'entrar'}
        </button>
      </form>

      <p className="login__swap">
        {registering ? '¿Ya tienes cuenta?' : '¿No tienes cuenta?'}{' '}
        <button type="button" className="login__link" onClick={swap}>
          {registering ? 'entrar' : 'crear una'}
        </button>
      </p>
    </div>
  );
}
