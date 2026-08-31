import { get, put } from './client';
import type { OwnedFile, SoftwareType } from './types';

export function listSoftware(): Promise<OwnedFile[]> {
  return get<OwnedFile[]>('/software');
}

export function listSoftwareTypes(): Promise<SoftwareType[]> {
  return get<SoftwareType[]>('/game/software-types');
}

/** Llevar un fichero a otro equipo. Hace falta cable entre ambos, o salida a Internet en los dos. */
export function moveSoftware(id: number, serverId: number): Promise<void> {
  return put<void>(`/software/${id}/machine`, { server_id: serverId });
}
