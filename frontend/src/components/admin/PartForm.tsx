import { tr, useIdioma } from '../../i18n/text';
import { useEffect, useState, type FormEvent } from 'react';

import type { Chipset, ComponentKind, Part, PartInput, Tier } from '../../utils/api';
import { AdminCard } from './AdminCard';
import { ChipsetPicker } from './ChipsetPicker';
import { PartIcon } from './PartIcon';
import { PartPreview } from './PartPreview';

const KINDS: { value: ComponentKind; label: string }[] = [
  { value: 'mobo', get label() { return tr("placa base"); } },
  { value: 'cpu', get label() { return tr("procesador"); } },
  { value: 'ram', get label() { return tr("memoria"); } },
  { value: 'hdd', get label() { return tr("disco"); } },
  { value: 'net', get label() { return tr("red"); } },
];

const TIERS: Tier[] = ['domestica', 'gaming', 'server', 'industrial'];

const DDR: number[] = [2, 3, 4, 5];

const POWER_LABEL: Record<ComponentKind, string> = {
  mobo: '',
  get cpu() { return tr("proceso (MHz)"); },
  get ram() { return tr("capacidad (MB)"); },
  get hdd() { return tr("capacidad (MB)"); },
  get net() { return tr("ancho de banda (Mbit)"); },
};

const CEILINGS = [
  { key: 'max_cpu', slots: 'cpu_slots', get label() { return tr("proceso máx. (MHz)"); }, get slotLabel() { return tr("zócalos cpu"); } },
  { key: 'max_ram', slots: 'ram_slots', get label() { return tr("memoria máx. (MB)"); }, get slotLabel() { return tr("zócalos ram"); } },
  { key: 'max_hdd', slots: 'hdd_slots', get label() { return tr("espacio máx. (MB)"); }, get slotLabel() { return tr("bahías disco"); } },
  { key: 'max_net', slots: 'net_slots', get label() { return tr("red máx. (Mbit)"); }, get slotLabel() { return tr("puertos red"); } },
] as const;

const EMPTY: PartInput = {
  kind: 'ram',
  name: '',
  power: 0,
  price: 0,
  generation: 3,
  tier: 'domestica',
  chipset_id: null,
  cpu_slots: 0,
  ram_slots: 0,
  hdd_slots: 0,
  net_slots: 0,
  max_cpu: 0,
  max_ram: 0,
  max_hdd: 0,
  max_net: 0,
  listed: true,
};

type PartFormProps = {
  editing: Part | null;
  chipsets: Chipset[];
  busy: boolean;
  onSubmit: (input: PartInput) => void;
  onCancel: () => void;
  onChipsetCreated: () => Promise<void>;
};

