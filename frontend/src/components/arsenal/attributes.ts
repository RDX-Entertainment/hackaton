import type { Capability, ResearchModule } from '../../utils/api';

export function visibleAttributes(capability?: Capability): string[] {
  return [...new Set(['stability', ...(capability?.relevant_attributes ?? []), ...(capability?.secondary_attributes ?? [])])];
}

export const resourceAttributes = ['cpu', 'ram', 'net', 'weight'];

export function usefulModule(module: ResearchModule, capability?: Capability): boolean {
  if (!capability || (module.capability && module.capability !== capability.code)) return false;
  const benefits = Object.entries(module.deltas).filter(([key, value]) => value !== 0 && !resourceAttributes.includes(key));
  return !benefits.length || benefits.some(([key]) => visibleAttributes(capability).includes(key));
}
