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
  es: 'Español',
  pt: 'Português',
  en: 'English',
};

/** Etiqueta BCP 47 para fechas y números; `pt` es portugués de Brasil. */
export const LOCALE_DEL_IDIOMA: Record<Idioma, string> = {
  es: 'es-ES',
  pt: 'pt-BR',
  en: 'en-US',
};

export function esIdioma(valor: unknown): valor is Idioma {
  return typeof valor === 'string' && (IDIOMAS as readonly string[]).includes(valor);
}

/**
 * El primer idioma sale del navegador. Se recorren todos los que declara, no solo el
 * primero: si el sistema trae `fr-FR, pt-BR`, el juego puede hablarle en portugués.
 */
export function idiomaDelNavegador(
  lenguas: readonly string[] = typeof navigator === 'undefined'
    ? []
    : navigator.languages.length
      ? navigator.languages
      : [navigator.language],
): Idioma {
  for (const lengua of lenguas) {
    const codigo = lengua.trim().toLowerCase().split('-')[0];
    if (esIdioma(codigo)) {
      return codigo;
    }
  }

  return IDIOMA_BASE;
}