export function PartForm({
  editing,
  chipsets,
  busy,
  onSubmit,
  onCancel,
  onChipsetCreated,
}: PartFormProps) {
  useIdioma();
  const [draft, setDraft] = useState<PartInput>(EMPTY);

  useEffect(() => {
    if (!editing) {
      setDraft(EMPTY);
      return;
    }

    setDraft({
      kind: editing.kind,
      name: editing.name,
      power: editing.power,
      price: editing.price,
      generation: editing.generation,
      tier: editing.tier,
      chipset_id: editing.chipset_id,
      cpu_slots: editing.cpu_slots,
      ram_slots: editing.ram_slots,
      hdd_slots: editing.hdd_slots,
      net_slots: editing.net_slots,
      max_cpu: editing.max_cpu,
      max_ram: editing.max_ram,
      max_hdd: editing.max_hdd,
      max_net: editing.max_net,
      listed: editing.listed,
    });
  }, [editing]);

  const set = <K extends keyof PartInput>(key: K, value: PartInput[K]) =>
    setDraft((current) => {
      const next = { ...current, [key]: value };

      if (key === 'kind' && next.kind !== 'mobo' && next.kind !== 'cpu') {
        next.chipset_id = null;
      }

      return next;
    });

  const number = (value: string) => Number(value) || 0;

  const submit = (event: FormEvent) => {
    event.preventDefault();
    onSubmit(draft);
  };

  const isBoard = draft.kind === 'mobo';
  const usesChipset = isBoard || draft.kind === 'cpu';

  return (
    <AdminCard
      prefix="Hard"
      brand="Craft"
      note={editing ? tr("editando · {0}", editing.name) : tr("alta de pieza")}
    >
      <div className="partcard__kinds">
        {KINDS.map((option) => (
          <button
            type="button"
            key={option.value}
            className={option.value === draft.kind ? 'kindpick is-active' : 'kindpick'}
            onClick={() => set('kind', option.value)}
          >
            <PartIcon kind={option.value} className="kindpick__icon" />
            {option.label}
          </button>
        ))}
      </div>

      <div className="partcard__split">
        <form className="partform" onSubmit={submit}>
          <fieldset className="partform__group">
            <legend>{tr("Identidad")}</legend>

            <div className="partform__grid partform__grid--identity">
              <label className="partform__wide">
                {tr("nombre")} <input
                  value={draft.name}
                  placeholder="Cross 1G"
                  onChange={(event) => set('name', event.target.value)}
                  required
                />
              </label>

              <label>
                {tr("gama")} <select
                  value={draft.tier}
                  onChange={(event) => set('tier', event.target.value as Tier)}
                >
                  {TIERS.map((tier) => (
                    <option key={tier} value={tier}>
                      {tier}
                    </option>
                  ))}
                </select>
              </label>

              <label>
                {tr("precio")} <input
                  inputMode="numeric"
                  value={draft.price}
                  onChange={(event) => set('price', number(event.target.value))}
                  required
                />
              </label>
            </div>
          </fieldset>

          <fieldset className="partform__group">
            <legend>{usesChipset ? tr("Plataforma") : tr("Especificación")}</legend>

            <div className="partform__grid">
              {!isBoard && (
                <label>
                  {POWER_LABEL[draft.kind]}
                  <input
                    inputMode="numeric"
                    value={draft.power}
                    onChange={(event) => set('power', number(event.target.value))}
                    required
                  />
                </label>
              )}

              {(isBoard || draft.kind === 'ram') && (
                <label>
                  {isBoard ? tr("ddr que acepta") : tr("generación ddr")}
                  <select
                    value={draft.generation}
                    onChange={(event) => set('generation', number(event.target.value))}
                  >
                    {DDR.map((generation) => (
                      <option key={generation} value={generation}>
                        DDR{generation}
                      </option>
                    ))}
                  </select>
                </label>
              )}

              {usesChipset && (
                <ChipsetPicker
                  chipsets={chipsets}
                  value={draft.chipset_id}
                  onChange={(id) => set('chipset_id', id)}
                  onCreated={onChipsetCreated}
                  label={isBoard ? tr("chipset que monta") : tr("chipset que exige")}
                  emptyLabel={isBoard ? tr("sin chipset") : tr("compatible con cualquiera")}
                />
              )}

              <label className="partform__check">
                <input
                  type="checkbox"
                  checked={draft.listed}
                  onChange={(event) => set('listed', event.target.checked)}
                />
                {tr("a la venta")} </label>
            </div>
          </fieldset>

          {isBoard && (
            <fieldset className="partform__group">
              <legend>{tr("Compatibilidad")}</legend>

              <div className="partform__grid partform__grid--limits">
                {CEILINGS.map((row) => (
                  <div className="partform__limit" key={row.key}>
                    <label>
                      {row.slotLabel}
                      <input
                        inputMode="numeric"
                        value={draft[row.slots]}
                        onChange={(event) => set(row.slots, number(event.target.value))}
                      />
                    </label>

                    <label>
                      {row.label}
                      <input
                        inputMode="numeric"
                        value={draft[row.key]}
                        onChange={(event) => set(row.key, number(event.target.value))}
                      />
                    </label>
                  </div>
                ))}
              </div>

              <p className="partform__hint">
                {tr("Los zócalos limitan cuántas piezas caben. El máximo limita cuánto pueden sumar entre todas. Son dos cosas distintas y las dos se comprueban al instalar.")} </p>
            </fieldset>
          )}

          <div className="partform__actions">
            {editing && (
              <button type="button" onClick={onCancel} disabled={busy}>
                {tr("cancelar")} </button>
            )}
            <button className="is-primary" disabled={busy}>
              {editing ? tr("guardar cambios") : tr("crear pieza")}
            </button>
          </div>
        </form>

        <PartPreview draft={draft} chipsets={chipsets} />
      </div>
    </AdminCard>
  );
}
