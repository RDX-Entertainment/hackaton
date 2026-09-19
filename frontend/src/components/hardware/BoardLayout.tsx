import { tr, useIdioma } from '../../i18n/text';
import type { Component, ComponentKind, Slots } from '../../utils/api';
import { formatCount } from '../../utils/format';
import { PartIcon } from '../admin/PartIcon';

const UNIT: Record<ComponentKind, string> = {
  cpu: 'MHz',
  ram: 'MB',
  hdd: 'MB',
  net: 'Mbit',
  mobo: '',
};

const ZONES = [
  { kind: 'cpu', legend: 'socket', empty: 'CPU' },
  { kind: 'ram', legend: 'dimm', empty: 'DIMM' },
  { kind: 'hdd', legend: 'sata / m.2', empty: 'SATA' },
  { kind: 'net', legend: 'pcie', empty: 'LAN' },
] as const;

function specOf(part: Component): string {
  const base = `${formatCount(part.power)} ${UNIT[part.kind]}`;

  return part.kind === 'ram' ? `${base} · DDR${part.generation}` : base;
}

type BoardLayoutProps = {
  board: Component;
  components: Component[];
  slots: Slots;
  busy: boolean;
  onUninstall: (component: Component) => void;
  onAdd: (kind: ComponentKind) => void;
};

export function BoardLayout({ board, components, slots, busy, onUninstall, onAdd }: BoardLayoutProps) {
  useIdioma();
  const zones = ZONES.flatMap((zone) => Array.from({ length: zone.kind === 'ram' ? Math.max(1, slots.cpu_total) : 1 }, (_, group) => ({ ...zone, group })));
  const total = (kind: ComponentKind) =>
    kind === 'cpu'
      ? slots.cpu_total
      : kind === 'ram'
        ? slots.ram_total
        : kind === 'hdd'
          ? slots.hdd_total
          : slots.net_total;

  return (
    <div className="board">
      <header className="board__head">
        <PartIcon kind="mobo" className="board__icon" />

        <span className="board__text">
          <span className="board__name">{board.name}</span>
          <span className="board__spec">
            {board.chipset ?? tr("sin chipset")} · DDR{slots.generation}
          </span>
        </span>

        <button type="button" disabled={busy} onClick={() => onUninstall(board)}>
          {tr("sacar placa")} </button>
      </header>

      <div className="board__deck">
        <span className="board__trace board__trace--a" aria-hidden="true" />
        <span className="board__trace board__trace--b" aria-hidden="true" />

        {zones.map((zone) => {
          const filled = components.filter((part) => part.kind === zone.kind).sort((a, b) => a.id - b.id)
            .filter((_, index) => zone.kind !== 'ram' || index % Math.max(1, slots.cpu_used) === zone.group);
          const count = zone.kind === 'ram' ? Math.floor(slots.ram_total / Math.max(1, slots.cpu_total)) : total(zone.kind);

          return (
            <section className={`zone zone--${zone.kind}`} key={`${zone.kind}-${zone.group}`}>
              <span className="zone__legend">{zone.kind === 'ram' ? `DIMM · CPU ${zone.group + 1}${zone.group >= slots.cpu_used ? tr("· sin procesador") : ''}` : zone.legend}</span>

              <div className="zone__slots">
                {Array.from({ length: Math.max(count, filled.length) }, (_, index) => {
                  const part = filled[index];

                  if (!part) {
                    return (
                      <article className="mount mount--free" key={`${zone.kind}-${index}`}>
                        <span className="mount__label">
                          {zone.empty} {index + 1}
                        </span>
                        <span className="mount__free">{tr("libre")}</span>
                        <button disabled={busy} onClick={() => onAdd(zone.kind)} aria-label={tr("Agregar {0} compatible", zone.empty)}>+</button>
                      </article>
                    );
                  }

                  return (
                    <article className="mount" key={part.id} title={`${part.name} · ${specOf(part)}`}>
                      <PartIcon kind={part.kind} className="mount__icon" />

                      <span className="mount__name">{part.name}</span>
                      <span className="mount__spec">{specOf(part)}</span>

                      <button
                        type="button"
                        className="mount__pull"
                        aria-label={tr("Sacar {0}", part.name)}
                        disabled={busy}
                        onClick={() => onUninstall(part)}
                      >
                        ×
                      </button>
                    </article>
                  );
                })}

                {count === 0 && <p className="zone__none">{tr("esta placa no tiene")}</p>}
              </div>
            </section>
          );
        })}
      </div>
    </div>
  );
}
