import { formatCount, formatMoney } from '../../utils/format';
import { KIND_UNIT, TIER_LABEL, type Build } from '../../utils/kitBuild';
import { PartIcon } from './PartIcon';

type KitPreviewProps = {
  build: Build;
  price: number;
};

const SLOT_LABEL: Record<string, string> = {
  cpu: 'proceso',
  ram: 'memoria',
  hdd: 'disco',
  net: 'red',
};

function bar(used: number, total: number): string {
  if (total <= 0) {
    return '········';
  }

  const filled = Math.min(8, Math.round((used / total) * 8));

  return '▓'.repeat(filled) + '░'.repeat(Math.max(0, 8 - filled));
}

export function KitPreview({ build, price }: KitPreviewProps) {
  const { board, slots, problems } = build;

  const saving = build.price > 0 ? Math.round((1 - price / build.price) * 100) : 0;
  const broken = problems.length > 0;

  return (
    <aside className={broken ? 'kitprev is-broken' : 'kitprev'}>
      <div className="kitprev__stage">
        <span className="kitprev__glow" aria-hidden="true" />

        {board ? (
          <>
            <PartIcon kind="mobo" className="kitprev__icon" />
            <p className="kitprev__name">{board.name}</p>
            <p className="kitprev__spec">
              {board.chipset ?? 'sin chipset'} · DDR{board.generation}
            </p>
            <span className={`tier tier--${board.tier}`}>{TIER_LABEL[board.tier]}</span>
          </>
        ) : (
          <>
            <span className="kitprev__hole" aria-hidden="true" />
            <p className="kitprev__idle">sin placa</p>
            <p className="kitprev__spec">elige una y el resto encajará sobre ella</p>
          </>
        )}
      </div>

      <dl className="kitprev__slots">
        {slots.map((slot) => {
          const overSlots = slot.used > slot.total;
          const overPower = slot.power > slot.max;

          return (
            <div
              key={slot.kind}
              className={overSlots || overPower ? 'kitprev__slot is-over' : 'kitprev__slot'}
            >
              <dt>
                <PartIcon kind={slot.kind} className="kitprev__sloticon" />
                {SLOT_LABEL[slot.kind]}
              </dt>

              <dd>
                <span className="kitprev__bar" aria-hidden="true">
                  {bar(slot.used, slot.total)}
                </span>
                <span className="kitprev__count">
                  {slot.used}/{slot.total || '—'}
                </span>
                <span className="kitprev__power">
                  {formatCount(slot.power)}
                  {slot.max ? ` / ${formatCount(slot.max)}` : ''} {KIND_UNIT[slot.kind]}
                </span>
              </dd>
            </div>
          );
        })}
      </dl>

      <div className="kitprev__money">
        <span className="kitprev__pieces">{build.pieces} piezas</span>
        <span className="kitprev__loose">{formatMoney(build.price)} sueltas</span>
        <span className="kitprev__price">{formatMoney(price)}</span>
        {saving > 0 && <span className="kitprev__saving">−{saving}%</span>}
        {saving < 0 && <span className="kitprev__saving is-up">+{Math.abs(saving)}%</span>}
      </div>

      {broken && (
        <ul className="kitprev__problems">
          {problems.map((problem) => (
            <li key={problem}>{problem}</li>
          ))}
        </ul>
      )}
    </aside>
  );
}
