import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';

import { signedOut } from '../../app/authSlice';
import { useAppDispatch, useAppSelector } from '../../app/hooks';
import { adminApi } from '../../utils/api';
import { formatIp, maskIp } from '../../utils/ip';
import { NavMenu } from './NavMenu';

/**
 * The top rail: the brand, the categories, and the way out. It carries the
 * whole navigation now that the left sidebar is gone.
 */
export function TopBar() {
  const dispatch = useAppDispatch();
  const user = useAppSelector((state) => state.auth.user);
  const [admin, setAdmin] = useState(false);
  const [revealed, setRevealed] = useState(false);

  // The panel routes 404 for everyone else, so asking is how we find out.
  useEffect(() => {
    let alive = true;

    void adminApi.isAdmin().then((allowed) => {
      if (alive) {
        setAdmin(allowed);
      }
    });

    return () => {
      alive = false;
    };
  }, []);

  return (
    <header className="topbar">
      <Link className="topbar__brand" to="/">
        <span className="topbar__mark" aria-hidden="true">
          H
        </span>
        <span className="topbar__name">
          hacker<b>experience</b>
        </span>
      </Link>

      <NavMenu />

      <div className="topbar__session">
        {admin && (
          <Link className="topbar__back" to="/admin">
            panel
          </Link>
        )}

        {user && (
          <span className="topbar__user">
            <span className="topbar__user-label">ip</span>
            <button
              type="button"
              className={revealed ? 'topbar__user-ip is-shown' : 'topbar__user-ip'}
              aria-pressed={revealed}
              aria-label={revealed ? 'Ocultar tu dirección' : 'Mostrar tu dirección'}
              title={revealed ? 'Ocultar tu dirección' : 'Mostrar tu dirección'}
              onClick={() => setRevealed((current) => !current)}
            >
              {revealed ? formatIp(user.game_ip) : maskIp(user.game_ip)}
            </button>
            <span className="topbar__user-name">{user.login}</span>
          </span>
        )}

        <button className="topbar__logout" type="button" onClick={() => dispatch(signedOut())}>
          salir
        </button>
      </div>
    </header>
  );
}
