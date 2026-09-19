/**
 * Every screen turns a rejected request into something the player can read.
 * It used to be the same ternary copied into two dozen catch blocks.
 */
import { t } from '../i18n';
import { tr } from '../i18n/text';
import { HttpError } from './api/client';

const FALLBACK_MESSAGE = 'Algo ha fallado.';

const NOT_FOUND = 404;

/** Nada respondio en esa direccion, que no es lo mismo que haber fallado al preguntar. */
export function isNotFound(error: unknown): boolean {
  return error instanceof HttpError && error.status === NOT_FOUND;
}

/**
 * Lo que se le enseña al jugador cuando algo falla.
 *
 * Antes esto devolvia `error.message` tal cual, y por eso la pantalla decia
 * `NO_CERTIFICATION` en un parrafo gris. Ahora primero busca el codigo en el catalogo del
 * idioma del jugador; el mensaje del servidor se queda de respaldo para todo lo que aun no
 * esta traducido, que sigue viendose exactamente como antes.
 */
export function messageFrom(error: unknown): string {
  if (error instanceof HttpError && error.code) {
    const dicho = t(`error.${error.code}`);
    if (dicho) {
      return dicho;
    }
  }

  if (error instanceof Error && error.message) {
    return error.message;
  }

  if (typeof error === 'string' && error) {
    return error;
  }

  return tr(FALLBACK_MESSAGE);
}
