import type { ComponentKind, Part, Tier } from './api';

/**
 * A preview of what a list of parts would assemble into, and what stops it.
 *
 * The authority on compatibility is the server: `fitting.rs` decides, and the
 * panel shows whatever it answers. This mirror exists only so the form can go
 * red while you are still typing instead of after you press save. It is
 * deliberately the same five rules in the same order, so a build this accepts
 * is one the server accepts.
 */

export type BuildItem = {
  part: Part;
  amount: number;
};

export type SlotUse = {
  kind: ComponentKind;
  used: number;
  total: number;
  power: number;
  max: number;
};

export type Build = {
  board: Part | null;
  slots: SlotUse[];
  problems: string[];
  price: number;
  pieces: number;
  cpu: number;
  ram: number;
  hdd: number;
  net: number;
};

const SLOTTED: ComponentKind[] = ['cpu', 'ram', 'hdd', 'net'];

const SLOTS: Record<string, keyof Part> = {
  cpu: 'cpu_slots',
  ram: 'ram_slots',
  hdd: 'hdd_slots',
  net: 'net_slots',
};

const CEILINGS: Record<string, keyof Part> = {
  cpu: 'max_cpu',
  ram: 'max_ram',
  hdd: 'max_hdd',
  net: 'max_net',
};

export const KIND_UNIT: Record<ComponentKind, string> = {
  cpu: 'MHz',
  ram: 'MB',
  hdd: 'MB',
  net: 'Mbit',
  mobo: '',
};

export const TIER_LABEL: Record<Tier, string> = {
  domestica: 'doméstica',
  gaming: 'gaming',
  server: 'servidor',
  industrial: 'industrial',
};

export function build(items: BuildItem[]): Build {
  const boards = items.filter((item) => item.part.kind === 'mobo');
  const board = boards[0]?.part ?? null;

  const problems: string[] = [];

  const price = items.reduce((total, item) => total + item.part.price * item.amount, 0);
  const pieces = items.reduce((total, item) => total + item.amount, 0);

  const totals = { cpu: 0, ram: 0, hdd: 0, net: 0 } as Record<string, number>;

  for (const item of items) {
    if (item.part.kind !== 'mobo') {
      totals[item.part.kind] += item.part.power * item.amount;
    }
  }

  if (!board) {
    problems.push('Un kit necesita una placa base.');
  }

  if (boards.length > 1 || (boards[0]?.amount ?? 1) > 1) {
    problems.push('Un equipo lleva una placa, no varias.');
  }

  const slots: SlotUse[] = SLOTTED.map((kind) => {
    const used = items
      .filter((item) => item.part.kind === kind)
      .reduce((total, item) => total + item.amount, 0);

    return {
      kind,
      used,
      total: board ? (board[SLOTS[kind]] as number) : 0,
      power: totals[kind],
      max: board ? (board[CEILINGS[kind]] as number) : 0,
    };
  });

  if (board) {
    for (const item of items) {
      const { part } = item;

      if (part.kind === 'mobo') {
        continue;
      }

      if (part.tier !== board.tier) {
        problems.push(
          `${part.name} es ${TIER_LABEL[part.tier]} y la placa es ${TIER_LABEL[board.tier]}.`,
        );
      }

      if (part.kind === 'cpu' && part.chipset_id && part.chipset_id !== board.chipset_id) {
        problems.push(`${part.name} pide chipset ${part.chipset} y la placa es ${board.chipset}.`);
      }

      if (part.kind === 'ram' && part.generation !== board.generation) {
        problems.push(
          `${part.name} es DDR${part.generation} y la placa acepta DDR${board.generation}.`,
        );
      }
    }

    for (const slot of slots) {
      if (slot.used > slot.total) {
        problems.push(
          `${slot.used} piezas de ${slot.kind} y la placa tiene ${slot.total} ranuras.`,
        );
      }

      if (slot.power > slot.max) {
        problems.push(
          `${slot.power} ${KIND_UNIT[slot.kind]} de ${slot.kind} sobre un máximo de ${slot.max}.`,
        );
      }
    }
  }

  return {
    board,
    slots,
    problems,
    price,
    pieces,
    cpu: totals.cpu,
    ram: totals.ram,
    hdd: totals.hdd,
    net: totals.net,
  };
}
