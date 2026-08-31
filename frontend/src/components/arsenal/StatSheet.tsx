import type { ProgramStats } from '../../utils/api';

/**
 * Los doce atributos de combate, en los seis pares que los enfrenta el motor.
 *
 * No son doce números sueltos: la tabla `combat_attribute_pairs` empareja cada atributo de
 * ataque con el que lo responde, y una fase se resuelve comparando **ese par y solo ese**. Un
 * Sigilo de 90 no dice nada por sí solo —dice algo frente a la Detección de quien tienes
 * delante—, así que enseñarlos en una rejilla plana esconde justo lo que hay que leer.
 *
 * El orden es el de las fases, que es el orden en que ocurren.
 */
const PARES: {
  fase: string;
  ataque: { key: keyof ProgramStats; label: string };
  defensa: { key: keyof ProgramStats; label: string };
}[] = [
  {
    fase: 'Ejecución',
    ataque: { key: 'effectiveness', label: 'Eficacia' },
    defensa: { key: 'resistance', label: 'Resistencia' },
  },
  {
    fase: 'Penetración',
    ataque: { key: 'penetration', label: 'Penetración' },
    defensa: { key: 'hardening', label: 'Blindaje' },
  },
  {
    fase: 'Detección',
    ataque: { key: 'stealth', label: 'Sigilo' },
    defensa: { key: 'detection', label: 'Detección' },
  },
  {
    fase: 'Manipulación',
    ataque: { key: 'manipulation', label: 'Manipulación' },
    defensa: { key: 'integrity', label: 'Integridad' },
  },
  {
    fase: 'Rastro',
    ataque: { key: 'anti_forensics', label: 'Anti-forense' },
    defensa: { key: 'forensics', label: 'Forense' },
  },
  {
    fase: 'Persistencia',
    ataque: { key: 'persistence', label: 'Persistencia' },
    defensa: { key: 'removal', label: 'Limpieza' },
  },
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
 * La hoja de un programa: doce atributos en seis pares, la estabilidad, y lo que cuesta
 * tenerlo corriendo.
 *
 * Los atributos llevan barra porque están todos en la misma escala de 0 a 100 y solo
 * significan algo unos al lado de otros; los costes van en cifra, porque se comparan contra la
 * máquina y no entre ellos.
 */
export function StatSheet({ stats, capacidad }: StatSheetProps) {
  return (
    <div className="asheet">
      <ul className="asheet__pairs">
        {PARES.map(({ fase, ataque, defensa }) => (
          <li className="apair" key={fase}>
            <span className="apair__phase">{fase}</span>
            <Atributo label={ataque.label} valor={stats[ataque.key]} lado="atq" />
            <Atributo label={defensa.label} valor={stats[defensa.key]} lado="def" />
          </li>
        ))}
      </ul>

      {/*
        La estabilidad no tiene pareja: no la responde nadie, es cuánto aguanta el programa
        antes de romperse solo. Por eso va fuera de la rejilla de pares y no dentro.
      */}
      <p className="asheet__lone">
        <Atributo label="Estabilidad" valor={stats.stability} lado="neutro" />
      </p>

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

/** Un atributo: etiqueta, cifra y barra. `lado` solo cambia el color, no la escala. */
function Atributo({
  label,
  valor,
  lado,
}: {
  label: string;
  valor: number;
  lado: 'atq' | 'def' | 'neutro';
}) {
  return (
    <span className={`astat astat--${lado}${valor >= TOPE ? ' is-full' : ''}`}>
      <span className="astat__label">{label}</span>
      <span className="astat__value">{valor}</span>
      <span className="astat__bar">
        <span className="astat__fill" style={{ width: `${valor}%` }} />
      </span>
    </span>
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
