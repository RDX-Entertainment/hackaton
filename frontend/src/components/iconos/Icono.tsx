/**
 * Los doce iconos del sistema, y para de contar.
 *
 * Trazo de 2 px sobre una rejilla de 24, uniones y esquinas a escuadra —el
 * mismo filo duro que el resto del sistema— y siempre en `currentColor`, para
 * que un icono dentro de un rótulo apagado no grite más que su palabra.
 *
 * Cada icono nuevo hay que ganárselo: el juego es de palabras, y un pictograma
 * de más es ruido. Nunca emoji.
 */
export type NombreDeIcono =
  | 'equipo'
  | 'disco'
  | 'memoria'
  | 'red'
  | 'cerrado'
  | 'llave'
  | 'vigilar'
  | 'lanzar'
  | 'borrar'
  | 'aviso'
  | 'tiempo'
  | 'dinero';

const TRAZOS: Record<NombreDeIcono, string> = {
  equipo: 'M3 5h18v12H3zM8 21h8M12 17v4',
  disco: 'M3 7h18v10H3zM7 11h2M7 13h6',
  memoria: 'M4 8h16v8H4zM8 8V5M12 8V5M16 8V5',
  red: 'M12 4a8 8 0 1 0 0 16 8 8 0 0 0 0-16M4 12h16M12 4c3 4 3 12 0 16',
  cerrado: 'M5 11h14v9H5zM8 11V7a4 4 0 0 1 8 0v4',
  llave: 'M9 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8M13 12h8M18 12v4',
  vigilar: 'M2 12s4-6 10-6 10 6 10 6-4 6-10 6S2 12 2 12zM12 9.5a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5',
  lanzar: 'M13 2 4 14h7l-1 8 9-12h-7z',
  borrar: 'M4 7h16M9 7V4h6v3M6 7l1 14h10l1-14',
  aviso: 'M12 3 2 20h20L12 3zM12 10v4M12 17v.5',
  tiempo: 'M12 4a8 8 0 1 0 0 16 8 8 0 0 0 0-16M12 7v5l3 2',
  dinero: 'M12 3v18M8 7h6a3 3 0 0 1 0 6H9a3 3 0 0 0 0 6h7',
};

type IconoProps = {
  nombre: NombreDeIcono;
  /** En píxeles. 16 dentro de una línea de texto, 20 en un rótulo, 24 suelto. */
  tamano?: number;
  /** Lo que lee quien no ve el icono. Sin él, el icono es decorativo y se calla. */
  titulo?: string;
};

export function Icono({ nombre, tamano = 20, titulo }: IconoProps) {
  return (
    <svg
      className="icono"
      width={tamano}
      height={tamano}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="square"
      strokeLinejoin="miter"
      role={titulo ? 'img' : undefined}
      aria-hidden={titulo ? undefined : true}
    >
      {titulo && <title>{titulo}</title>}
      <path d={TRAZOS[nombre]} />
    </svg>
  );
}
