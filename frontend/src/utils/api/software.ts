import { del, get, post, put } from './client';
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

/**
 * Crear una carpeta. No ocupa disco: es un sitio, no un fichero.
 *
 * `parent_id` a `null` la deja en la raíz del equipo, que es lo que se ve al
 * entrar en él.
 */
export function createFolder(
  name: string,
  serverId: number,
  parentId: number | null,
): Promise<{ id: number }> {
  return post('/software/folders', { name, server_id: serverId, parent_id: parentId });
}

/** Mover algo a otra carpeta **de la misma máquina**. Cambiar de equipo es `moveSoftware`. */
export function moveIntoFolder(id: number, parentId: number | null): Promise<void> {
  return put<void>(`/software/${id}/folder`, { parent_id: parentId });
}

/** Borrar una carpeta. Solo si está vacía. */
export function deleteFolder(id: number): Promise<void> {
  return del<void>(`/software/folders/${id}`);
}
