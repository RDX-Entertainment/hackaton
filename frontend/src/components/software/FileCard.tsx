import { tr, useIdioma } from '../../i18n/text';
import { useTutorial } from '../../app/tutorial';
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

/**
 * El nombre que entiende el tutorial para cada verbo del legado.
 *
 * Lo que no está aquí no lo gobierna el tutorial —aparcar en el disco externo, por ejemplo—,
 * igual que en el servidor: negar por omisión convertiría cada verbo nuevo en un botón
 * apagado sin que nadie lo hubiera decidido.
 */
const CODIGO_DEL_VERBO: Record<number, string> = {
  [ACCION.ocultar]: 'discos.ocultar',
  [ACCION.revelar]: 'discos.ocultar',
  [ACCION.instalar]: 'discos.cargar',
  [ACCION.descargar]: 'discos.descargar',
};

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
  useIdioma();
  const verbos: Verbo[] = [
    fichero.running
      ? { label: tr("Sacar de memoria"), accion: ACCION.descargar, motivo: noSePuedeDescargar(fichero) }
      : {
          label: tr("Cargar en memoria"),
          accion: ACCION.instalar,
          motivo: noSePuedeInstalar(fichero, ctx),
        },
    fichero.hidden
      ? { label: tr("Revelar"), accion: ACCION.revelar, motivo: noSePuedeRevelar(fichero, ctx) }
      : {
          label: tr("Ocultar"),
          accion: ACCION.ocultar,
          motivo: noSePuedeOcultar(fichero, ctx),
          nota: ctx.hidder ? tr("Se esconderá con tu Hidder v{0}.", ctx.hidder) : undefined,
        },
    {
      label: tr("Aparcar fuera"),
      accion: ACCION.aparcar,
      motivo: null,
      nota: tr("Al disco externo, donde sobrevive a formatear la máquina."),
    },
  ];

  // El tutorial es una pega más, y entra por la misma puerta que las otras: se apaga el
  // verbo y su motivo sale en la lista de abajo, donde el jugador ya sabe mirar.
  const tutorial = useTutorial();

  const conPermiso = verbos.map((verbo) => {
    const codigo = CODIGO_DEL_VERBO[verbo.accion];

    return {
      ...verbo,
      codigo,
      motivo:
        !codigo || tutorial.permiteAccion(codigo) ? verbo.motivo : (verbo.motivo ?? tutorial.motivo),
    };
  });

  const pegas = conPermiso.filter((verbo) => verbo.motivo);
  const avisos = conPermiso.filter((verbo) => !verbo.motivo && verbo.nota);

  return (
    <article className={`swfile${fichero.running ? ' is-running' : ''}`}>
      {/*
        Una fila y no un bloque apilado. Antes cada fichero ocupaba cabecera, línea de datos,
        botonera y una lista de motivos: con ocho programas la pantalla era un scroll. Aquí
        la identidad va a la izquierda, las cifras en el medio alineadas, y los verbos a la
        derecha, que es como se lee un inventario.
      */}
      <div className="swfile__id">
        <b>{fichero.name}</b>
        <span className="swfile__ver">v{fichero.version}</span>
        <span className="swfile__type">{tipo}</span>
        {fichero.hidden && (
          <span className="swfile__flag" title={tr("Escondido con un v{0}", fichero.hidden_with)}>
            {tr("oculto")} </span>
        )}
      </div>

      <div className="swfile__nums">
        <span title={tr("Ocupa en disco")}>{fichero.size}▪</span>
        <span title={tr("Pide de RAM al cargarse")}>{fichero.ram}▫</span>
      </div>

      <div className="swfile__actions">
        {conPermiso.map((verbo) => (
          <button
            key={verbo.label}
            data-foco={verbo.codigo}
            className={verbo.codigo && tutorial.foco === verbo.codigo ? 'es-foco' : undefined}
            disabled={Boolean(verbo.motivo)}
            title={verbo.motivo ?? verbo.nota ?? ''}
            onClick={() => onTarea(verbo.accion, verbo.label)}
          >
            {verbo.label}
          </button>
        ))}

        <button title={tr("Sube a la v{0}.", fichero.version + 1)} onClick={onInvestigar}>
          {tr("Investigar")} </button>

        {/*
          Mover es un select y no un botón porque la pregunta no es «¿mover?» sino «¿a
          dónde?». Cargado en memoria no se mueve: primero se descarga.
        */}
        {destinos.length > 0 && (
          <select
            className="swfile__move"
            value=""
            disabled={fichero.running}
            title={
              fichero.running
                ? tr("Sácalo de memoria antes de moverlo.")
                : tr("Llevarlo a otro equipo. Hace falta cable entre los dos, o salida a Internet en ambos.")
            }
            onChange={(event) => {
              const destino = Number(event.target.value);

              if (destino) {
                onMover(destino);
              }
            }}
          >
            <option value="">{tr("Mover a…")}</option>
            {destinos.map((destino) => (
              <option key={destino.id} value={destino.id}>
                {destino.name}
              </option>
            ))}
          </select>
        )}
      </div>

      {/*
        Los verbos que no valen siguen saliendo apagados y con su motivo, que era el punto de
        antes —esconderlos deja la duda de por qué no se puede—, pero en una línea seguida en
        vez de una lista con viñetas: es el bloque que más alto hacía la tarjeta.
      */}
      {(pegas.length > 0 || avisos.length > 0) && (
        <p className="swfile__why">
          {pegas.map((verbo) => (
            <span className="is-blocked" key={verbo.label}>
              <b>{verbo.label}:</b> {verbo.motivo}
            </span>
          ))}
          {avisos.map((verbo) => (
            <span key={verbo.label}>
              <b>{verbo.label}:</b> {verbo.nota}
            </span>
          ))}
        </p>
      )}
    </article>
  );
}
