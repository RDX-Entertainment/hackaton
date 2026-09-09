import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';

import { signedOut } from '../../app/authSlice';
import { useAppDispatch, useAppSelector } from '../../app/hooks';
import { adminApi, dashboardApi, type Dashboard } from '../../utils/api';
import { formatIp, maskIp } from '../../utils/ip';
import { formatMoney } from '../../utils/format';
import { Icono } from '../iconos/Icono';

/** Cada cuánto se refresca la cabecera. El worker mueve el dinero y las tareas. */
const REFRESCO_MS = 20_000;

/**
 * El HUD: lo que nunca debe desaparecer de la pantalla.
 *
 * Antes esta barra llevaba la navegación entera y ningún dato del juego, así
 * que para saber cuánto dinero tenías había que ir a Finanzas. Ahora la
 * navegación vive en el carril y aquí quedan las lecturas: quién eres, tu
 * dirección, tus fondos y si hay algo en marcha.
 *
 * Las cifras van en la voz de display y el ámbar significa **tiempo**: si hay
 * tareas corriendo, se ven desde cualquier pantalla.
 */
export function TopBar() {
  const dispatch = useAppDispatch();
  const user = useAppSelector((state) => state.auth.user);
  const [admin, setAdmin] = useState(false);
  const [revelada, setRevelada] = useState(false);
  const [dashboard, setDashboard] = useState<Dashboard>();

  // El panel contesta 404 a todo el mundo menos a un administrador, así que
  // preguntar es como se averigua.
  useEffect(() => {
    let vivo = true;

    void adminApi.isAdmin().then((permitido) => {
      if (vivo) {
        setAdmin(permitido);
      }
    });

    return () => {
      vivo = false;
    };
  }, []);

  useEffect(() => {
    const leer = () =>
      dashboardApi
        .getDashboard()
        .then(setDashboard)
        .catch(() => setDashboard(undefined));

    leer();
    const reloj = window.setInterval(leer, REFRESCO_MS);
    return () => window.clearInterval(reloj);
  }, []);

  const tareas = dashboard?.processes ?? 0;

  return (
    <header className="hud">
      <Link className="hud__marca" to="/">
        <span className="hud__cuadro" aria-hidden="true">
          H
        </span>
        <span className="hud__nombre">
          hacker<b>exp</b>
        </span>
      </Link>

      <div className="hud__lecturas">
        {dashboard && (
          <>
            <span className="hud__lectura">
              <span className="micro">
                <Icono nombre="dinero" tamano={12} /> fondos
              </span>
              <span className="dato">{formatMoney(dashboard.money)}</span>
            </span>
            <span className="hud__lectura">
              <span className="micro">
                <Icono nombre="tiempo" tamano={12} /> en marcha
              </span>
              <span className={tareas ? 'dato dato--tiempo' : 'dato'}>{tareas}</span>
            </span>
            <span className="hud__lectura">
              <span className="micro">correo</span>
              <span className="dato">{dashboard.unread_mail}</span>
            </span>
          </>
        )}
      </div>

      <div className="hud__sesion">
        {admin && (
          <Link className="hud__panel" to="/admin">
            panel
          </Link>
        )}

        {user && (
          <span className="hud__usuario">
            <span className="micro">ip</span>
            <button
              type="button"
              className={revelada ? 'hud__ip is-shown' : 'hud__ip'}
              aria-pressed={revelada}
              aria-label={revelada ? 'Ocultar tu dirección' : 'Mostrar tu dirección'}
              title={revelada ? 'Ocultar tu dirección' : 'Mostrar tu dirección'}
              onClick={() => setRevelada((actual) => !actual)}
            >
              {revelada ? formatIp(user.game_ip) : maskIp(user.game_ip)}
            </button>
            <span className="hud__login">{user.login}</span>
          </span>
        )}

        <button className="hud__salir" type="button" onClick={() => dispatch(signedOut())}>
          salir
        </button>
      </div>
    </header>
  );
}
