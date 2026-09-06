/**
 * Buscar una frase, y decidir en que lengua.
 *
 * La lengua vive en el servidor —`users_language`— y llega con la sesion, asi que un jugador
 * la lleva consigo entre navegadores en vez de tenerla atada a este. Aqui solo se guarda
 * cual es la de ahora.
 *
 * **Nunca sale una clave cruda.** La cadena es: la lengua del jugador, el castellano, y por
 * ultimo lo que dijo el servidor. Ese ultimo escalon es el que permite ir traduciendo poco a
 * poco: un codigo que todavia no esta en ningun catalogo enseña exactamente lo que enseñaba
 * antes, ni mejor ni peor, y el dia que se añade empieza a hablar sin tocar nada mas.
 */
import { en } from './en';
import { es } from './es';
import { IDIOMA_BASE, esIdioma, type Idioma } from './idiomas';
import { pt } from './pt';

const CATALOGOS: Record<Idioma, Record<string, string>> = { es, pt, en };

let actual: Idioma = IDIOMA_BASE;

/** La pone quien conoce la sesion. Un valor que no sea de los tres se ignora. */
export function usarIdioma(lang: unknown): void {
  if (esIdioma(lang)) {
    actual = lang;
  }
}

export function idiomaActual(): Idioma {
  return actual;
}

/**
 * La frase de una clave, o `null` si no hay ninguna en ningun catalogo. Devuelve `null` en
 * vez de la clave a proposito: quien llama sabe que enseñar cuando no hay traduccion, y
 * enseñar `error.NO_CERTIFICATION` seria peor que lo que habia.
 */
export function t(clave: string): string | null {
  return CATALOGOS[actual][clave] ?? CATALOGOS[IDIOMA_BASE][clave] ?? null;
}

export { IDIOMAS, IDIOMA_BASE, NOMBRE_DEL_IDIOMA, esIdioma } from './idiomas';
export type { Idioma } from './idiomas';
