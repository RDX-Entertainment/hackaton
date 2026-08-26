/**
 * El sistema modular de software, que el backend sirve bajo `/software-system/`.
 *
 * No sustituye a `software.ts`: aquel lista los ficheros del disco y los instala, y sigue
 * siendo el que carga en memoria lo que aquí se compila.
 */
import { get, post } from './client';
import type {
  Blueprint,
  Capability,
  CompilationQueued,
  OperationStarted,
  Program,
  ResearchModule,
} from './types';

export function listCapabilities(): Promise<Capability[]> {
  return get<Capability[]>('/software-system/capabilities');
}

export function listModules(): Promise<ResearchModule[]> {
  return get<ResearchModule[]>('/software-system/research/modules');
}

export function listPrograms(): Promise<Program[]> {
  return get<Program[]>('/software-system/programs');
}

export function listBlueprints(): Promise<Blueprint[]> {
  return get<Blueprint[]>('/software-system/blueprints');
}

export function createBlueprint(
  name: string,
  capability: string,
  moduleIds: number[],
): Promise<Blueprint> {
  return post<Blueprint>('/software-system/blueprints', {
    name,
    capability,
    module_ids: moduleIds,
  });
}

/** Compilar encola el trabajo; el fichero aparece al terminar en Procesos. */
export function compile(blueprintId: number): Promise<CompilationQueued> {
  return post<CompilationQueued>('/software-system/compile', { blueprint_id: blueprintId });
}

/**
 * Lanzar una operación encola un proceso; el worker lo resuelve al vencer, y es entonces
 * cuando cambia algo. El resultado se lee en Procesos.
 */
export function startOperation(
  capability: string,
  targetUserId: number,
): Promise<OperationStarted> {
  return post<OperationStarted>('/software-system/operations', {
    capability,
    target_user_id: targetUserId,
  });
}
