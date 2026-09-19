import { tr, useIdioma } from '../../i18n/text';
import { useEffect, useRef } from 'react';
import type { Component, ComponentKind, MachineDetail, Part } from '../../utils/api';
import { formatMoney } from '../../utils/format';

type Piece = Pick<Part, 'kind' | 'tier' | 'generation' | 'chipset_id' | 'power'>;

export function fitsMachine(piece: Piece, detail: MachineDetail): boolean {
  if (detail.machine.node_kind === 'switch') return false;
  const board = detail.board;
  if (piece.kind === 'mobo') return !board;
  if (!board || piece.tier !== board.tier) return false;
  if (piece.kind === 'ram' && piece.generation !== board.generation) return false;
  if (piece.kind === 'cpu' && piece.chipset_id !== null && piece.chipset_id !== board.chipset_id) return false;
  const installed = detail.components.filter((one) => one.kind === piece.kind);
  return installed.length < board[`${piece.kind}_slots`]
    && installed.reduce((sum, one) => sum + one.power, piece.power) <= board[`max_${piece.kind}`];
}

type Props = {
  detail: MachineDetail; owned: Component[]; parts: Part[]; busy: boolean; account: string;
  kind: ComponentKind | null; onClose: () => void;
  onInstall: (part: Component, machine: number) => void;
  onBuy: (part: Part, machine: number) => void;
};

export function CompatibleParts({ detail, owned, parts, busy, account, kind, onClose, onInstall, onBuy }: Props) {
  useIdioma();
  const panel = useRef<HTMLElement>(null);
  useEffect(() => {
    panel.current?.scrollIntoView({ block: 'center', behavior: 'instant' });
    panel.current?.focus({ preventScroll: true });
  }, [detail.machine.id, kind]);
  const matches = (piece: Piece) => (!kind || piece.kind === kind) && fitsMachine(piece, detail);
  const drawer = owned.filter((one) => one.installed_in === null && matches(one));
  const shop = parts.filter((one) => one.listed && matches(one));
  return <section ref={panel} tabIndex={-1} className="panel compatible-parts" aria-label={tr("Componentes compatibles")}>
    <header className="panel__head"><b>{tr("Compatibles con")} {detail.machine.name}{kind ? ` · ${kind.toUpperCase()}` : ''}</b>
      <button onClick={onClose}>{tr("Cerrar")}</button></header>
    <div className="panel__body">
      <p>{tr("Solo piezas que encajan por gama, chipset, DDR, ranuras y capacidad. Se instalan directamente.")}</p>
      <h4>{tr("Tu cajón")}</h4>
      {!drawer.length && <p className="muted">{tr("No tienes piezas compatibles libres.")}</p>}
      {drawer.map((one) => <div className="compatible-parts__row" key={one.id}><span>{one.name}</span>
        <button disabled={busy} onClick={() => onInstall(one, detail.machine.id)}>{tr("Instalar")}</button></div>)}
      <h4>{tr("Tienda")}</h4>
      {!shop.length && <p className="muted">{tr("No hay piezas compatibles a la venta.")}</p>}
      {!account && <p>{tr("Selecciona una cuenta para comprar.")}</p>}
      {shop.map((one) => <div className="compatible-parts__row" key={one.id}><span>{one.name} · {formatMoney(one.price)}</span>
        <button disabled={busy || !account} onClick={() => onBuy(one, detail.machine.id)}>{tr("Comprar e instalar")}</button></div>)}
    </div>
  </section>;
}
