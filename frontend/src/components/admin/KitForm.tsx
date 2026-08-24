import { useEffect, useMemo, useState, type FormEvent } from 'react';

import type { Kit, KitClan, KitInput, KitScope, Part } from '../../utils/api';
import { build } from '../../utils/kitBuild';
import { AdminCard } from './AdminCard';
import { ClanPicker } from './ClanPicker';
import { KitBuilder, type BuildLine } from './KitBuilder';
import { KitPreview } from './KitPreview';

const SCOPES: { value: KitScope; label: string; hint: string }[] = [
  { value: 'shop', label: 'tienda', hint: 'cualquiera lo ve y lo compra' },
  { value: 'clan', label: 'corporación', hint: 'solo los miembros de las que elijas' },
  { value: 'marketplace', label: 'mercado', hint: 'reservado a los equipos de NPC' },
  { value: 'hidden', label: 'oculto', hint: 'no se ofrece; existe para aplicarlo a mano' },
];

const EMPTY = {
  name: '',
  description: '',
  price: '0',
  scope: 'shop' as KitScope,
  listed: true,
  lines: [] as BuildLine[],
  clans: [] as number[],
};

type Draft = typeof EMPTY;

type KitFormProps = {
  editing: Kit | null;
  parts: Part[];
  clans: KitClan[];
  busy: boolean;
  onSubmit: (input: KitInput) => void;
  onCancel: () => void;
};

export function KitForm({ editing, parts, clans, busy, onSubmit, onCancel }: KitFormProps) {
  const [draft, setDraft] = useState<Draft>(EMPTY);

  useEffect(() => {
    if (!editing) {
      setDraft(EMPTY);
      return;
    }

    setDraft({
      name: editing.name,
      description: editing.description,
      price: String(editing.price),
      scope: editing.scope,
      listed: editing.listed,
      lines: editing.items.map((item) => ({ part: item.part, amount: item.amount })),
      clans: editing.clans.map((clan) => clan.id),
    });
  }, [editing]);

  const set = <K extends keyof Draft>(key: K, value: Draft[K]) =>
    setDraft((current) => ({ ...current, [key]: value }));

  const byId = useMemo(() => new Map(parts.map((part) => [part.id, part])), [parts]);

  const preview = useMemo(
    () =>
      build(
        draft.lines
          .map((line) => ({ part: byId.get(line.part), amount: line.amount }))
          .filter((item): item is { part: Part; amount: number } => Boolean(item.part)),
      ),
    [draft.lines, byId],
  );

  const price = Number(draft.price) || 0;

  const ready =
    draft.name.trim().length > 0 &&
    draft.lines.length > 0 &&
    preview.problems.length === 0 &&
    (draft.scope !== 'clan' || draft.clans.length > 0);

  const send = (event: FormEvent) => {
    event.preventDefault();

    if (!ready) {
      return;
    }

    onSubmit({
      name: draft.name.trim(),
      description: draft.description.trim(),
      price,
      scope: draft.scope,
      listed: draft.listed,
      items: draft.lines,
      clans: draft.scope === 'clan' ? draft.clans : [],
    });
  };

  const chosen = SCOPES.find((scope) => scope.value === draft.scope);

  return (
    <AdminCard
      prefix="Hard"
      brand="Kit"
      note={editing ? `editando ${editing.name}` : 'equipos montados'}
    >
      <div className="partcard__split">
        <form className="partform" onSubmit={send}>
          <fieldset className="partform__group">
            <legend>Identidad</legend>

            <div className="partform__grid partform__grid--identity">
              <label>
                nombre
                <input
                  value={draft.name}
                  maxLength={40}
                  onChange={(event) => set('name', event.target.value)}
                  required
                />
              </label>

              <label>
                precio del kit
                <input
                  inputMode="numeric"
                  value={draft.price}
                  onChange={(event) => set('price', event.target.value)}
                />
              </label>
            </div>

            <label className="kitform__desc">
              descripción
              <textarea
                value={draft.description}
                rows={2}
                maxLength={240}
                placeholder="qué es este equipo y para quién"
                onChange={(event) => set('description', event.target.value)}
              />
            </label>
          </fieldset>

          <fieldset className="partform__group">
            <legend>Dónde se ofrece</legend>

            <div className="chips">
              {SCOPES.map((scope) => (
                <button
                  type="button"
                  key={scope.value}
                  className={draft.scope === scope.value ? 'chip is-active' : 'chip'}
                  onClick={() => set('scope', scope.value)}
                >
                  {scope.label}
                </button>
              ))}
            </div>

            <p className="partform__hint">{chosen?.hint}</p>

            {draft.scope === 'clan' && (
              <ClanPicker
                clans={clans}
                chosen={draft.clans}
                onChange={(next) => set('clans', next)}
              />
            )}

            <label className="partform__check">
              <input
                type="checkbox"
                checked={draft.listed}
                onChange={(event) => set('listed', event.target.checked)}
              />
              a la venta
            </label>
          </fieldset>

          <fieldset className="partform__group">
            <legend>Contenido</legend>

            <KitBuilder
              parts={parts}
              lines={draft.lines}
              onChange={(lines) => set('lines', lines)}
            />
          </fieldset>

          <div className="partform__actions">
            {editing && (
              <button type="button" onClick={onCancel} disabled={busy}>
                cancelar
              </button>
            )}
            <button className="is-primary" disabled={busy || !ready}>
              {editing ? 'guardar cambios' : 'crear kit'}
            </button>
          </div>
        </form>

        <KitPreview build={preview} price={price} />
      </div>
    </AdminCard>
  );
}
