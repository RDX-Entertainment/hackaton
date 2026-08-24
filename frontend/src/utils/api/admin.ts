import { del, get, post, put } from './client';
import type {
  AdminUser,
  Chipset,
  CreatedRecord,
  Kit,
  KitClan,
  KitInput,
  Part,
  PartInput,
  Site,
} from './types';

export function listParts(): Promise<Part[]> {
  return get<Part[]>('/admin/parts');
}

export function createPart(input: PartInput): Promise<CreatedRecord> {
  return post<CreatedRecord>('/admin/parts', input);
}

export function editPart(id: number, input: PartInput): Promise<void> {
  return put<void>(`/admin/parts/${id}`, input);
}

export function removePart(id: number): Promise<void> {
  return del<void>(`/admin/parts/${id}`);
}

export function grantComponents(
  part: number,
  user: number,
  amount: number,
): Promise<{ components: number[] }> {
  return post('/admin/components', { part, user, amount });
}

/**
 * The admin routes answer 404 to everyone else, so asking is how the client
 * finds out. There is no is_admin flag on the session and there should not be:
 * the server already decided, and a flag would only be a second opinion.
 */
export async function isAdmin(): Promise<boolean> {
  try {
    await listParts();
    return true;
  } catch {
    return false;
  }
}

export function listUsers(): Promise<AdminUser[]> {
  return get<AdminUser[]>('/admin/users');
}

export function setAdminFlag(id: number, admin: boolean): Promise<void> {
  return put<void>(`/admin/users/${id}/admin`, { admin });
}

export function listChipsets(): Promise<Chipset[]> {
  return get<Chipset[]>('/admin/chipsets');
}

export function createChipset(name: string): Promise<CreatedRecord> {
  return post<CreatedRecord>('/admin/chipsets', { name });
}

export function removeChipset(id: number): Promise<void> {
  return del<void>(`/admin/chipsets/${id}`);
}

export function listKits(): Promise<Kit[]> {
  return get<Kit[]>('/admin/kits');
}

export function createKit(input: KitInput): Promise<CreatedRecord> {
  return post<CreatedRecord>('/admin/kits', input);
}

export function editKit(id: number, input: KitInput): Promise<void> {
  return put<void>(`/admin/kits/${id}`, input);
}

export function removeKit(id: number): Promise<void> {
  return del<void>(`/admin/kits/${id}`);
}

/**
 * Marks the kit every new player is born with. The server allows only one, so
 * turning this on anywhere turns it off everywhere else.
 */
export function setDefaultKit(id: number, on: boolean): Promise<void> {
  return put<void>(`/admin/kits/${id}/default`, { on });
}

export function listClans(): Promise<KitClan[]> {
  return get<KitClan[]>('/admin/clans');
}

export function listSites(): Promise<Site[]> {
  return get<Site[]>('/admin/sites');
}

export function saveSite(kind: Site['kind'], id: number, web: string): Promise<void> {
  return put<void>(`/admin/sites/${kind}/${id}`, { web });
}
