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
  access_as: string | null;
  bookmarked: boolean;
  scanned: boolean;
  analysis: RemoteAnalysis | null;
  web: string | null;
  site: SiteDocument | null;
  bank: { id: number; name: string } | null;
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

export type OwnedFile = RemoteFile & {
  hidden: boolean;
  hidden_with: number;
  running: boolean;
  /**
   * En que equipo esta guardado. Un fichero propio vive en un disco concreto; los remotos
   * (`RemoteFile`) no lo llevan porque de la maquina ajena solo se ve lo que hay dentro.
   */
  server_id: number;

  /** La carpeta que lo contiene, dentro de esa máquina. `null` es la raíz del equipo. */
  parent_id: number | null;
};

export type Visited = {
  ip: number;
  name: string;
  visited_at?: string;
};
