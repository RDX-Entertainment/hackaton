import { tr, useIdioma, idiomaActual } from '../../i18n/text';
import { useEffect, useMemo, useState, type FormEvent } from 'react';
import { useLocation } from 'react-router-dom';

import { useAccion } from '../../app/tutorial';
import { arsenalApi, type Capability, type ResearchModule, type ProgramStats } from '../../utils/api';
import { StatSheet } from './StatSheet';
import { useStatusMessage } from '../../utils/useStatusMessage';
import { StatusMessage } from '../StatusMessage';
import { Icono, type NombreDeIcono } from '../iconos/Icono';

const LARGO_NOMBRE = 25;

type BlueprintForgeProps = {
  capacidades: Capability[];
  modulos: ResearchModule[];
  onCreado: () => void;
  modulesUnavailable?: boolean;
};

/**
 * Diseñar un programa: una capacidad y hasta tres módulos.
 *
 * La previsión viene del servidor: elegir módulos y compilar usa la misma regla.
 */
export function BlueprintForge({ capacidades, onCreado }: BlueprintForgeProps) {
  useIdioma();
  const location = useLocation();
  const requested = new URLSearchParams(location.search).get('capability');
  // El tutorial decide si este es el paso de disenar; el servidor lo comprueba igual.
  const disenar = useAccion('lab.disenar');

  const status = useStatusMessage();

  const [nombre, setNombre] = useState('');
  const [elegida, setElegida] = useState('');
  const [preview, setPreview] = useState<ProgramStats>();
  const [previewError, setPreviewError] = useState('');
  const [saving, setSaving] = useState(false);

  const capacidad = elegida || capacidades.find(c => c.code === requested)?.code || capacidades[0]?.code || '';
  const definicion = capacidades.find((una) => una.code === capacidad);
  const icono: NombreDeIcono = definicion?.role === 'defender'
    ? 'cerrado'
    : definicion?.action_kind === 'intrude' ? 'llave'
    : definicion?.role === 'utility' ? 'red' : 'lanzar';
  useEffect(() => {
    let current = true;
    setPreview(undefined);
    setPreviewError('');
    if (capacidad) {
      arsenalApi.previewBlueprint(capacidad, [])
        .then((stats) => { if (current) setPreview(stats); })
        .catch(() => { if (current) setPreviewError(tr("No se pudo calcular la vista previa.")); });
    }
    return () => { current = false; };
  }, [capacidad]);

  // Los dominios, en el orden en que llegan, para agrupar el selector sin inventar un orden.
  const dominios = useMemo(
    () => [...new Set(capacidades.map((una) => una.domain))],
    [capacidades, idiomaActual()],
  );

  const crear = async (event: FormEvent) => {
    event.preventDefault();
    if (saving) return;
    setSaving(true);

    try {
      const creado = await arsenalApi.createBlueprint(nombre.trim(), capacidad, []);
      setNombre('');
      status.report(tr("Diseño «{0}» guardado. Compílalo para tener el fichero.", creado.name));
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
    <form className="compose aforge" onSubmit={crear}>
      <header className="aforge__heading">
        <div><span className="micro">{tr("Mesa de creación")}</span><h3>{tr("Diseña tu software")}</h3></div>
        <span className="micro">{tr("Capacidad + módulos → programa")}</span>
      </header>
      <div className="aforge__workbench">
      <div className="aforge__design">
        <div className="aforge__hologram">
          <div className="aforge__orb" aria-hidden="true"><Icono nombre={icono} tamano={88} /></div>
          <strong>{nombre.trim() || tr(definicion?.name ?? '')}</strong>
          <span>{tr(definicion?.domain ?? '')} · {definicion?.role === 'defender' ? tr("defensa") : definicion?.role === 'utility' ? tr("utilidad") : tr("ataque")}</span>
        </div>
      <label>
        {tr("Nombre")} <input
          data-guia="lab.nombre"
          value={nombre}
          maxLength={LARGO_NOMBRE}
          onChange={(event) => setNombre(event.target.value)}
          required
        />
      </label>

      <label>
        {tr("Capacidad")} <select data-guia="lab.capacidad" value={capacidad} onChange={(event) => setElegida(event.target.value)}>
          {dominios.map((dominio) => (
            <optgroup key={dominio} label={tr(dominio)}>
              {capacidades
                .filter((una) => una.domain === dominio)
                .map((una) => (
                  <option key={una.code} value={una.code}>
                    {tr(una.name)} · {una.role === 'defender' ? tr("defensa") : una.role === 'utility' ? tr("utilidad") : tr("ataque")}
                  </option>
                ))}
            </optgroup>
          ))}
        </select>
      </label>

      <button
        data-foco="lab.disenar"
        className={disenar.destacada ? 'es-foco' : undefined}
        disabled={saving || !nombre.trim() || !preview || !disenar.permitida}
        title={disenar.motivo ?? undefined}
      >
        {saving ? tr("Guardando…") : tr("Guardar el diseño")}
      </button>
      </div>
      <aside className="aforge__preview" aria-live="polite" aria-label={tr("Estadísticas del diseño")}>
        <h4>{tr("Resultado del diseño")}</h4>
        <p className="muted">{tr("Vista previa · perfil base · ranuras después de compilar")}</p>
        {preview ? <StatSheet stats={preview} capability={definicion} /> : <p>{previewError || tr("Calculando atributos…")}</p>}
        {capacidad === 'antivirus' && <p className="muted">{tr("Detección encuentra virus frente a su sigilo; limpieza los elimina frente a su persistencia. Un mismo antivirus realiza ambas tareas.")}</p>}
        {capacidad === 'port_scan' && <p className="muted">{tr("Descubre los servicios desde Internet → Servicios. Eficacia contra blindaje; sigilo contra detección. Compila y activa el escáner antes de usarlo. Requiere certificación 3.")}</p>}
      </aside>
      </div>

      <p className="muted">{tr("Compila el diseño y abre Lab → Mejorar para instalar módulos físicos, añadir capacidades y fusionar copias. Las ranuras se desbloquean al investigar el programa.")}</p>

      <StatusMessage text={status.message} />
    </form>
  );
}
