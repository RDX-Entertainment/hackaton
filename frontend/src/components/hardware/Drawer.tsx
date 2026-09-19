import { tr, useIdioma, idiomaActual } from '../../i18n/text';
import { useMemo, useState } from 'react';

import type { Component, ComponentKind, MachineDetail, Tier } from '../../utils/api';
import { DrawerFolder } from './DrawerFolder';

const FOLDERS: { kind: ComponentKind; label: string }[] = [
  { kind: 'mobo', get label() { return tr("placas base"); } },
  { kind: 'cpu', label: 'procesadores' },
  { kind: 'ram', label: 'memorias' },
  { kind: 'hdd', label: 'almacenamiento' },
  { kind: 'net', get label() { return tr("red"); } },
];

const TIERS: Tier[] = ['domestica', 'gamer', 'workstation', 'server'];

function haystack(part: Component): string {
  return [
    part.name,
    part.kind,
    part.tier,
    part.chipset ?? '',
    part.generation ? `ddr${part.generation}` : '',
    String(part.power),
  ]
    .join(' ')
    .toLowerCase();
}

type DrawerProps = {
  components: Component[];
  machines: MachineDetail[];
  busy: boolean;
  onInstall: (component: Component, machine: number) => void;
};

export function Drawer({ components, machines, busy, onInstall }: DrawerProps) {
  useIdioma();
  const [text, setText] = useState('');
  const [tier, setTier] = useState<Tier | 'all'>('all');
  const [opened, setOpened] = useState<ComponentKind[]>(['mobo']);

  const shown = useMemo(() => {
    const terms = text.trim().toLowerCase().split(/\s+/).filter(Boolean);

    return components.filter((part) => {
      if (tier !== 'all' && part.tier !== tier) {
        return false;
      }

      if (!terms.length) {
        return true;
      }

      const words = haystack(part);

      return terms.every((term) => words.includes(term));
    });
  }, [components, text, tier, idiomaActual()]);

  const searching = text.trim().length > 0 || tier !== 'all';

  const toggle = (kind: ComponentKind) =>
    setOpened((current) =>
      current.includes(kind) ? current.filter((one) => one !== kind) : [...current, kind],
    );

  const countTier = (value: Tier) => components.filter((part) => part.tier === value).length;

  return (
    <section className="panel">
      <header className="panel__head">
        <span className="panel__path">~/piezas</span>
        <span className="drawer__count">
          {searching ? tr("{0} de {1}", shown.length, components.length) : components.length}
        </span>
      </header>

      <div className="panel__body">
        {components.length === 0 ? (
          <p className="panel__empty">
            <span aria-hidden="true">$ </span>{tr("ls ~/piezas: el cajón está vacío")} </p>
        ) : (
          <>
            <div className="drawer__tools">
              <div className="drawer__search">
                <span className="drawer__glass" aria-hidden="true">
                  ⌕
                </span>
                <input
                  value={text}
                  placeholder={tr("buscar por nombre, chipset, DDR, capacidad…")}
                  onChange={(event) => setText(event.target.value)}
                />
                {text && (
                  <button type="button" aria-label={tr("Limpiar búsqueda")} onClick={() => setText('')}>
                    ×
                  </button>
                )}
              </div>

              <div className="drawer__tiers">
                <button
                  type="button"
                  className={tier === 'all' ? 'drawer__tier is-active' : 'drawer__tier'}
                  onClick={() => setTier('all')}
                >
                  {tr("todas")} </button>

                {TIERS.filter((value) => countTier(value) > 0).map((value) => (
                  <button
                    type="button"
                    key={value}
                    className={tier === value ? 'drawer__tier is-active' : 'drawer__tier'}
                    onClick={() => setTier(value)}
                  >
                    {value} <b>{countTier(value)}</b>
                  </button>
                ))}
              </div>
            </div>

            {shown.length === 0 ? (
              <p className="panel__empty">{tr("Ninguna pieza del cajón encaja con eso.")}</p>
            ) : (
              <ul className="folders">
                {FOLDERS.map((folder) => {
                  const parts = shown.filter((part) => part.kind === folder.kind);

                  if (searching && parts.length === 0) {
                    return null;
                  }

                  return (
                    <DrawerFolder
                      key={folder.kind}
                      kind={folder.kind}
                      label={folder.label}
                      parts={parts}
                      machines={machines}
                      open={searching || opened.includes(folder.kind)}
                      busy={busy}
                      onToggle={() => toggle(folder.kind)}
                      onInstall={onInstall}
                    />
                  );
                })}
              </ul>
            )}
          </>
        )}
      </div>
    </section>
  );
}
