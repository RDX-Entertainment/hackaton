import { del, get, post } from './client';
import type { ProcessAction, ProcessCommand, ProcessSnapshot, ProcessStarted, UnknownRow } from './types';

export function listProcesses(): Promise<UnknownRow[]> {
  return get<UnknownRow[]>('/processes');
}

/** Consulta una tarea concreta; se usa para animar solo la accion que el jugador inicio. */
export function getProcess(id: number): Promise<ProcessSnapshot> {
  return get<ProcessSnapshot>(`/processes/${id}`);
}

/**
 * Queues one of the numbered legacy actions. Which of the optional arguments
 * matter depends on the action: a hack needs a victim, a software verb needs a
 * file, and a few carry a free-text note instead.
 */
export function startProcess(
  action: number,
  victimId?: number,
  softwareId?: number,
  info?: string,
): Promise<ProcessStarted> {
  return post<ProcessStarted>('/processes', {
    action,
    victim_id: victimId,
    software_id: softwareId,
    info,
  });
}

export function getProcessActions(): Promise<ProcessAction[]> {
  return get<ProcessAction[]>('/game/process-actions');
}

export function commandProcess(id: number, command: ProcessCommand): Promise<void> {
  if (command === 'delete') {
    return del<void>(`/processes/${id}`);
  }

  return post<void>(`/processes/${id}/${command}`);
}
