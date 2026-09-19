import { tr } from '../i18n/text';
import { DESTINOS } from './navigation';

/**
 * Los sitios que el tutorial puede señalar.
 *
 * Un ancla es un `data-foco` puesto a mano en un botón del juego, y esta lista es la de los
 * que existen. **Vive en el cliente a propósito**: el servidor no sabe —ni tiene por qué
 * saber— qué botones hay en cada pantalla, y la lista de rutas ya vive aquí por lo mismo.
 *
 * El panel elige de aquí en vez de dejar escribir un texto libre. Un ancla inventada no
 * rompería nada, y eso es exactamente el problema: no se vería el fallo, se vería un tutorial
 * que un día dejó de señalar y nadie sabría desde cuándo.
 *
 * Añadir un ancla son dos líneas: el `data-foco` en el botón y su entrada aquí.
 */
export type Foco = {
  /** El mismo código que la acción que ese botón dispara: el paso resalta lo que deja hacer. */
  code: string;
  label: string;
  /** La pantalla donde está, para que el panel proponga el foco y la vista a la vez. */
  view: string;
};

export const FOCOS: Foco[] = [
  { code: 'lab.investigar', get label() { return tr("Lab · botón de investigar"); }, view: 'lab' },
  { code: 'lab.disenar', get label() { return tr("Lab · guardar el diseño"); }, view: 'lab' },
  { code: 'lab.compilar', get label() { return tr("Lab · compilar un diseño"); }, view: 'lab' },
  { code: 'discos.cargar', get label() { return tr("Discos · cargar en memoria"); }, view: 'software' },
  { code: 'discos.descargar', get label() { return tr("Discos · sacar de memoria"); }, view: 'software' },
  { code: 'discos.ocultar', get label() { return tr("Discos · ocultar o revelar"); }, view: 'software' },
  { code: 'red.entrar', get label() { return tr("Internet · entrar en la máquina"); }, view: 'internet' },
  { code: 'red.hackear', get label() { return tr("Internet · romper la contraseña"); }, view: 'internet' },
];

/** Todas las pantallas del juego, con el nombre que lee el jugador y su destino. */
export const VISTAS = DESTINOS.flatMap((destino) =>
  destino.items.map((item) => ({
    path: item.path,
    get label() { return item.label; },
    get destino() { return destino.nombre; },
  })),
);

export function nombreDeVista(path: string): string {
  return VISTAS.find((vista) => vista.path === path)?.label ?? path;
}
