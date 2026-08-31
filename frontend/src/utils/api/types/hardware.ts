export type Tier = 'domestica' | 'gaming' | 'server' | 'industrial';

export type ComponentKind = 'cpu' | 'ram' | 'hdd' | 'net' | 'mobo';

export type Machine = {
  id: number;
  name: string;
  cpu: number;
  hdd: number;
  ram: number;
  net: number;
  /** Donde esta puesto en el mapa de nodos, en pixeles desde la esquina del lienzo. */
  map_x: number;
  map_y: number;
  /** Como se la alcanza desde fuera. `null` = sin tarjeta de red, no sale a Internet. */
  public_ip: number | null;
  /** Como se la nombra dentro de la red de su dueno. La tienen todas. */
  lan_ip: number | null;
};

/**
 * Un cable LAN entre dos equipos. No tiene direccion: `from` y `to` son solo el orden en
 * que se tendio, y el backend trata A-B y B-A como el mismo cable.
 */
export type MachineLink = {
  id: number;
  from_server: number;
  to_server: number;
};

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

