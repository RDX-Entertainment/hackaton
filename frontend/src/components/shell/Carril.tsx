import { tr, useIdioma } from '../../i18n/text';
import { useEffect } from 'react';
import { NavLink, useLocation, useNavigate } from 'react-router-dom';

import { DESTINOS, destinoDe } from '../../app/navigation';
import { useTutorial } from '../../app/tutorial';
import { t } from '../../i18n';

const nombreDestino = (nombre: string) =>
  t(`nav.destination.${nombre.toLowerCase()}`) ?? nombre;

const nombrePantalla = (path: string, respaldo: string) =>
  t(`nav.screen.${path}`) ?? respaldo;

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
 *
 * Mientras dura el tutorial hay destinos cerrados, y **se ven cerrados**: apagados, con el
 * motivo al pasar por encima. Esconderlos dejaría a quien empieza creyendo que el juego tiene
 * dos pantallas.
 */
export function Carril() {
  useIdioma();
  const location = useLocation();
  const navigate = useNavigate();
  const actual = destinoDe(location.pathname);
  const tutorial = useTutorial();

  // Un destino está abierto si lo está alguna de sus pantallas, y el enlace lleva a la
  // primera que lo esté: mandar a Equipo cuando lo único abierto ahí es Lab y la primera de
  // la lista es Piezas sería abrir una pantalla cerrada.
  const primeraAbierta = (destino: (typeof DESTINOS)[number]) =>
    destino.items.find((item) => tutorial.permiteVista(item.path));

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
      const abierta = elegido?.items.find((item) => tutorial.permiteVista(item.path));

      // La tecla de un destino cerrado no hace nada. Llevar igualmente a la pantalla y
      // enseñarla vacía sería contar la misma historia dos veces, y peor la segunda.
      if (abierta) {
        navigate(`/${abierta.path}`);
      }
    };

    window.addEventListener('keydown', alPulsar);
    return () => window.removeEventListener('keydown', alPulsar);
  }, [navigate, tutorial]);

  return (
    <nav className="carril" aria-label={t('nav.destinations') ?? tr("Destinos del juego")}>
      {DESTINOS.map((destino) => {
        const abierta = primeraAbierta(destino);

        if (!abierta) {
          return (
            <span
              key={destino.nombre}
              className="carril__tecla is-locked"
              aria-disabled="true"
              title={tutorial.motivo}
            >
              <span className="carril__numero" aria-hidden="true">
                {destino.tecla}
              </span>
              <span className="carril__nombre">{nombreDestino(destino.nombre)}</span>
            </span>
          );
        }

        return (
          <NavLink
            key={destino.nombre}
            to={`/${abierta.path}`}
            data-guia-vistas={destino.items.map((item) => `/${item.path}`).join(' ')}
            className={
              destino.nombre === actual?.nombre ? 'carril__tecla is-current' : 'carril__tecla'
            }
            aria-current={destino.nombre === actual?.nombre ? 'page' : undefined}
          >
            <span className="carril__numero" aria-hidden="true">
              {destino.tecla}
            </span>
            <span className="carril__nombre">{nombreDestino(destino.nombre)}</span>
          </NavLink>
        );
      })}
    </nav>
  );
}

/** Las pantallas del destino en el que estás. Vacía fuera de ellos. */
export function Tira() {
  useIdioma();
  const location = useLocation();
  const actual = destinoDe(location.pathname);
  const tutorial = useTutorial();

  if (!actual) {
    return null;
  }

  return (
    <nav
      className="tira"
      aria-label={`${t('nav.screens') ?? tr("Pantallas")}: ${nombreDestino(actual.nombre)}`}
    >
      {actual.items.map((item) =>
        tutorial.permiteVista(item.path) ? (
          <NavLink
            key={item.path}
            to={`/${item.path}`}
            className={({ isActive }) => (isActive ? 'tira__paso is-active' : 'tira__paso')}
          >
            {nombrePantalla(item.path, item.label)}
          </NavLink>
        ) : (
          <span
            key={item.path}
            className="tira__paso is-locked"
            aria-disabled="true"
            title={tutorial.motivo}
          >
            {nombrePantalla(item.path, item.label)}
          </span>
        ),
      )}
    </nav>
  );
}
