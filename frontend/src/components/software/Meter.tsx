import { tr, useIdioma } from '../../i18n/text';
type MeterProps = {
  label: string;
  usado: number;
  total: number;
  unidad: string;
  /** Una línea que explica qué se está midiendo y por qué importa. */
  nota: string;
};

/** A partir de aquí el color avisa antes de que algo deje de caber. */
const APRETADO = 85;

/**
 * Cuánto queda de disco o de memoria.
 *
 * Está arriba del todo porque es la respuesta a la pregunta que más se hace esta pantalla:
 * por qué un fichero no se carga. El worker no lo dice —calla y da la tarea por hecha—, así
 * que el número tiene que estar a la vista antes de intentarlo.
 */
export function Meter({ label, usado, total, unidad, nota }: MeterProps) {
  useIdioma();
  const porcentaje = total > 0 ? Math.min(100, Math.round((usado / total) * 100)) : 0;
  const libre = Math.max(0, total - usado);

  return (
    <div className={`meter${porcentaje >= APRETADO ? ' is-tight' : ''}`}>
      <p className="meter__head">
        <span className="meter__label">{label}</span>
        <span className="meter__value">
          {libre} {unidad} <small>{tr("libres de")} {total}</small>
        </span>
      </p>

      <span className="meter__bar">
        <span className="meter__fill" style={{ width: `${porcentaje}%` }} />
      </span>

      <p className="meter__note">{nota}</p>
    </div>
  );
}
