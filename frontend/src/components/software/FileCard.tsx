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
  /** Los otros equipos a los que se puede llevar. Vacio si solo hay uno. */
  destinos: { id: number; name: string }[];
  onMover: (destino: number) => void;
};

/**
 * Un fichero y lo que se puede hacer con él.
 *
 * Los verbos que ahora mismo no valen salen igualmente, apagados y con el motivo debajo.
 * Esconderlos dejaría la misma duda que antes —«¿y por qué no puedo instalarlo?»— con un
 * botón menos donde buscar la respuesta.
 */
export function FileCard({ fichero, tipo, ctx, onTarea, onInvestigar, destinos, onMover }: FileCardProps) {
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

        {/*
          Mover es un select y no un boton porque la pregunta no es «¿mover?» sino «¿a
          donde?»: con un boton habria que abrir algo despues para responderla. Cargado en
          memoria no se mueve -- primero se descarga --, y el motivo va en el title en vez de
          esconder el control, igual que hacen los demas verbos de esta tarjeta.
        */}
        {destinos.length > 0 && (
          <select
            className="swfile__move"
            value=""
            disabled={fichero.running}
            title={
              fichero.running
                ? 'Sacalo de memoria antes de moverlo.'
                : 'Llevarlo a otro equipo. Hace falta cable entre los dos, o salida a Internet en ambos.'
            }
            onChange={(event) => {
              const destino = Number(event.target.value);

              if (destino) {
                onMover(destino);
              }
            }}
          >
            <option value="">Mover a…</option>
            {destinos.map((destino) => (
              <option key={destino.id} value={destino.id}>
                {destino.name}
              </option>
            ))}
          </select>
        )}
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
