import { useState } from 'react';

import type { Component, ComponentKind, MachineDetail } from '../../utils/api';
import { formatCount } from '../../utils/format';
import { PartIcon } from '../admin/PartIcon';

const UNIT: Record<ComponentKind, string> = {
  cpu: 'MHz',
  ram: 'MB',
  hdd: 'MB',
  net: 'Mbit',
  mobo: '',
};

function specOf(part: Component): string {
  if (part.kind === 'mobo') {
    return `${part.chipset ?? 'sin chipset'} · DDR${part.generation} · ${part.cpu_slots}c ${part.ram_slots}r ${part.hdd_slots}d ${part.net_slots}n`;
  }

  const base = `${formatCount(part.power)} ${UNIT[part.kind]}`;

  return part.kind === 'ram' ? `${base} · DDR${part.generation}` : base;
}

type DrawerFolderProps = {
  kind: ComponentKind;
  label: string;
  parts: Component[];
  machines: MachineDetail[];
  open: boolean;
  busy: boolean;
  onToggle: () => void;
  onInstall: (component: Component, machine: number) => void;
};

export function DrawerFolder({
  kind,
  label,
  parts,
  machines,
  open,
  busy,
  onToggle,
  onInstall,
}: DrawerFolderProps) {
  const [target, setTarget] = useState<number | null>(null);

  const empty = parts.length === 0;
  const single = machines.length === 1;

  const install = (part: Component, machine: number) => {
    setTarget(null);
    onInstall(part, machine);
  };

  return (
    <li className={open && !empty ? 'folder is-open' : 'folder'}>
      <button
        type="button"
        className="folder__head"
        aria-expanded={open && !empty}
        disabled={empty}
        onClick={onToggle}
      >
        <span className="folder__chevron" aria-hidden="true">
          ▸
        </span>

        <PartIcon kind={kind} className="folder__icon" />

        <span className="folder__label">{label}</span>

        <span className="folder__count">{parts.length}</span>
      </button>

      {open && !empty && (
        <ul className="folder__list">
          {parts.map((part) => (
            <li className="piece" key={part.id}>
              <span className="piece__text">
                <span className="piece__name">{part.name}</span>
                <span className="piece__spec">{specOf(part)}</span>
              </span>

              <span className={`tier tier--${part.tier}`}>{part.tier}</span>

              <span className="piece__actions">
                {single ? (
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() => install(part, machines[0].machine.id)}
                  >
                    poner
                  </button>
                ) : (
                  <button
                    type="button"
                    className={target === part.id ? 'piece__pick is-open' : 'piece__pick'}
                    aria-expanded={target === part.id}
                    disabled={busy}
                    onClick={() => setTarget(target === part.id ? null : part.id)}
                  >
                    poner en <span aria-hidden="true">▾</span>
                  </button>
                )}
              </span>

              {target === part.id && !single && (
                <ul className="piece__targets">
                  {machines.map((machine) => (
                    <li key={machine.machine.id}>
                      <button
                        type="button"
                        disabled={busy}
                        onClick={() => install(part, machine.machine.id)}
                      >
                        <span className="piece__target">{machine.machine.name}</span>
                        <small>
                          {machine.board ? machine.board.name : 'sin placa'}
                        </small>
                      </button>
                    </li>
                  ))}
                </ul>
              )}
            </li>
          ))}
        </ul>
      )}
    </li>
  );
}
