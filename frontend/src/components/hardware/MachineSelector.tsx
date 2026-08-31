import type { MachineDetail } from '../../utils/api';

type MachineSelectorProps = {
  machines: MachineDetail[];
  /** El equipo que se está viendo. */
  selected: number;
  onSelect: (machine: number) => void;
};

/**
 * El selector de equipo: `‹ Servidor ›`.
 *
 * Antes la página apilaba una ficha por equipo, una debajo de otra. Con dos ya había que
 * bajar para ver el segundo, y la ficha es alta —placa, ranuras, piezas puestas—; con
 * cinco, comparar dos equipos era imposible sin recordar lo que ponía el de arriba.
 *
 * Aquí solo hay un equipo en pantalla y las flechas lo cambian, así que la ficha está
 * siempre en el mismo sitio: lo que cambia entre uno y otro salta a la vista porque los
 * números no se mueven de su posición.
 *
 * Da la vuelta por los dos extremos a propósito. Con pocos equipos, tener que retroceder
 * cuatro veces para volver al primero es más molesto que útil, y no hay ningún orden que
 * el jugador pueda perder.
 */
export function MachineSelector({ machines, selected, onSelect }: MachineSelectorProps) {
  const index = machines.findIndex((detail) => detail.machine.id === selected);
  const actual = index < 0 ? 0 : index;
  const machine = machines[actual]?.machine;

  if (!machine) {
    return null;
  }

  const mover = (paso: number) => {
    const siguiente = (actual + paso + machines.length) % machines.length;
    onSelect(machines[siguiente].machine.id);
  };

  const solo = machines.length < 2;

  return (
    <div className="mchooser">
      <button
        type="button"
        className="mchooser__arrow"
        disabled={solo}
        onClick={() => mover(-1)}
        aria-label="Equipo anterior"
      >
        ‹
      </button>

      <span className="mchooser__name">
        <b>{machine.name}</b>
        {!solo && (
          <small>
            {actual + 1} / {machines.length}
          </small>
        )}
      </span>

      <button
        type="button"
        className="mchooser__arrow"
        disabled={solo}
        onClick={() => mover(1)}
        aria-label="Equipo siguiente"
      >
        ›
      </button>
    </div>
  );
}
