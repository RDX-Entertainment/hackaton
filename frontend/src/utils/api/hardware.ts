import { del, get, post, put } from './client';
import type {
  Component,
  Kit,
  KitBought,
  Machine,
  MachineLink,
  MachineBought,
  MachineDetail,
  MachineQuote,
  Part,
} from './types';

export function listMachines(): Promise<Machine[]> {
  return get<Machine[]>('/hardware');
}

export function listMachineDetail(): Promise<MachineDetail[]> {
  return get<MachineDetail[]>('/hardware/machines');
}

export function listComponents(): Promise<Component[]> {
  return get<Component[]>('/hardware/components');
}

export function installComponent(id: number, machine: number): Promise<Component> {
  return post<Component>(`/hardware/components/${id}/install`, { machine });
}

export function uninstallComponent(id: number): Promise<Component> {
  return del<Component>(`/hardware/components/${id}/install`);
}

export function listShopParts(): Promise<Part[]> {
  return get<Part[]>('/hardware/shop');
}

export function buyPart(
  part: number,
  account: number,
): Promise<{ component: number; kind: string; name: string; power: number; paid: number }> {
  return post('/hardware/shop', { part, account });
}

/** Assembled machines this player is allowed to see: the shop plus their clan's. */
export function listKits(): Promise<Kit[]> {
  return get<Kit[]>('/hardware/kits');
}

export function buyKit(kit: number, account: number): Promise<KitBought> {
  return post<KitBought>(`/hardware/kits/${kit}/buy`, { account });
}

export function machineQuote(): Promise<MachineQuote> {
  return get<MachineQuote>('/hardware/machines/quote');
}

/** An empty machine: somewhere to put a board, not a computer yet. */
export function buyMachine(account: number, name?: string): Promise<MachineBought> {
  return post<MachineBought>('/hardware/machines', { account, name: name || null });
}

export function renameMachine(id: number, name: string): Promise<void> {
  return put<void>(`/hardware/machines/${id}`, { name });
}

export function scrapMachine(id: number): Promise<void> {
  return del<void>(`/hardware/machines/${id}`);
}

//
// El mapa de nodos.
//

export function moveMachine(id: number, mapX: number, mapY: number): Promise<void> {
  return put<void>(`/hardware/machines/${id}/position`, { map_x: mapX, map_y: mapY });
}

export function listLinks(): Promise<MachineLink[]> {
  return get<MachineLink[]>('/hardware/links');
}

export function createLink(from: number, to: number): Promise<MachineLink> {
  return post<MachineLink>('/hardware/links', { from, to });
}

export function deleteLink(id: number): Promise<void> {
  return del<void>(`/hardware/links/${id}`);
}
