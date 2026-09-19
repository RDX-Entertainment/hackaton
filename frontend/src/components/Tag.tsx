import { tr, useIdioma } from '../i18n/text';
/**
 * El vocabulario del circuito, en una sola pieza.
 *
 * Mismo estado, misma palabra, mismo color y misma forma en todas las
 * pantallas: si «en memoria» se dice distinto en Arsenal y en Equipo, el
 * jugador tiene que aprender dos juegos.
 *
 * Cada estado se distingue además por **forma** —relleno, filo, punteado—,
 * porque en una pantalla monocroma el color solo no llega, y hay gente que no
 * lo ve.
 */
export type Estado = 'memoria' | 'disco' | 'plano' | 'obra' | 'mal' | 'off';

const PALABRA: Record<Estado, string> = {
  get memoria() { return tr("en memoria"); },
  get disco() { return tr("en disco"); },
  get plano() { return tr("solo diseño"); },
  get obra() { return tr('compilando'); },
  get mal() { return tr("no cabe"); },
  get off() { return tr('bloqueado'); },
};

type TagProps = {
  estado: Estado;
  /** Sustituye a la palabra por defecto sin cambiar el color ni la forma. */
  children?: string;
};

export function Tag({ estado, children }: TagProps) {
  useIdioma();
  return <span className={`tag tag--${estado}`}>{children ?? PALABRA[estado]}</span>;
}
