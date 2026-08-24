import { get, post } from './client';
import type { Certifications } from './types';

export function getCertifications(): Promise<Certifications> {
  return get<Certifications>('/certifications');
}

/** Free certifications take no account; paid ones are charged to the one given. */
export function startCertification(level: number, account?: number): Promise<void> {
  return post<void>(`/certifications/${level}/start`, { account });
}

export function completeCertification(level: number): Promise<void> {
  return post<void>(`/certifications/${level}/complete`);
}
