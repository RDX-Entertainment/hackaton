import type { Chipset, ComponentKind, PartInput } from '../../utils/api';
import { formatCount, formatMoney } from '../../utils/format';
import { PartIcon } from './PartIcon';

const KIND_LABEL: Record<ComponentKind, string> = {
  mobo: 'placa base',
  cpu: 'procesador',
  ram: 'memoria',
  hdd: 'almacenamiento',
  net: 'red',
};

const UNIT: Record<ComponentKind, string> = {
  mobo: '',
  cpu: 'MHz',
  ram: 'MB',
  hdd: 'MB',
  net: 'Mbit',
};

type Row = {
  key: string;
  value: string;
};

type PartPreviewProps = {
  draft: PartInput;
  chipsets: Chipset[];
};

export function PartPreview({ draft, chipsets }: PartPreviewProps) {
  const isBoard = draft.kind === 'mobo';
  const chipset = chipsets.find((option) => option.id === draft.chipset_id);

  const rows: Row[] = [];

  if (isBoard) {
    rows.push(
      { key: 'chipset', value: chipset?.name ?? 'sin chipset' },
      { key: 'acepta', value: `DDR${draft.generation}` },
      {
        key: 'zócalos',
        value: `${draft.cpu_slots}c · ${draft.ram_slots}r · ${draft.hdd_slots}d · ${draft.net_slots}n`,
      },
      { key: 'cpu máx', value: `${formatCount(draft.max_cpu)} MHz` },
      { key: 'ram máx', value: `${formatCount(draft.max_ram)} MB` },
      { key: 'hdd máx', value: `${formatCount(draft.max_hdd)} MB` },
      { key: 'red máx', value: `${formatCount(draft.max_net)} Mbit` },
    );
  } else {
    rows.push({ key: 'capacidad', value: `${formatCount(draft.power)} ${UNIT[draft.kind]}` });

    if (draft.kind === 'ram') {
      rows.push({ key: 'generación', value: `DDR${draft.generation}` });
    }

    if (draft.kind === 'cpu') {
      rows.push({ key: 'chipset', value: chipset?.name ?? 'cualquiera' });
    }
  }

  rows.push({ key: 'precio', value: formatMoney(draft.price) });

  return (
    <aside className="preview">
      <p className="preview__kind">{KIND_LABEL[draft.kind]}</p>

      <div className="preview__stage">
        <span className="preview__glow" aria-hidden="true" />
        <span className="preview__ring" aria-hidden="true" />
        <PartIcon kind={draft.kind} className="preview__icon" />
      </div>

      <p className="preview__name">{draft.name.trim() || 'sin nombre'}</p>

      <span className={`tier tier--${draft.tier}`}>{draft.tier}</span>

      <dl className="preview__rows">
        {rows.map((row) => (
          <div className="preview__row" key={row.key}>
            <dt>{row.key}</dt>
            <dd>{row.value}</dd>
          </div>
        ))}
      </dl>

      <p className={draft.listed ? 'preview__state is-listed' : 'preview__state'}>
        {draft.listed ? 'a la venta' : 'retirada del catálogo'}
      </p>
    </aside>
  );
}
