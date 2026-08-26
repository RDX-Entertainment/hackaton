export type AdminUser = {
  id: number;
  login: string;
  email: string;
  game_ip: number;
  is_admin: boolean;
  premium: boolean;
  last_login: string;
  components: number;
};

export type Site = {
  kind: 'npc' | 'player';
  id: number;
  ip: number;
  name: string;
  web: string;
};

export type NetworkZone = 'public' | 'dmz' | 'internal' | 'restricted';
export type OrganizationBehavior = { code: string; config: Record<string, unknown> };
export type OrganizationNetworkNpc = {
  id: number; ip: number; type: number; name: string; role: string; source_kit_id: number | null;
  zone: NetworkZone; map_x: number; map_y: number; lan_available: number; lan_total: number; computer_components: number;
};
export type OrganizationNetwork = {
  id: number; name: string; zone: NetworkZone; created_at: string; npcs: OrganizationNetworkNpc[];
};
export type OrganizationNodeLink = { id: number; from_npc_id: number; to_npc_id: number };
export type Organization = {
  id: number; slug: string; name: string; category: string; description: string; active: boolean;
  created_at: string; updated_at: string; behaviors: OrganizationBehavior[]; networks: OrganizationNetwork[];
  links: OrganizationNodeLink[];
};
export type OrganizationInput = {
  slug: string; name: string; category: string; description: string; active: boolean;
  behaviors: OrganizationBehavior[];
};
export type OrganizationNpcCatalog = {
  id: number; ip: number; type: number; name: string; organization_id: number | null;
  network_id: number | null; zone: NetworkZone | null;
};
export type OrganizationNetworkInput = { name: string; zone: NetworkZone };
export type CreateOrganizationServerInput = {
  name: string; kit_id: number; role: string; npc_type: number; web: string;
};
export type OrganizationComputerBuildInput = {
  kit_id?: number;
  items?: { part: number; amount: number }[];
};
export type OrganizationComputerConfiguration = {
  source_kit_id: number | null;
  items: { part: number; amount: number }[];
};
export type OrganizationServerCreated = {
  npc_id: number; ip: number; password: string; machine_id: number; kit_id: number; components: number[];
};

export type ActionInfo = {
  verb: string;
  family: string;
  help: string;
};

export type SiteGrant = {
  id: number;
  action: string;
  family: string;
  scope: string;
  organization_id: number | null;
  npc_id: number | null;
  target_name: string | null;
};

