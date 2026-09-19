import { tr, useIdioma } from '../../i18n/text';
import { useState, type FormEvent } from 'react';

import type { Component, MachineDetail } from '../../utils/api';
import { BoardLayout } from './BoardLayout';
import { MachineStats } from './MachineStats';

type MachineCardProps = {
  detail: MachineDetail;
  only: boolean;
  busy: boolean;
  onUninstall: (component: Component) => void;
  onRename: (machine: number, name: string) => void;
  onScrap: (machine: number, name: string) => void;
};

export function MachineCard({
  detail,
  only,
  busy,
  onUninstall,
  onRename,
  onScrap,
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
                disabled={busy || inside > 0}
                title={
                  inside > 0 ? tr("Saca antes las piezas que lleva dentro") : tr("Deshacerse del equipo")
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
          />
        ) : (
          <div className="board board--empty">
            <span className="board__hole" aria-hidden="true" />
            <p className="board__missing">{tr("sin placa")}</p>
            <p className="board__hint">
              {tr("Instala una desde el cajón: es la que decide qué entra después.")} </p>
          </div>
        )}

        <MachineStats slots={slots} hasBoard={Boolean(board)} />
      </div>
    </section>
  );
}
