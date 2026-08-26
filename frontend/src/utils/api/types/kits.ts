import type { ComponentKind, Tier } from './hardware';

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

