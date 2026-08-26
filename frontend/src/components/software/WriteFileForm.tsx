import { useState, type FormEvent } from 'react';

import { softwareApi, type SoftwareType } from '../../utils/api';
import { useStatusMessage } from '../../utils/useStatusMessage';
import { StatusMessage } from '../StatusMessage';

const LARGO_NOMBRE = 25;

type WriteFileFormProps = {
  tipos: SoftwareType[];
  discoLibre: number;
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
export function WriteFileForm({ tipos, discoLibre, onEscrito }: WriteFileFormProps) {
  const status = useStatusMessage();

  const [nombre, setNombre] = useState('');
  const [tipo, setTipo] = useState('1');
  const [tamano, setTamano] = useState('30');
  const [ram, setRam] = useState('50');

  const pedido = Number(tamano);
  const noCabe = Number.isFinite(pedido) && pedido > discoLibre;

  const escribir = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await softwareApi.createSoftware(nombre, Number(tipo), pedido, Number(ram));
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
        Tamaño en disco
        <input
          value={tamano}
          inputMode="numeric"
          onChange={(event) => setTamano(event.target.value)}
          required
        />
        <small className="muted">Te quedan {discoLibre} libres.</small>
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
        <span className="error">No cabe en el disco: te quedan {discoLibre}.</span>
      ) : (
        <StatusMessage text={status.message} />
      )}
    </form>
  );
}
