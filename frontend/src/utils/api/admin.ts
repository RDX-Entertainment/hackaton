import { del, get, post, put } from './client';
import type {
  AdminUser,
  BankAccount,
  Chipset,
  CreatedRecord,
  Kit,
  KitClan,
  KitInput,
  Organization,
  OrganizationInput,
  OrganizationNetworkInput,
  OrganizationNpcCatalog,
  CreateOrganizationServerInput,
  OrganizationComputerBuildInput,
  OrganizationComputerConfiguration,
  OrganizationServerCreated,
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

export function listOrganizations(): Promise<Organization[]> { return get<Organization[]>('/admin/organizations'); }
export function createOrganization(input: OrganizationInput): Promise<CreatedRecord> { return post<CreatedRecord>('/admin/organizations', input); }
export function editOrganization(id: number, input: OrganizationInput): Promise<void> { return put<void>(`/admin/organizations/${id}`, input); }
export function removeOrganization(id: number): Promise<void> { return del<void>(`/admin/organizations/${id}`); }
export function listOrganizationNpcs(): Promise<OrganizationNpcCatalog[]> { return get<OrganizationNpcCatalog[]>('/admin/organizations/npcs'); }
export function createOrganizationNetwork(organization: number, input: OrganizationNetworkInput): Promise<CreatedRecord> { return post<CreatedRecord>(`/admin/organizations/${organization}/networks`, input); }
export function editOrganizationNetwork(organization: number, network: number, input: OrganizationNetworkInput): Promise<void> { return put<void>(`/admin/organizations/${organization}/networks/${network}`, input); }
export function removeOrganizationNetwork(organization: number, network: number): Promise<void> { return del<void>(`/admin/organizations/${organization}/networks/${network}`); }
export function assignOrganizationNpc(organization: number, network: number, input: { npc_id: number; role: string }): Promise<void> { return post<void>(`/admin/organizations/${organization}/networks/${network}/npcs`, input); }
export function editOrganizationNode(organization: number, network: number, npc: number, input: { map_x: number; map_y: number; role: string }): Promise<void> { return put<void>(`/admin/organizations/${organization}/networks/${network}/npcs/${npc}`, input); }

/** Saca el nodo de la red y corta sus enlaces LAN. El NPC sobrevive: puede volver a asignarse. */
export function removeOrganizationNode(organization: number, network: number, npc: number): Promise<void> { return del<void>(`/admin/organizations/${organization}/networks/${network}/npcs/${npc}`); }
/** Las cuentas que una organización tiene abiertas, en cualquier banco. */
export function listOrganizationAccounts(organization: number): Promise<BankAccount[]> { return get<BankAccount[]>(`/admin/organizations/${organization}/accounts`); }
export function openOrganizationAccount(organization: number, input: { bank_id: number; password: string }): Promise<{ account: number }> { return post<{ account: number }>(`/admin/organizations/${organization}/accounts`, input); }
export function listUserAccounts(user: number): Promise<BankAccount[]> { return get<BankAccount[]>(`/admin/users/${user}/accounts`); }
export function openUserAccount(user: number, input: { bank_id: number; password: string }): Promise<{ account: number }> { return post<{ account: number }>(`/admin/users/${user}/accounts`, input); }

/** Ingresar o retirar: el importe va con signo. Es dinero creado a mano desde el panel. */
export function adjustAccountCash(account: number, amount: number): Promise<{ cash: number }> { return post<{ cash: number }>(`/admin/accounts/${account}/cash`, { amount }); }
export function saveOrganizationComputer(organization: number, network: number, npc: number, input: OrganizationComputerBuildInput): Promise<void> { return put<void>(`/admin/organizations/${organization}/networks/${network}/npcs/${npc}/computer`, input); }
export function getOrganizationComputer(organization: number, network: number, npc: number): Promise<OrganizationComputerConfiguration> { return get<OrganizationComputerConfiguration>(`/admin/organizations/${organization}/networks/${network}/npcs/${npc}/computer`); }
export function createOrganizationLink(organization: number, input: { from_npc_id: number; to_npc_id: number }): Promise<CreatedRecord> { return post<CreatedRecord>(`/admin/organizations/${organization}/links`, input); }
export function removeOrganizationLink(organization: number, link: number): Promise<void> { return del<void>(`/admin/organizations/${organization}/links/${link}`); }
export function createOrganizationServer(organization: number, network: number, input: CreateOrganizationServerInput): Promise<OrganizationServerCreated> { return post<OrganizationServerCreated>(`/admin/organizations/${organization}/networks/${network}/servers`, input); }
