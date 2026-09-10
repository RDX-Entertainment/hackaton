import { useEffect, useState } from 'react';

import { dashboardApi, type UnknownRow } from '../utils/api';
import { formatCell } from '../utils/format';

/**
 * Los números del servidor.
 *
 * Era una tabla de una sola fila con las columnas de la instantánea puestas de
 * lado: para leer un número había que cruzar la vista con su cabecera. Son ocho
 * cifras sueltas, y una cifra suelta es una tarjeta.
 *
 * Las claves que el worker escribe se traducen aquí; la que no conozcamos sale
 * con su nombre crudo en vez de desaparecer, que es como se descubre que el
 * worker empezó a contar algo nuevo.
 */
const NOMBRES: Record<string, string> = {
  users: 'jugadores',
  total_users: 'jugadores',
  online: 'conectados',
  clans: 'clanes',
  total_clans: 'clanes',
  servers: 'máquinas',
  hardware: 'máquinas',
  software: 'programas',
  processes: 'tareas en marcha',
  npc: 'máquinas del mundo',
  money: 'dinero en juego',
  bitcoin: 'bitcoin en juego',
  updated_at: 'medido',
  created_at: 'medido',
};

export function StatsPage() {
  const [numeros, setNumeros] = useState<UnknownRow>();

  useEffect(() => {
    dashboardApi.getServerStats().then(setNumeros).catch(() => setNumeros(undefined));
  }, []);

  // La instantánea no existe hasta que el worker ha pasado una vez.
  if (!numeros) {
    return (
      <>
        <h2>Números</h2>
        <div className="vacio">
          <div className="vacio__titulo">Todavía no hay instantánea</div>
          <p className="vacio__texto">
            La escribe el worker la primera vez que pasa. Si lleva un rato en marcha y sigue
            vacía, es que el worker no está corriendo.
          </p>
        </div>
      </>
    );
  }

  return (
    <>
      <h2>Números</h2>
      <p className="muted">Lo que el worker contó la última vez que pasó.</p>

      <div className="rejilla-tarjetas">
        {Object.entries(numeros).map(([clave, valor]) => (
          <article className="tarjeta" key={clave}>
            <span className="tarjeta__rotulo">{NOMBRES[clave] ?? clave}</span>
            <b className="tarjeta__cifra">{formatCell(valor)}</b>
            <div className="tarjeta__regla" />
          </article>
        ))}
      </div>
    </>
  );
}
