import { tr, useIdioma } from '../../i18n/text';
import { formatCount, formatMoney } from '../../utils/format';
import { KIND_UNIT, TIER_LABEL, type Build } from '../../utils/kitBuild';
import { PartIcon } from './PartIcon';

type KitPreviewProps = {
  build: Build;
  price: number;
  /** Physical node-to-node cables already attached to this machine. */
  lanLinks?: number;
};

const SLOT_LABEL: Record<string, string> = {
  cpu: 'proceso',
  ram: 'memoria',
  hdd: 'disco',
  get net() { return tr("interfaz Internet"); },
};

function bar(used: number, total: number): string {
  if (total <= 0) {
    return '········';
  }

  const filled = Math.min(8, Math.round((used / total) * 8));

  return '▓'.repeat(filled) + '░'.repeat(Math.max(0, 8 - filled));
}

export function KitPreview({ build, price, lanLinks = 0 }: KitPreviewProps) {
  useIdioma();
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
              {board.chipset ?? tr("sin chipset")} · DDR{board.generation}
            </p>
            <span className={`tier tier--${board.tier}`}>{TIER_LABEL[board.tier]}</span>
          </>
        ) : (
          <>
            <span className="kitprev__hole" aria-hidden="true" />
            <p className="kitprev__idle">{tr("sin placa")}</p>
            <p className="kitprev__spec">{tr("elige una y el resto encajará sobre ella")}</p>
          </>
        )}
      </div>

      <dl className="kitprev__slots">
        {slots.map((slot) => {
          // A connected LAN cable activates the board's Internet interface visually.
          const used = slot.kind === 'net' ? Math.min(slot.total, slot.used + lanLinks) : slot.used;
          const power = slot.kind === 'net' && used > 0 ? Math.max(slot.power, slot.max) : slot.power;
          const overSlots = used > slot.total;
          const overPower = power > slot.max;

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
                  {bar(used, slot.total)}
                </span>
                <span className="kitprev__count">
                  {used}/{slot.total || '—'}
                </span>
                <span className="kitprev__power">
                  {formatCount(power)}
                  {slot.max ? ` / ${formatCount(slot.max)}` : ''} {KIND_UNIT[slot.kind]}
                </span>
              </dd>
            </div>
          );
        })}
      </dl>

      {board && <p className="kitprev__lan">{tr("LAN de placa:")} <b>{lanLinks}/{board.net_slots}</b> {tr("ocupados ·")} <b>{Math.max(0, board.net_slots - lanLinks)}</b> {tr("libres")}</p>}

      {build.networkInterfaces.length > 0 && <section className="kitprev__internet"><small>{tr("Interfaz a Internet")}</small>{build.networkInterfaces.map((network) => <span key={network.name}>{network.amount > 1 ? `${network.amount}× ` : ''}{network.name} · {formatCount(network.power)} Mbit</span>)}</section>}

      <div className="kitprev__money">
        <span className="kitprev__pieces">{build.pieces} {tr("piezas")}</span>
        <span className="kitprev__loose">{formatMoney(build.price)} {tr("sueltas")}</span>
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
