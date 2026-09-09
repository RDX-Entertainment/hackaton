import { get } from './client';
import type { Onboarding } from './types';

export function getOnboarding(): Promise<Onboarding> {
  return get<Onboarding>('/onboarding');
}
