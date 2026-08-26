import { get } from './client';
import type { RankedPlayer } from './types';

/**
 * El ranking es, de momento, la única lista de jugadores que sirve el backend, y una
 * operación modular pide un jugador por id. Hasta que haya una búsqueda de verdad, esto es
 * lo que llena el selector de objetivo.
 */
export function listPlayers(): Promise<RankedPlayer[]> {
  return get<RankedPlayer[]>('/ranking/users');
}
