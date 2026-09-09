import { useEffect } from 'react';
import { NavLink, useLocation, useNavigate } from 'react-router-dom';

import { DESTINOS, destinoDe } from '../../app/navigation';

/**
 * El carril de destinos: cinco teclas, y ninguna se abre al pasar el ratón.
 *
 * El menú anterior era un desplegable con 140 ms de gracia para el ratón, lo
 * que en una pantalla táctil no existe: a 390 px el juego no se navegaba. Aquí
 * cada destino es un enlace de verdad, y lleva su número porque también se
 * salta con el teclado —pulsar 1 a 5 va a ese destino—.
 *
 * Debajo, la tira con las pantallas del destino en el que estás. Dos niveles y
 * ninguno escondido.
 */
export function Carril() {
  const location = useLocation();
  const navigate = useNavigate();
  const actual = destinoDe(location.pathname);

  useEffect(() => {
    const alPulsar = (evento: KeyboardEvent) => {
      // Escribir una IP en un campo no puede cambiarte de pantalla.
      const destinoDelFoco = evento.target as HTMLElement | null;
      const escribiendo =
        destinoDelFoco?.tagName === 'INPUT' ||
        destinoDelFoco?.tagName === 'TEXTAREA' ||
        destinoDelFoco?.tagName === 'SELECT' ||
        destinoDelFoco?.isContentEditable;

      if (escribiendo || evento.metaKey || evento.ctrlKey || evento.altKey) {
        return;
      }

      const elegido = DESTINOS.find((destino) => destino.tecla === evento.key);

      if (elegido) {
        navigate(`/${elegido.items[0].path}`);
      }
    };

    window.addEventListener('keydown', alPulsar);
    return () => window.removeEventListener('keydown', alPulsar);
  }, [navigate]);

  return (
    <nav className="carril" aria-label="Destinos del juego">
      {DESTINOS.map((destino) => (
        <NavLink
          key={destino.nombre}
          to={`/${destino.items[0].path}`}
          className={
            destino.nombre === actual?.nombre ? 'carril__tecla is-current' : 'carril__tecla'
          }
          aria-current={destino.nombre === actual?.nombre ? 'page' : undefined}
        >
          <span className="carril__numero" aria-hidden="true">
            {destino.tecla}
          </span>
          <span className="carril__nombre">{destino.nombre}</span>
        </NavLink>
      ))}
    </nav>
  );
}

/** Las pantallas del destino en el que estás. Vacía fuera de ellos. */
export function Tira() {
  const location = useLocation();
  const actual = destinoDe(location.pathname);

  if (!actual) {
    return null;
  }

  return (
    <nav className="tira" aria-label={`Pantallas de ${actual.nombre}`}>
      {actual.items.map((item) => (
        <NavLink
          key={item.path}
          to={`/${item.path}`}
          className={({ isActive }) => (isActive ? 'tira__paso is-active' : 'tira__paso')}
        >
          {item.label}
        </NavLink>
      ))}
    </nav>
  );
}
