import type { OwnedFile } from '../../utils/api';
import {
  noSePuedeDescargar,
  noSePuedeInstalar,
  noSePuedeOcultar,
  noSePuedeRevelar,
  type Contexto,
} from './rules';

/** Los números de acción del legado. Ver `completeProcess` en `bin/worker.rs`. */
export const ACCION = {
  ocultar: 4,
  revelar: 5,
  instalar: 13,
  descargar: 14,
  aparcar: 18,
} as const;

type Verbo = {
  label: string;
  accion: number;
  /** Null si se puede; si no, lo que hay que arreglar antes. */
  motivo: string | null;
  /** Lo que va a pasar si se pulsa, cuando no es evidente. */
  nota?: string;
};

type FileCardProps = {
  fichero: OwnedFile;
  /** El nombre del tipo, que es lo que se lee; el número es cosa del legado. */
  tipo: string;
  ctx: Contexto;
  onTarea: (accion: number, label: string) => void;
  onInvestigar: () => void;
};

/**
 * Un fichero y lo que se puede hacer con él.
 *
 * Los verbos que ahora mismo no valen salen igualmente, apagados y con el motivo debajo.
 * Esconderlos dejaría la misma duda que antes —«¿y por qué no puedo instalarlo?»— con un
 * botón menos donde buscar la respuesta.
 */
export function FileCard({ fichero, tipo, ctx, onTarea, onInvestigar }: FileCardProps) {
  const verbos: Verbo[] = [
    fichero.running
      ? { label: 'Sacar de memoria', accion: ACCION.descargar, motivo: noSePuedeDescargar(fichero) }
      : {
          label: 'Cargar en memoria',
          accion: ACCION.instalar,
          motivo: noSePuedeInstalar(fichero, ctx),
        },
    fichero.hidden
      ? { label: 'Revelar', accion: ACCION.revelar, motivo: noSePuedeRevelar(fichero, ctx) }
      : {
          label: 'Ocultar',
          accion: ACCION.ocultar,
          motivo: noSePuedeOcultar(fichero, ctx),
          nota: ctx.hidder ? `Se esconderá con tu Hidder v${ctx.hidder}.` : undefined,
        },
    {
      label: 'Aparcar fuera',
      accion: ACCION.aparcar,
      motivo: null,
      nota: 'Al disco externo, donde sobrevive a formatear la máquina.',
    },
  ];

  const pegas = verbos.filter((verbo) => verbo.motivo);
  const avisos = verbos.filter((verbo) => !verbo.motivo && verbo.nota);

  return (
    <article className="swfile">
      <p className="swfile__head">
        <b>{fichero.name}</b>
        <span className="swfile__ver">v{fichero.version}</span>
        <span className="swfile__type">{tipo}</span>
      </p>

      <p className="swfile__meta">
        {fichero.size} de disco · {fichero.ram} de RAM
        {fichero.hidden && ` · escondido con un v${fichero.hidden_with}`}
      </p>

      <p className="swfile__actions">
        {verbos.map((verbo) => (
          <button
            key={verbo.label}
            disabled={Boolean(verbo.motivo)}
            title={verbo.motivo ?? verbo.nota ?? ''}
            onClick={() => onTarea(verbo.accion, verbo.label)}
          >
            {verbo.label}
          </button>
        ))}

        <button title={`Sube a la v${fichero.version + 1}.`} onClick={onInvestigar}>
          Investigar
        </button>
      </p>

      {(pegas.length > 0 || avisos.length > 0) && (
        <ul className="swfile__why">
          {pegas.map((verbo) => (
            <li className="is-blocked" key={verbo.label}>
              <b>{verbo.label}:</b> {verbo.motivo}
            </li>
          ))}
          {avisos.map((verbo) => (
            <li key={verbo.label}>
              <b>{verbo.label}:</b> {verbo.nota}
            </li>
          ))}
        </ul>
      )}
    </article>
  );
}
