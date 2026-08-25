/**
 * The shapes the API answers with. One file so a screen never has to guess
 * which endpoint module owns a type.
 */

export type User = {
  id: number;
  login: string;
  email: string;
  game_ip: number;
  premium: boolean;
};

/** What the server hands back on login, and what we keep in localStorage. */
export type Session = {
  token: string | null;
  user: User | null;
};

/** The same thing, once authentication has actually succeeded. */
export type AuthenticatedSession = {
  token: string;
  user: User;
};

/** A generic row from an endpoint we have not given a typed screen yet. */
export type UnknownRow = Record<string, unknown>;

export type Dashboard = {
  user: User;
  money: number;
  processes: number;
  unread_mail: number;
};

export type Page = {
  page: string;
  user_id: number;
  status: string;
};

export type News = {
  id: number;
  author: string;
  title: string;
  content: string;
  published_at: string;
  type: number;
};

export type Target = {
  id: number;
  ip: number;
  name: string | null;
  type: number;
  web: string | null;
};

export type RemoteAnalysis = {
  cpu_min: number;
  cpu_max: number;
  ram_min: number;
  ram_max: number;
};

/** Lo que una página declara que se puede hacer, leído del bloque `<action>`. */
export type SiteAction = {
  kind: string;
  target: string;
  label: string;
  params?: Record<string, string>;
};

export type SitePage = {
  id: string;
  title: string;
  html: string;
  actions: SiteAction[];
};

/**
 * El sitio ya leído por el servidor: sus páginas y lo que ofrece hacer. Viaja aparte del
 * HTML porque el marco donde se pinta está aislado y no puede avisar de un clic, así que la
 * navegación la dibuja el juego por fuera.
 *
 * No confundir con `Site`, que es la ficha con la que el panel administra las páginas.
 */
export type SiteDocument = {
  pages: SitePage[];
  redirect: string | null;
};

export type Remote = {
  ip: number;
  kind: 'npc' | 'player';
  id: number;
  name: string;
  type: number;
  down: boolean;
  access: boolean;
  bookmarked: boolean;
  scanned: boolean;
  analysis: RemoteAnalysis | null;
  /** The page this machine serves. Rendered in a sandboxed frame, never inline. */
  web: string | null;
  /** Presente solo cuando el sitio declara navegación o acciones. */
  site: SiteDocument | null;
  /** El banco al que pertenece esta máquina, si es de alguno. Lo dice el servidor. */
  bank: { id: number; name: string } | null;
  /**
   * Las órdenes que esta web tiene permitido usar. Sale de las concesiones del juego, no de
   * la página: escribir una acción en el HTML no sirve de nada si no está concedida.
   */
  allowed: string[];
};

export type HackResult = {
  process_id: number;
  action: number;
  seconds: number;
  attacker_version: number;
  defender_version: number;
};

export type RemoteFile = {
  id: number;
  name: string;
  version: number;
  size: number;
  ram: number;
  type: number;
  folder: boolean;
};

/** A file on our own disk knows things a file on someone else's does not. */
export type OwnedFile = RemoteFile & {
  hidden: boolean;
  hidden_with: number;
  running: boolean;
};

export type Visited = {
  ip: number;
  name: string;
  visited_at?: string;
};

export type Machine = {
  id: number;
  name: string;
  cpu: number;
  hdd: number;
  ram: number;
  net: number;
};

export type Tier = 'domestica' | 'gaming' | 'server' | 'industrial';

export type Chipset = {
  id: number;
  name: string;
};

export type Part = {
  id: number;
  kind: ComponentKind;
  name: string;
  power: number;
  price: number;
  generation: number;
  tier: Tier;
  cpu_slots: number;
  ram_slots: number;
  hdd_slots: number;
  net_slots: number;
  max_cpu: number;
  max_ram: number;
  max_hdd: number;
  max_net: number;
  chipset_id: number | null;
  chipset: string | null;
  listed: boolean;
};

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

export type ComponentKind = 'cpu' | 'ram' | 'hdd' | 'net' | 'mobo';

export type Component = {
  id: number;
  part_id: number;
  kind: ComponentKind;
  name: string;
  power: number;
  generation: number;
  tier: Tier;
  installed_in: number | null;
  cpu_slots: number;
  ram_slots: number;
  hdd_slots: number;
  net_slots: number;
  max_cpu: number;
  max_ram: number;
  max_hdd: number;
  max_net: number;
  chipset_id: number | null;
  chipset: string | null;
};

export type Slots = {
  cpu_used: number;
  cpu_total: number;
  ram_used: number;
  ram_total: number;
  hdd_used: number;
  hdd_total: number;
  net_used: number;
  net_total: number;
  cpu_power: number;
  cpu_max: number;
  ram_power: number;
  ram_max: number;
  hdd_power: number;
  hdd_max: number;
  net_power: number;
  net_max: number;
  generation: number;
  tier: Tier | '';
};

export type MachineDetail = {
  machine: Machine;
  board: Component | null;
  components: Component[];
  slots: Slots;
};

export type PartInput = {
  kind: ComponentKind;
  name: string;
  power: number;
  price: number;
  generation: number;
  tier: Tier;
  chipset_id: number | null;
  cpu_slots: number;
  ram_slots: number;
  hdd_slots: number;
  net_slots: number;
  max_cpu: number;
  max_ram: number;
  max_hdd: number;
  max_net: number;
  listed: boolean;
};


