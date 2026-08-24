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
};

export function BoardLayout({ board, components, slots, busy, onUninstall }: BoardLayoutProps) {
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
            {board.chipset ?? 'sin chipset'} · DDR{slots.generation}
          </span>
        </span>

        <button type="button" disabled={busy} onClick={() => onUninstall(board)}>
          sacar placa
        </button>
      </header>

      <div className="board__deck">
        <span className="board__trace board__trace--a" aria-hidden="true" />
        <span className="board__trace board__trace--b" aria-hidden="true" />

        {ZONES.map((zone) => {
          const filled = components.filter((part) => part.kind === zone.kind);
          const count = total(zone.kind);

          return (
            <section className={`zone zone--${zone.kind}`} key={zone.kind}>
              <span className="zone__legend">{zone.legend}</span>

              <div className="zone__slots">
                {Array.from({ length: count }, (_, index) => {
                  const part = filled[index];

                  if (!part) {
                    return (
                      <article className="mount mount--free" key={`${zone.kind}-${index}`}>
                        <span className="mount__label">
                          {zone.empty} {index + 1}
                        </span>
                        <span className="mount__free">libre</span>
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
                        aria-label={`Sacar ${part.name}`}
                        disabled={busy}
                        onClick={() => onUninstall(part)}
                      >
                        ×
                      </button>
                    </article>
                  );
                })}

                {count === 0 && <p className="zone__none">esta placa no tiene</p>}
              </div>
            </section>
          );
        })}
      </div>
    </div>
  );
}
