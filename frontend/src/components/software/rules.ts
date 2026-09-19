import { tr } from '../../i18n/text';
import type { OwnedFile } from '../../utils/api';

/**
 * Por qué un verbo no se puede usar sobre un fichero.
 *
 * Estas reglas son **copia de las del worker**, y están aquí por una razón concreta: el
 * worker no rechaza, calla. Si un install no cabe en memoria o un hide no encuentra un
 * Hidder, `completeProcess` hace `return Ok(())` y la tarea termina «correcta» sin que haya
 * pasado nada. Desde la pantalla eso se ve como un botón que no hace nada.
 *
 * Así que se comprueban antes de encolar y se dice el motivo. Si algún día cambia una regla
 * en `bin/worker.rs`, cambia aquí: son dos sitios a propósito, porque el segundo existe para
 * que el primero no tenga que hablar.
 */

/** Los tipos del legado que hacen falta para decidir. Ver `legacy.rs`. */
const HIDDER = 5;
const SEEKER = 6;
const CARPETA = 31;
const DOOM = 29;

/** `is_virus`: lo que no se descarga de memoria una vez dentro. */
const VIRUS = new Set([8, 9, 10, 20, 29]);

/** `is_executable` niega también estos: torrent, fichero de texto y carpeta. */
const SIN_EJECUCION = new Set([17, 30, 31]);

/** Lo que se sabe de la máquina en el momento de decidir. */
export type Contexto = {
  /** RAM libre: la de las máquinas menos la que ya ocupa lo cargado. */
  ramLibre: number;
  /** Versión del mejor Hidder cargado en memoria, o null si no hay ninguno. */
  hidder: number | null;
  /** Ídem para el Seeker. Un oculto solo se revela con un Seeker igual o mejor. */
  seeker: number | null;
};

export function noSePuedeInstalar(fichero: OwnedFile, ctx: Contexto): string | null {
  if (fichero.running) return tr("Ya está cargado en memoria.");
  if (fichero.hidden) return tr("Está oculto: revélalo antes.");
  if (VIRUS.has(fichero.type) || SIN_EJECUCION.has(fichero.type)) {
    return tr("No es un programa que se pueda ejecutar.");
  }
  if (ctx.ramLibre < fichero.ram) {
    return tr("No cabe: te faltan {0} de RAM.", fichero.ram - ctx.ramLibre);
  }

  return null;
}

export function noSePuedeDescargar(fichero: OwnedFile): string | null {
  if (!fichero.running) return tr("No está en memoria.");
  if (VIRUS.has(fichero.type)) return tr("Un virus no se deja sacar de memoria.");

  return null;
}

export function noSePuedeOcultar(fichero: OwnedFile, ctx: Contexto): string | null {
  if (fichero.hidden) return tr("Ya está oculto.");
  if (fichero.type === CARPETA) return tr("Una carpeta no se oculta.");
  if (fichero.type === DOOM) return tr("Esto no se oculta.");
  if (fichero.running) return tr("Sácalo de memoria antes de esconderlo.");
  if (ctx.hidder === null) return tr("Necesitas un Hidder cargado en memoria.");

  return null;
}

export function noSePuedeRevelar(fichero: OwnedFile, ctx: Contexto): string | null {
  if (!fichero.hidden) return tr("No está oculto.");
  if (ctx.seeker === null) return tr("Necesitas un Seeker cargado en memoria.");
  if (ctx.seeker < fichero.hidden_with) {
    return tr("Tu Seeker v{0} no llega: esto se escondió con un v{1}.", ctx.seeker, fichero.hidden_with);
  }

  return null;
}

/** Lo que ocupa en memoria lo que está cargado ahora mismo. */
export function ramEnUso(ficheros: OwnedFile[]): number {
  return ficheros.reduce((total, uno) => total + (uno.running ? uno.ram : 0), 0);
}

/** El disco lo ocupa todo lo que hay, corra o no. */
export function discoEnUso(ficheros: OwnedFile[]): number {
  return ficheros.reduce((total, uno) => total + uno.size, 0);
}

/** La mejor versión cargada de una herramienta, que es como la mide el worker. */
function mejorCargado(ficheros: OwnedFile[], tipo: number): number | null {
  const versiones = ficheros
    .filter((uno) => uno.running && uno.type === tipo)
    .map((uno) => uno.version);

  return versiones.length ? Math.max(...versiones) : null;
}

export function contextoDe(ficheros: OwnedFile[], ramTotal: number): Contexto {
  return {
    ramLibre: ramTotal - ramEnUso(ficheros),
    hidder: mejorCargado(ficheros, HIDDER),
    seeker: mejorCargado(ficheros, SEEKER),
  };
}
