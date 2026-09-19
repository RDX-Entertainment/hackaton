import { tr } from '../../i18n/text';
import { get, post } from './client';
import type { ProgramStats } from './types';
export type ModuleConfig = {
  allowed_capabilities: string[]; allowed_slots: number[]; min_level: number;
  max_copies: number; excludes: string[]; grants: string[]; active: boolean;
  collectable: boolean; deltas: Partial<ProgramStats>;
};
export type ModuleTemplate = { id?: number; code: string; name: string; config: ModuleConfig };
export type ModuleStock = { module_id: number; tier: number; quantity: number };
export type EquippedModule = { software_id: number; slot: number; module_id: number; tier: number;
  snapshot: { code: string; name: string; deltas: Partial<ProgramStats>; grants: string[]; excludes: string[] } };
export type Workshop = { catalog: ModuleTemplate[]; stock: ModuleStock[]; equipped: EquippedModule[];
  slot_levels: number[]; grades: string[] };
export const loadWorkshop = () => get<Workshop>('/software-system/workshop');
export const equipModule = (software_id: number, slot: number, module_id: number, tier: number) => post('/software-system/workshop/equip', { software_id, slot, module_id, tier });
export const unequipModule = (software_id: number, slot: number) => post('/software-system/workshop/unequip', { software_id, slot });
export const fuseModule = (module_id: number, tier: number) => post('/software-system/workshop/fuse', { module_id, tier });
export const moduleCatalog = () => get<ModuleTemplate[]>('/admin/software-modules');
export const saveModule = (template: ModuleTemplate) => post('/admin/software-modules', template);
export const multipliers = [1, 1.35, 1.8, 2.4, 3.2];
export const statNames: Record<string,string> = { get effectiveness() { return tr("Eficacia"); }, get resistance() { return tr("Resistencia"); }, get stealth() { return tr("Sigilo"); }, get detection() { return tr("Detección"); }, get removal() { return tr("Limpieza"); }, get penetration() { return tr("Penetración"); }, get hardening() { return tr("Blindaje"); }, get manipulation() { return tr("Manipulación"); }, get integrity() { return tr("Integridad"); }, get anti_forensics() { return tr("Antiforense"); }, get forensics() { return tr("Forense"); }, get persistence() { return tr("Persistencia"); }, get stability() { return tr("Estabilidad"); }, cpu:'CPU', ram:'RAM', get net() { return tr("Red"); }, get weight() { return tr("Disco"); } };
export function moduleDelta(deltas: Partial<ProgramStats>, tier = 0) {
  return Object.entries(deltas).filter(([,v]) => v !== 0).map(([k,v]) => {
    const amount = Math.sign(v!) * Math.round(Math.abs(v!) * multipliers[tier]);
    return `${amount > 0 ? '+' : ''}${amount} ${statNames[k] ?? k}`;
  }).join(' · ');
}
