import { useMemo } from 'react';

import { Icono } from '../iconos/Icono';
import type { Machine, OwnedFile } from '../../utils/api';

/**
 * El sitio en el que estás mirando.
 *
 * `null` es la raíz, que **no es un disco**: es la lista de volúmenes. Hoy un
 * volumen es una máquina tuya; el día que exista el espacio en la nube será otra
 * entrada de esa misma lista y nada más de aquí tiene que enterarse.
 */
export type Sitio = { volumen: number; carpeta: number | null } | null;

type ExploradorProps = {
  sitio: Sitio;
  maquinas: Machine[];
  ficheros: OwnedFile[];
  onIr: (sitio: Sitio) => void;
};

/** Las migas: dónde estás y cómo se vuelve. */
export function Migas({ sitio, maquinas, ficheros, onIr }: ExploradorProps) {
  const camino = useMemo(() => {
    if (!sitio) {
      return [];
    }

    const maquina = maquinas.find((una) => una.id === sitio.volumen);
    const pasos: { id: number | null; nombre: string }[] = [
      { id: null, nombre: maquina?.name ?? `Equipo #${sitio.volumen}` },
    ];

    // Se sube por los padres y se le da la vuelta: la carpeta de la que cuelga
    // todo es la que va primero, y el sitio en el que estás, el último.
    const subida: OwnedFile[] = [];
    let actual = sitio.carpeta;

    while (actual !== null) {
      const carpeta = ficheros.find((uno) => uno.id === actual);
      if (!carpeta) {
        break;
      }
      subida.unshift(carpeta);
      actual = carpeta.parent_id;
    }

    for (const carpeta of subida) {
      pasos.push({ id: carpeta.id, nombre: carpeta.name });
    }

    return pasos;
  }, [sitio, maquinas, ficheros]);

  return (
    <nav className="explorador__migas" aria-label="Dónde estás">
      <button type="button" onClick={() => onIr(null)}>
        <Icono nombre="red" tamano={13} /> raíz
      </button>

      {camino.map((paso, indice) => (
        <span key={paso.id ?? 'volumen'}>
          <i aria-hidden="true">/</i>
          {indice === camino.length - 1 ? (
            <b>{paso.nombre}</b>
          ) : (
            <button
              type="button"
              onClick={() => sitio && onIr({ volumen: sitio.volumen, carpeta: paso.id })}
            >
              {paso.nombre}
            </button>
          )}
        </span>
      ))}
    </nav>
  );
}

/**
 * La raíz: un volumen por equipo, y nada más.
 *
 * Nada de software suelto aquí. Un fichero vive **dentro** de una máquina, igual
 * que en un ordenador de verdad no hay ficheros flotando por encima de los
 * discos.
 */
export function Volumenes({
  maquinas,
  ficheros,
  onEntrar,
}: {
  maquinas: Machine[];
  ficheros: OwnedFile[];
  onEntrar: (id: number) => void;
}) {
  return (
    <div className="explorador__volumenes">
      {maquinas.map((maquina) => {
        const dentro = ficheros.filter((uno) => uno.server_id === maquina.id);
        const ocupado = dentro.reduce((suma, uno) => suma + uno.size, 0);
        const lleno = maquina.hdd ? Math.min(100, (ocupado / maquina.hdd) * 100) : 0;

        return (
          <button
            type="button"
            className="explorador__volumen"
            key={maquina.id}
            onClick={() => onEntrar(maquina.id)}
          >
            <span className="explorador__icono">
              <Icono nombre="equipo" tamano={26} />
            </span>

            <span className="explorador__nombre">{maquina.name}</span>
            <span className="micro">
              {dentro.length} {dentro.length === 1 ? 'elemento' : 'elementos'}
            </span>

            <span className="barra explorador__disco">
              <i style={{ width: `${lleno}%` }} />
            </span>
            <span className="micro">
              {ocupado} / {maquina.hdd} MB
            </span>
          </button>
        );
      })}

      {/*
        Aquí es donde entra el espacio en la nube el día que exista: otra entrada
        de esta lista, con su propio origen de ficheros. La raíz no sabe qué es un
        equipo, solo que hay volúmenes.
      */}
      <div className="explorador__volumen es-futuro">
        <span className="explorador__icono">
          <Icono nombre="red" tamano={26} />
        </span>
        <span className="explorador__nombre">Nube</span>
        <span className="micro">todavía no existe</span>
      </div>
    </div>
  );
}
