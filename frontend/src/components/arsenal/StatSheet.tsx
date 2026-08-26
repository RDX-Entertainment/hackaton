import type { ProgramStats } from '../../utils/api';

/**
 * Los seis atributos que deciden un combate, en el orden en que actúan: lo que empuja, lo que
 * atraviesa, lo que se esconde, y luego los tres con los que responde un defensor.
 */
const ATRIBUTOS: { key: keyof ProgramStats; label: string }[] = [
  { key: 'effectiveness', label: 'Eficacia' },
  { key: 'penetration', label: 'Penetración' },
  { key: 'stealth', label: 'Sigilo' },
  { key: 'detection', label: 'Detección' },
  { key: 'removal', label: 'Limpieza' },
  { key: 'stability', label: 'Estabilidad' },
];

const TOPE = 100;

type StatSheetProps = {
  stats: ProgramStats;
  /**
   * La máquina sobre la que tiene que correr, cuando hay con qué comparar. Un coste que el
   * hardware no cubre es la diferencia entre un programa que tienes y uno que puedes usar.
   */
  capacidad?: { cpu: number; ram: number; net: number };
};

/**
 * Los diez números de un programa. Los atributos llevan barra porque están todos en la misma
 * escala de 0 a 100 y solo significan algo unos al lado de otros; los costes van en cifra,
 * porque se comparan contra la máquina y no entre ellos.
 */
export function StatSheet({ stats, capacidad }: StatSheetProps) {
  return (
    <div className="asheet">
      <ul className="asheet__attrs">
        {ATRIBUTOS.map(({ key, label }) => {
          const valor = stats[key];

          return (
            <li className={`astat${valor >= TOPE ? ' is-full' : ''}`} key={key}>
              <span className="astat__label">{label}</span>
              <span className="astat__value">{valor}</span>
              <span className="astat__bar">
                <span className="astat__fill" style={{ width: `${valor}%` }} />
              </span>
            </li>
          );
        })}
      </ul>

      <p className="asheet__cost">
        <Coste label="CPU" pide={stats.cpu} hay={capacidad?.cpu} />
        <Coste label="RAM" pide={stats.ram} hay={capacidad?.ram} />
        <Coste label="RED" pide={stats.net} hay={capacidad?.net} />
        <span className="acost">
          <span className="acost__label">DISCO</span>
          <span className="acost__value">{stats.weight}</span>
        </span>
      </p>
    </div>
  );
}

/** Un coste, marcado cuando la máquina no llega. */
function Coste({ label, pide, hay }: { label: string; pide: number; hay?: number }) {
  const corto = hay !== undefined && hay < pide;

  return (
    <span className={`acost${corto ? ' is-short' : ''}`}>
      <span className="acost__label">{label}</span>
      <span className="acost__value">
        {pide}
        {hay !== undefined && <small> / {hay}</small>}
      </span>
    </span>
  );
}
