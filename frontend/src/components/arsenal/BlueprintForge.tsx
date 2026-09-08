import { useEffect, useMemo, useState, type FormEvent } from 'react';

import { arsenalApi, type Capability, type ResearchModule, type ProgramStats } from '../../utils/api';
import { StatSheet } from './StatSheet';
import { useStatusMessage } from '../../utils/useStatusMessage';
import { StatusMessage } from '../StatusMessage';

const LARGO_NOMBRE = 25;
const MAX_MODULOS = 3;

type BlueprintForgeProps = {
  capacidades: Capability[];
  modulos: ResearchModule[];
  onCreado: () => void;
};

/**
 * Diseñar un programa: una capacidad y hasta tres módulos.
 *
 * La previsión viene del servidor: elegir módulos y compilar usa la misma regla.
 */
export function BlueprintForge({ capacidades, modulos, onCreado }: BlueprintForgeProps) {
  const status = useStatusMessage();

  const [nombre, setNombre] = useState('');
  const [elegida, setElegida] = useState('');
  const [piezas, setPiezas] = useState<number[]>([]);
  const [preview, setPreview] = useState<ProgramStats>();
  const [previewError, setPreviewError] = useState('');
  const [saving, setSaving] = useState(false);

  const capacidad = elegida || capacidades[0]?.code || '';
  useEffect(() => {
    let current = true;
    setPreview(undefined);
    setPreviewError('');
    if (capacidad) {
      arsenalApi.previewBlueprint(capacidad, piezas)
        .then((stats) => { if (current) setPreview(stats); })
        .catch(() => { if (current) setPreviewError('No se pudo calcular la vista previa.'); });
    }
    return () => { current = false; };
  }, [capacidad, piezas]);

  // Los dominios, en el orden en que llegan, para agrupar el selector sin inventar un orden.
  const dominios = useMemo(
    () => [...new Set(capacidades.map((una) => una.domain))],
    [capacidades],
  );

  // El servidor solo acepta módulos de la capacidad elegida —o universales—, así que la lista
  // se recorta aquí en vez de dejar que el intento termine en un 400.
  const encajan = useMemo(
    () => modulos.filter((pieza) => !pieza.capability || pieza.capability === capacidad),
    [modulos, capacidad],
  );

  const cambiarCapacidad = (code: string) => {
    setElegida(code);
    // Las piezas de la capacidad anterior no valen para la nueva.
    setPiezas([]);
  };

  const alternar = (id: number) => {
    status.clear();

    setPiezas((actuales) => {
      if (actuales.includes(id)) {
        return actuales.filter((otro) => otro !== id);
      }

      if (actuales.length >= MAX_MODULOS) {
        status.report(`Un diseño admite ${MAX_MODULOS} módulos como mucho.`);
        return actuales;
      }

      return [...actuales, id];
    });
  };

  const crear = async (event: FormEvent) => {
    event.preventDefault();
    if (saving) return;
    setSaving(true);

    try {
      const creado = await arsenalApi.createBlueprint(nombre.trim(), capacidad, piezas);
      setNombre('');
      setPiezas([]);
      status.report(`Diseño «${creado.name}» guardado. Compílalo para tener el fichero.`);
      onCreado();
    } catch (error) {
      status.reportError(error);
    } finally {
      setSaving(false);
    }
  };

  if (!capacidades.length) {
    return null;
  }

  return (
    <form className="compose" onSubmit={crear}>
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
        Capacidad
        <select value={capacidad} onChange={(event) => cambiarCapacidad(event.target.value)}>
          {dominios.map((dominio) => (
            <optgroup key={dominio} label={dominio}>
              {capacidades
                .filter((una) => una.domain === dominio)
                .map((una) => (
                  <option key={una.code} value={una.code}>
                    {una.name} · {una.role === 'defender' ? 'defensa' : 'ataque'}
                  </option>
                ))}
            </optgroup>
          ))}
        </select>
      </label>

      <fieldset className="aforge__modules">
        <legend>
          Módulos <small>{piezas.length}/{MAX_MODULOS}</small>
        </legend>

        {encajan.length ? (
          encajan.map((pieza) => (
            <label className="aforge__module" key={pieza.id}>
              <input
                type="checkbox"
                checked={piezas.includes(pieza.id)}
                onChange={() => alternar(pieza.id)}
              />
              <span>{pieza.name}</span>
              <small>{resumir(pieza)}</small>
            </label>
          ))
        ) : (
          <p className="muted">Ninguna pieza investigada encaja en esta capacidad.</p>
        )}
      </fieldset>

      <div className="aforge__preview" aria-live="polite">
        <h4>Resultado del diseño</h4>
        {preview ? <StatSheet stats={preview} /> : <p>{previewError || 'Calculando atributos…'}</p>}
      </div>
      <button disabled={saving || !nombre.trim() || !preview}>{saving ? 'Guardando…' : 'Guardar el diseño'}</button>
      <StatusMessage text={status.message} />
    </form>
  );
}

/** Lo que aporta y lo que cuesta una pieza, en una línea y sin los ceros. */
function resumir(pieza: ResearchModule): string {
  const { deltas } = pieza;

  const partes = [
    ['efic', deltas.effectiveness],
    ['resistencia', deltas.resistance],
    ['blindaje', deltas.hardening],
    ['manipulación', deltas.manipulation],
    ['integridad', deltas.integrity],
    ['anti-forense', deltas.anti_forensics],
    ['forense', deltas.forensics],
    ['persistencia', deltas.persistence],
    ['sigilo', deltas.stealth],
    ['detec', deltas.detection],
    ['limp', deltas.removal],
    ['penet', deltas.penetration],
    ['estab', deltas.stability],
    ['cpu', deltas.cpu],
    ['ram', deltas.ram],
    ['red', deltas.net],
    ['disco', deltas.weight],
  ] as const;

  return partes
    .filter(([, valor]) => valor !== 0)
    .map(([etiqueta, valor]) => `${valor > 0 ? '+' : ''}${valor} ${etiqueta}`)
    .join(' · ');
}
