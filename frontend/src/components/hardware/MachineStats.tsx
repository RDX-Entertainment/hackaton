import type { Slots } from '../../utils/api';
import { formatCount } from '../../utils/format';
import { PartIcon } from '../admin/PartIcon';

const ROWS = [
  { kind: 'cpu', label: 'proceso', unit: 'MHz' },
  { kind: 'ram', label: 'memoria', unit: 'MB' },
  { kind: 'hdd', label: 'disco', unit: 'MB' },
  { kind: 'net', label: 'red', unit: 'Mbit' },
] as const;

type MachineStatsProps = {
  slots: Slots;
  hasBoard: boolean;
};

export function MachineStats({ slots, hasBoard }: MachineStatsProps) {
  return (
    <aside className="mstats">
      <h4 className="mstats__title">capacidad</h4>

      {ROWS.map((row) => {
        const power = slots[`${row.kind}_power`];
        const max = slots[`${row.kind}_max`];
        const used = slots[`${row.kind}_used`];
        const total = slots[`${row.kind}_total`];

        const ratio = max > 0 ? Math.min(100, (power / max) * 100) : 0;
        const full = max > 0 && power >= max;

        return (
          <div className={full ? 'mstat is-full' : 'mstat'} key={row.kind}>
            <div className="mstat__head">
              <PartIcon kind={row.kind} className="mstat__icon" />
              <span className="mstat__label">{row.label}</span>
              <span className="mstat__slots">
                {used}/{total || '—'}
              </span>
            </div>

            <p className="mstat__value">
              {formatCount(power)}
              <small>
                {' '}
                / {formatCount(max)} {row.unit}
              </small>
            </p>

            <div className="mstat__bar">
              <span className="mstat__fill" style={{ width: `${ratio}%` }} />
            </div>
          </div>
        );
      })}

      {!hasBoard && (
        <p className="mstats__note">
          Sin placa no hay capacidad: todo lo que suma un equipo son las piezas que lleva
          puestas.
        </p>
      )}
    </aside>
  );
}
