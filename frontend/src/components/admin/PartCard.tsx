import { tr, useIdioma } from '../../i18n/text';
import type { Part } from '../../utils/api';
import { formatCount, formatMoney } from '../../utils/format';
import { PartIcon } from './PartIcon';

const UNIT: Record<string, string> = {
  cpu: 'MHz',
  ram: 'MB',
  hdd: 'MB',
  net: 'Mbit',
};

type PartCardProps = {
  part: Part;
  busy: boolean;
  onEdit: (part: Part) => void;
  onRemove: (part: Part) => void;
};

export function PartCard({ part, busy, onEdit, onRemove }: PartCardProps) {
  useIdioma();
  const isBoard = part.kind === 'mobo';

  const specs = isBoard
    ? [
        { key: 'chipset', value: part.chipset ?? '—' },
        { key: 'acepta', value: `DDR${part.generation}` },
        {
          key: tr("zócalos"),
          value: `${part.cpu_slots}c ${part.ram_slots}r ${part.hdd_slots}d ${part.net_slots}n`,
        },
        { key: tr("ram máx"), value: `${formatCount(part.max_ram)} MB` },
        { key: tr("hdd máx"), value: `${formatCount(part.max_hdd)} MB` },
      ]
    : [
        { key: 'capacidad', value: `${formatCount(part.power)} ${UNIT[part.kind] ?? ''}` },
        ...(part.kind === 'ram' ? [{ key: 'ddr', value: `DDR${part.generation}` }] : []),
        ...(part.kind === 'cpu'
          ? [{ key: 'chipset', value: part.chipset ?? 'cualquiera' }]
          : []),
      ];

  return (
    <article className={part.listed ? 'partitem' : 'partitem is-retired'}>
      <header className="partitem__head">
        <PartIcon kind={part.kind} className="partitem__icon" />

        <div className="partitem__title">
          <h4>{part.name}</h4>
          <span className="partitem__id">#{part.id}</span>
        </div>

        <span className={`tier tier--${part.tier}`}>{part.tier}</span>
      </header>

      <dl className="partitem__specs">
        {specs.map((spec) => (
          <div key={spec.key}>
            <dt>{spec.key}</dt>
            <dd>{spec.value}</dd>
          </div>
        ))}
      </dl>

      <footer className="partitem__foot">
        <span className="partitem__price">{formatMoney(part.price)}</span>

        {!part.listed && <span className="partitem__retired">{tr("retirada")}</span>}

        <span className="partitem__actions">
          <button type="button" disabled={busy} onClick={() => onEdit(part)}>
            {tr("editar")} </button>
          <button type="button" disabled={busy} onClick={() => onRemove(part)}>
            {tr("borrar")} </button>
        </span>
      </footer>
    </article>
  );
}
