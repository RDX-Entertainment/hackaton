/**
 * Every screen turns a rejected request into something the player can read.
 * It used to be the same ternary copied into two dozen catch blocks.
 */
const FALLBACK_MESSAGE = 'Request failed';

export function messageFrom(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message;
  }

  if (typeof error === 'string' && error) {
    return error;
  }

  return FALLBACK_MESSAGE;
}