export type SoftwareType = {
  id: number;
  name: string;
};

export type ProcessAction = {
  id: number;
  name: string;
};

export type ProcessCommand = 'pause' | 'resume' | 'delete';

export type MissionCommand = 'accept' | 'abort';

export type HackKind = 'bf' | 'scan';

export type TransferDirection = 'download' | 'upload';

export type WikiSummary = {
  slug: string;
  title: string;
  updated_at: string;
};

export type WikiPage = WikiSummary & {
  content: string;
  author: string | null;
  created_at: string;
};

export type ForumTopic = {
  id: number;
  title: string;
  author: string;
  posts: number;
  updated_at: string;
};

export type ForumPost = {
  id: number;
  content: string;
  author: string;
  created_at: string;
};

export type ForumTopicDetail = {
  id: number;
  title: string;
  author: string;
  created_at: string;
  posts: ForumPost[];
};

export type Profile = {
  id: number;
  login: string;
  game_ip: number;
  biography: string;
  /** The page this player's machine serves. Rendered sandboxed, never inline. */
  web: string;
  /** -1 until the worker has ranked anybody. */
  rank: number;
  hacks: number;
  money_hardware: number;
  exp: number;
  /** Null when the stats row is missing rather than faked as "now". */
  date_joined: string | null;
};

export type ClanMember = {
  id: number;
  login: string;
  auth_level: number;
};

export type Clan = {
  id: number;
  name: string;
  nick: string;
  description: string;
  slots: number;
  money: number;
  power: number;
  auth_level: number;
  members: ClanMember[];
};

export type ClanSummary = {
  id: number;
  name: string;
  nick: string;
  members: number;
  slots: number;
  description: string;
};

export type ClanRequest = {
  id: number;
  login: string;
  message: string;
  created_at: string;
};

export type Wallet = {
  address: string;
  /** Satoshis: entero de siete decimales. Se formatea al pintarlo. */
  amount: number;
  created_at?: string;
};

export type Bank = {
  id: number;
  slug: string;
  name: string;
};

export type BankAccount = {
  id: number;
  account: number;
  bank_id: number;
  bank_name: string;
  cash: number;
  /** Falsa mientras el titular no haya elegido PIN. El PIN nunca viaja. */
  pin_set: boolean;
};

export type ListEntry = {
  id: number;
  ip: number;
  login: string;
  hacked_at: string;
  virus_id: number;
};

export type Certification = {
  level: number;
  name: string;
  description: string;
  price: number;
  completed: boolean;
  available: boolean;
  learning: boolean;
};

export type Certifications = {
  level: number;
  items: Certification[];
};

export type Settings = {
  login: string;
  email: string;
  premium: boolean;
  last_login: string;
};

/** Anything the server queues rather than doing on the spot. */
export type QueuedProcess = {
  process_id: number;
};

export type CreatedRecord = {
  id: number;
};

/**
 * A kit is a whole PC written down: a list of catalogue parts that, put
 * together, form a machine. Buying one assembles it; the parts stay ordinary
 * parts afterwards.
 */
export type KitScope = 'shop' | 'marketplace' | 'clan' | 'hidden';

export type KitItem = {
  part: number;
  kind: ComponentKind;
  name: string;
  power: number;
  price: number;
  tier: Tier;
  generation: number;
  chipset: string | null;
  amount: number;
};

export type KitClan = {
  id: number;
  name: string;
  nick: string;
};

export type KitTotals = {
  cpu: number;
  ram: number;
  hdd: number;
  net: number;
  /** What the same parts would cost bought one by one. */
  parts_price: number;
  pieces: number;
};

export type Kit = {
  id: number;
  name: string;
  description: string;
  price: number;
  scope: KitScope;
  is_default: boolean;
  listed: boolean;
  items: KitItem[];
  clans: KitClan[];
  totals: KitTotals;
};

export type KitInput = {
  name: string;
  description: string;
  price: number;
  scope: KitScope;
  listed: boolean;
  items: { part: number; amount: number }[];
  clans: number[];
};

export type KitBought = {
  machine: number;
  machine_name: string;
  kit: string;
  components: number[];
  paid: number;
};

/** What it takes to own another machine: how many, how many fit, what the next costs. */
export type MachineQuote = {
  machines: number;
  max: number;
  price: number;
  can_buy: boolean;
};

export type MachineBought = {
  machine: number;
  name: string;
  paid: number;
};

/** One editable page in the admin panel, whoever serves it. */
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
export type Organization = {
  id: number; slug: string; name: string; category: string; description: string; active: boolean;
  created_at: string; updated_at: string; behaviors: OrganizationBehavior[]; networks: OrganizationNetwork[];
  links: OrganizationNodeLink[];
};
export type OrganizationNodeLink = { id: number; from_npc_id: number; to_npc_id: number };
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
/** A creative node build is materialized directly on an NPC; it never consumes stock. */
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

/** Una orden del catálogo de sitios, tal como la ve el panel. */
export type ActionInfo = {
  verb: string;
  family: string;
  help: string;
};

/** Quién puede usar una orden: la familia, el alcance y a quién alcanza. */
export type SiteGrant = {
  id: number;
  action: string;
  family: string;
  scope: string;
  organization_id: number | null;
  npc_id: number | null;
  target_name: string | null;
};
