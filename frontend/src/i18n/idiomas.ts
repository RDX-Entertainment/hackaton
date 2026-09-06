/**
 * Las tres lenguas que el juego habla, y de donde salen sus frases.
 *
 * El servidor no manda prosa: manda un codigo —`NO_CERTIFICATION`, `DISK_FULL`— porque no
 * sabe en que lengua lee quien pregunta. La frase se escribe aqui.
 *
 * Eso parecia un descuido del backend, y a la cara del jugador lo era: hasta ahora la
 * pantalla enseñaba `NO_CERTIFICATION` tal cual, en un parrafo gris. Resulta ser el contrato
 * correcto, y lo unico que faltaba era este lado.
 */

export const IDIOMAS = ['es', 'pt', 'en'] as const;

export type Idioma = (typeof IDIOMAS)[number];

/** El de casa. Es tambien el respaldo cuando a otro le falta una clave. */
export const IDIOMA_BASE: Idioma = 'es';

export const NOMBRE_DEL_IDIOMA: Record<Idioma, string> = {
  es: 'Castellano',
  pt: 'Português',
  en: 'English',
};

export function esIdioma(valor: unknown): valor is Idioma {
  return typeof valor === 'string' && (IDIOMAS as readonly string[]).includes(valor);
}
