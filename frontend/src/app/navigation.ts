/**
 * Los cinco destinos del juego.
 *
 * Antes eran veintiocho enlaces repartidos en siete grupos que se abrían al
 * pasar el ratón —lo que en un móvil no existe— y cinco de ellos no llevaban a
 * ninguna parte. El menú estaba organizado como el PHP original: un enlace por
 * fichero heredado.
 *
 * Ahora está organizado por lo que haces. **Equipo** es un ordenador: sus
 * piezas, sus programas y lo que corre en él, que hasta hoy eran cinco
 * pantallas distintas hablando del mismo objeto. **Red** es el mundo y lo que
 * has conquistado en él. El resto cae por su propio peso.
 *
 * `path` sigue siendo la clave con la que el registro de pantallas encuentra la
 * suya, así que añadir una pantalla sigue siendo una línea aquí y otra en
 * `pageRegistry`.
 */
export type NavItem = {
  path: string;
  label: string;
};

export type Destino = {
  /** La tecla con la que se salta. El carril se recorre con el teclado. */
  tecla: string;
  nombre: string;
  items: NavItem[];
};

export const DESTINOS: Destino[] = [
  {
    tecla: '1',
    nombre: 'Equipo',
    items: [
      { path: 'hardware', label: 'Piezas' },
      { path: 'software', label: 'Discos' },
      { path: 'lab', label: 'Lab' },
      { path: 'processes', label: 'Tareas' },
    ],
  },
  {
    tecla: '2',
    nombre: 'Red',
    items: [
      { path: 'internet', label: 'Internet' },
      { path: 'accesses', label: 'Accesos' },
    ],
  },
  {
    tecla: '3',
    nombre: 'Trabajo',
    items: [
      { path: 'missions', label: 'Encargos' },
      { path: 'university', label: 'Academia' },
      { path: 'finances', label: 'Banco' },
      { path: 'bitcoin', label: 'Bitcoin' },
    ],
  },
  {
    tecla: '4',
    nombre: 'Gente',
    items: [
      { path: 'mail', label: 'Correo' },
      { path: 'clan', label: 'Clan' },
      { path: 'forum', label: 'Foro' },
      { path: 'wiki', label: 'Wiki' },
      { path: 'ranking', label: 'Ranking' },
    ],
  },
  {
    tecla: '5',
    nombre: 'Cuenta',
    items: [
      { path: 'profile', label: 'Perfil' },
      { path: 'settings', label: 'Ajustes' },
      { path: 'log', label: 'Registro' },
      { path: 'list', label: 'Listas' },
      { path: 'stats', label: 'Números' },
    ],
  },
];

/** Wiki y foro están en el menú pero tienen rutas escritas a mano. */
const A_MANO = new Set(['wiki', 'forum']);

/** Las pantallas del legado, una ruta generada por cada una. */
export const LEGACY_ITEMS: NavItem[] = DESTINOS.flatMap((destino) => destino.items).filter(
  (item) => !A_MANO.has(item.path),
);

/** A qué destino pertenece una ruta, para encender su tecla. */
export function destinoDe(pathname: string): Destino | undefined {
  const primero = pathname.split('/').filter(Boolean)[0];

  if (!primero) {
    return undefined;
  }

  return DESTINOS.find((destino) =>
    destino.items.some((item) => item.path === primero),
  );
}
