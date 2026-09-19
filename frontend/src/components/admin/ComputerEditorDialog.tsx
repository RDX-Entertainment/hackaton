import { tr, useIdioma, idiomaActual } from '../../i18n/text';
import { useEffect, useMemo, useState } from 'react';

import type { Kit, OrganizationComputerConfiguration, Part } from '../../utils/api';
import { build } from '../../utils/kitBuild';
import { KitBuilder, type BuildLine } from './KitBuilder';
import { KitPreview } from './KitPreview';

type ComputerEditorDialogProps = {
  open: boolean;
  title: string;
  kits: Kit[];
  parts: Part[];
  configuration: OrganizationComputerConfiguration | null;
  lanLinks: number;
  busy?: boolean;
  onClose: () => void;
  onSave: (input: { kit_id?: number; items?: BuildLine[] }) => void;
};

/**
 * The computer editor owns only drafting. Persisting and inventory semantics remain in
 * the organization screen/API, which keeps this component reusable for any NPC node.
 */
export function ComputerEditorDialog({ open, title, kits, parts, configuration, lanLinks, busy, onClose, onSave }: ComputerEditorDialogProps) {
  useIdioma();
  const [kitId, setKitId] = useState<number | null>(kits[0]?.id ?? null);
  const [mode, setMode] = useState<'kit' | 'custom'>('kit');
  const [lines, setLines] = useState<BuildLine[]>([]);
  const selectedKit = kits.find((kit) => kit.id === kitId) ?? null;
  const partById = useMemo(() => new Map(parts.map((part) => [part.id, part])), [parts, idiomaActual()]);
  const preview = useMemo(() => build(lines.map((line) => ({ part: partById.get(line.part), amount: line.amount })).filter((item): item is { part: Part; amount: number } => Boolean(item.part))), [lines, partById, idiomaActual()]);

  useEffect(() => {
    if (!open) return;
    const savedKit = kits.find((kit) => kit.id === configuration?.source_kit_id) ?? null;
    const first = savedKit ?? kits[0] ?? null;
    setKitId(first?.id ?? null);
    setMode(savedKit ? 'kit' : 'custom');
    setLines(configuration?.items ?? first?.items.map((item) => ({ part: item.part, amount: item.amount })) ?? []);
  }, [open, kits, configuration]);

  const chooseKit = (next: number) => {
    const kit = kits.find((item) => item.id === next);
    setKitId(next);
    setLines(kit?.items.map((item) => ({ part: item.part, amount: item.amount })) ?? []);
  };
  const useCustom = () => setMode('custom');
  const ready = mode === 'kit' ? Boolean(selectedKit) : lines.length > 0 && preview.problems.length === 0;

  if (!open) return null;
  return <div className="site-editor-backdrop" role="presentation" onMouseDown={onClose}>
    <section className="computer-editor" role="dialog" aria-modal="true" aria-label={tr("Editor de PC: {0}", title)} onMouseDown={(event) => event.stopPropagation()}>
      <header><div><small>{tr("COMPUTER · MODO CREATIVO")}</small><h3>{title}</h3></div><button type="button" onClick={onClose}>{tr("Cerrar")}</button></header>
      <div className="computer-editor__body">
        <div className="computer-editor__choices">
          <label>{tr("Kit base")}<select value={kitId ?? ''} onChange={(event) => chooseKit(Number(event.target.value))} disabled={!kits.length}>{kits.map((kit) => <option key={kit.id} value={kit.id}>{kit.name}</option>)}</select></label>
          {!kits.length && <p>{tr("No hay kits todavía. Monta el equipo desde cero.")}</p>}
          <div className="computer-editor__modes"><button type="button" className={mode === 'kit' ? 'is-active' : ''} disabled={!selectedKit} onClick={() => setMode('kit')}>{tr("Usar kit")}</button><button type="button" className={mode === 'custom' ? 'is-active' : ''} onClick={useCustom}>{tr("Personalizar / desde cero")}</button></div>
          <p>{mode === 'kit' ? tr("El kit se instala tal cual.") : tr("Las piezas se crean directamente para este nodo: inventario ilimitado.")}</p>
        </div>
        {mode === 'custom' && <KitBuilder parts={parts} lines={lines} onChange={setLines} />}
        {mode === 'kit' && <div className="computer-editor__kit"><b>{selectedKit?.name ?? tr("Sin kit")}</b><span>{selectedKit?.items.length ?? 0} {tr("tipos de pieza · puedes personalizarlo sin consumir existencias.")}</span></div>}
        <KitPreview build={preview} price={0} lanLinks={lanLinks} />
      </div>
      <footer><button type="button" onClick={onClose}>{tr("Cancelar")}</button><button className="is-primary" disabled={busy || !ready} onClick={() => onSave(mode === 'kit' ? { kit_id: selectedKit!.id } : { items: lines })}>{tr("Instalar PC")}</button></footer>
    </section>
  </div>;
}
