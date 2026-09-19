import { tr, useIdioma } from '../../i18n/text';
import { useState, type FormEvent } from 'react';

import type { Component, ComponentKind, MachineDetail } from '../../utils/api';
import { BoardLayout } from './BoardLayout';
import { MachineStats } from './MachineStats';

type MachineCardProps = {
  detail: MachineDetail;
  only: boolean;
  busy: boolean;
  onUninstall: (component: Component) => void;
  onRename: (machine: number, name: string) => void;
  onScrap: (machine: number, name: string) => void;
  onAdd: (kind: ComponentKind | null) => void;
};

export function MachineCard({
  detail,
  only,
  busy,
  onUninstall,
  onRename,
  onScrap,
  onAdd,
}: MachineCardProps) {
  useIdioma();
  const { machine, board, components, slots } = detail;

  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(machine.name);

  const inside = components.length + (board ? 1 : 0);

  const rename = (event: FormEvent) => {
    event.preventDefault();

    const wanted = name.trim();

    if (!wanted || wanted === machine.name) {
      setEditing(false);
      return;
    }

    onRename(machine.id, wanted);
    setEditing(false);
  };

  return (
    <section className="panel machine">
      <header className="panel__head">
        {editing ? (
          <form className="machine__rename" onSubmit={rename}>
            <input
              value={name}
              maxLength={15}
              autoFocus
              onChange={(event) => setName(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Escape') {
                  setName(machine.name);
                  setEditing(false);
                }
              }}
            />
            <button className="is-primary" disabled={busy}>
              {tr("guardar")} </button>
            <button
              type="button"
              onClick={() => {
                setName(machine.name);
                setEditing(false);
              }}
            >
              {tr("cancelar")} </button>
          </form>
        ) : (
          <span className="panel__path">
            /dev/{machine.name.replace(/\s+/g, '').toLowerCase()}
          </span>
        )}

        {!editing && (
          <span className="machine__badges">
            <span>{machine.operational ? tr("Activa") : tr("Desactivada")}</span>
            {machine.is_gateway && <span>{tr("★ Principal")}</span>}
            {board && (
              <>
                <span className={`tier tier--${slots.tier}`}>{slots.tier}</span>
                {board.chipset && <span className="machine__chipset">{board.chipset}</span>}
                <span className="machine__ddr">DDR{slots.generation}</span>
              </>
            )}

            <button type="button" className="machine__act" onClick={() => setEditing(true)}>
              {tr("renombrar")} </button>

            {!only && (
              <button
                type="button"
                className="machine__act machine__act--drop"
                disabled={busy || inside > 0 || machine.is_gateway}
                title={
                  machine.is_gateway ? tr("Nombra principal a otro equipo primero") : inside > 0 ? tr("Saca antes las piezas que lleva dentro") : tr("Deshacerse del equipo")
                }
                onClick={() => onScrap(machine.id, machine.name)}
              >
                {tr("dar de baja")} </button>
            )}
          </span>
        )}
      </header>

      <div className="machine__split">
        {board ? (
          <BoardLayout
            board={board}
            components={components}
            slots={slots}
            busy={busy}
            onUninstall={onUninstall}
            onAdd={onAdd}
          />
        ) : (
          <div className="board board--empty">
            <span className="board__hole" aria-hidden="true" />
            <p className="board__missing">{tr("sin placa")}</p>
            <p className="board__hint">
              {tr("Instala una desde el cajón: es la que decide qué entra después.")} </p>
            <button disabled={busy} onClick={() => onAdd('mobo')}>{tr("Agregar placa compatible")}</button>
          </div>
        )}

        <MachineStats slots={slots} hasBoard={Boolean(board)} />
      </div>
      <div className="machine__readiness" role="status">
        <b>{machine.operational ? tr("● Activa") : tr("○ Desactivada")}</b>
        {!machine.operational && <ul>{(machine.disabled_reasons ?? []).map((reason) => <li key={reason}>{({
          NO_BOARD: tr("Falta placa base."), NO_CPU: tr("Falta procesador."), NO_RAM: tr("Falta RAM."), NO_DISK: tr("Falta almacenamiento."),
          RAM_UNBALANCED: tr("Instala la misma cantidad de módulos en cada CPU: con {0} procesadores, la RAM debe ir en múltiplos de {1}.", slots.cpu_used, slots.cpu_used),
          RAM_NEEDS_CPU: tr("Hay más RAM que ranuras disponibles en los sockets con CPU: instala otro procesador o retira módulos."),
        } as Record<string, string>)[reason] ?? reason}</li>)}</ul>}
        <button disabled={busy} onClick={() => onAdd(null)}>{tr("+ Agregar compatibles")}</button>
      </div>
    </section>
  );
}
