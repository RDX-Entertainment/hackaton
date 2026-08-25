/**
 * Every screen turns a rejected request into something the player can read.
 * It used to be the same ternary copied into two dozen catch blocks.
 */
import { HttpError } from './api/client';

const FALLBACK_MESSAGE = 'Request failed';

const NOT_FOUND = 404;

/** Nada respondio en esa direccion, que no es lo mismo que haber fallado al preguntar. */
export function isNotFound(error: unknown): boolean {
  return error instanceof HttpError && error.status === NOT_FOUND;
}

export function messageFrom(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message;
  }

  if (typeof error === 'string' && error) {
    return error;
  }

  return FALLBACK_MESSAGE;
}
