import { tr, useIdioma } from '../../i18n/text';
import type { Capability, ProgramStats } from '../../utils/api';
import { visibleAttributes } from './attributes';

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
    get fase() { return tr("Ejecución"); },
    ataque: { key: 'effectiveness', get label() { return tr("Eficacia"); } },
    defensa: { key: 'resistance', get label() { return tr("Resistencia"); } },
  },
  {
    get fase() { return tr("Penetración"); },
    ataque: { key: 'penetration', get label() { return tr("Penetración"); } },
    defensa: { key: 'hardening', get label() { return tr("Blindaje"); } },
  },
  {
    get fase() { return tr("Detección"); },
    ataque: { key: 'stealth', get label() { return tr("Sigilo"); } },
    defensa: { key: 'detection', get label() { return tr("Detección"); } },
  },
  {
    get fase() { return tr("Manipulación"); },
    ataque: { key: 'manipulation', get label() { return tr("Manipulación"); } },
    defensa: { key: 'integrity', get label() { return tr("Integridad"); } },
  },
  {
    get fase() { return tr("Rastro"); },
    ataque: { key: 'anti_forensics', get label() { return tr("Anti-forense"); } },
    defensa: { key: 'forensics', get label() { return tr("Forense"); } },
  },
  {
    get fase() { return tr("Persistencia"); },
    ataque: { key: 'persistence', get label() { return tr("Persistencia"); } },
    defensa: { key: 'removal', get label() { return tr("Limpieza"); } },
  },
];

const TOPE = 100;

type StatSheetProps = {
  extraAttributes?: string[];
  stats: ProgramStats;
  capability?: Capability;
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
export function StatSheet({ stats, capacidad, capability, extraAttributes = [] }: StatSheetProps) {
  useIdioma();
  const visible = [...visibleAttributes(capability), ...extraAttributes];
  return (
    <div className="asheet">
      <ul className="asheet__pairs">
        {PARES.filter(({ ataque, defensa }) => visible.includes(ataque.key) || visible.includes(defensa.key)).map(({ fase, ataque, defensa }) => (
          <li className={`apair${visible.includes(ataque.key) && visible.includes(defensa.key) ? '' : ' apair--single'}`} key={fase}>
            <span className="apair__phase">{fase}</span>
            {visible.includes(ataque.key) && <Atributo label={ataque.label} valor={stats[ataque.key]} lado="atq" />}
            {visible.includes(defensa.key) && <Atributo label={defensa.label} valor={stats[defensa.key]} lado="def" />}
          </li>
        ))}
      </ul>

      {/*
        La estabilidad no tiene pareja: no la responde nadie, es cuánto aguanta el programa
        antes de romperse solo. Por eso va fuera de la rejilla de pares y no dentro.
      */}
      <p className="asheet__lone">
        <Atributo label={tr("Estabilidad")} valor={stats.stability} lado="neutro" />
      </p>

      <p className="asheet__cost">
        <Coste label="CPU" pide={stats.cpu} hay={capacidad?.cpu} />
        <Coste label="RAM" pide={stats.ram} hay={capacidad?.ram} />
        <Coste label="RED" pide={stats.net} hay={capacidad?.net} />
        <span className="acost">
          <span className="acost__label">{tr("DISCO")}</span>
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
  useIdioma();
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
  useIdioma();
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
