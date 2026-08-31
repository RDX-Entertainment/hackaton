import { useState, type FormEvent } from 'react';

import { softwareApi, type Machine, type OwnedFile, type SoftwareType } from '../../utils/api';
import { useStatusMessage } from '../../utils/useStatusMessage';
import { StatusMessage } from '../StatusMessage';

const LARGO_NOMBRE = 25;

type WriteFileFormProps = {
  tipos: SoftwareType[];
  /** Los equipos donde se puede escribir. Un fichero nace en un disco, no «en el jugador». */
  maquinas: Machine[];
  /** Todo lo guardado, para saber cuanto queda libre en el equipo que se elija. */
  ficheros: OwnedFile[];
  onEscrito: () => void;
};

/**
 * Escribir un fichero a mano.
 *
 * Es la vía del legado: se elige un tipo, un tamaño y una RAM, y aparece el programa. No es
 * lo que hace un jugador para armarse —eso es Arsenal, donde un programa sale de una
 * capacidad y unos módulos—, pero sigue siendo la única forma de crear las herramientas del
 * legado que el sistema modular todavía no cubre: Hidder, Seeker, analizadores.
 *
 * Por eso está al final y con la advertencia puesta, en vez de presidir la pantalla como
 * hasta ahora.
 */
export function WriteFileForm({ tipos, maquinas, ficheros, onEscrito }: WriteFileFormProps) {
  const status = useStatusMessage();

  const [nombre, setNombre] = useState('');
  const [tipo, setTipo] = useState('1');
  const [tamano, setTamano] = useState('30');
  const [ram, setRam] = useState('50');
  const [equipo, setEquipo] = useState('');

  // Sin eleccion previa, el primero. `maquinas` llega despues del primer render, y un select
  // controlado con valor vacio enviaria un `server_id` que no existe.
  const elegido = maquinas.find((m) => String(m.id) === equipo) ?? maquinas[0];

  // El espacio ya no es «lo que te queda»: es lo que queda **en ese disco**. Un fichero que
  // no cabe aqui puede caber en el equipo de al lado, y eso ahora es una decision del jugador.
  const ocupado = elegido
    ? ficheros
        .filter((fichero) => fichero.server_id === elegido.id)
        .reduce((total, fichero) => total + fichero.size, 0)
    : 0;
  const discoLibre = elegido ? Math.max(0, elegido.hdd - ocupado) : 0;

  const pedido = Number(tamano);
  const noCabe = Number.isFinite(pedido) && pedido > discoLibre;

  const escribir = async (event: FormEvent) => {
    event.preventDefault();

    try {
      if (!elegido) {
        status.report('No hay ningun equipo donde escribirlo.');
        return;
      }

      await softwareApi.createSoftware(nombre, Number(tipo), pedido, Number(ram), elegido.id);
      setNombre('');
      status.report('Fichero escrito. Cárgalo en memoria para poder usarlo.');
      onEscrito();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <form className="compose" onSubmit={escribir}>
      <label>
        Nombre
        <input
          value={nombre}
          maxLength={LARGO_NOMBRE}
          onChange={(event) => setNombre(event.target.value)}
          required
        />
      </label>

      <label>
        Tipo
        <select value={tipo} onChange={(event) => setTipo(event.target.value)}>
          {tipos.map((uno) => (
            <option key={uno.id} value={uno.id}>
              {uno.name}
            </option>
          ))}
        </select>
      </label>

      <label>
        Equipo
        <select value={String(elegido?.id ?? '')} onChange={(event) => setEquipo(event.target.value)}>
          {maquinas.map((maquina) => (
            <option key={maquina.id} value={maquina.id}>
              {maquina.name} · {maquina.hdd} MB
            </option>
          ))}
        </select>
      </label>

      <label>
        Tamaño en disco
        <input
          value={tamano}
          inputMode="numeric"
          onChange={(event) => setTamano(event.target.value)}
          required
        />
        <small className="muted">Quedan {discoLibre} libres en {elegido?.name ?? 'ese equipo'}.</small>
      </label>

      <label>
        RAM que pedirá al cargarse
        <input
          value={ram}
          inputMode="numeric"
          onChange={(event) => setRam(event.target.value)}
          required
        />
      </label>

      <button disabled={noCabe}>Escribirlo</button>

      {noCabe ? (
        <span className="error">No cabe en {elegido?.name ?? 'ese equipo'}: quedan {discoLibre}.</span>
      ) : (
        <StatusMessage text={status.message} />
      )}
    </form>
  );
}
