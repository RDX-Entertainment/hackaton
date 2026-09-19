import { tr, useIdioma } from '../../i18n/text';
import { useEffect, useRef, useState } from 'react';

import type { MachineDetail, MachineLink } from '../../utils/api';
import { formatIp } from '../../utils/ip';

/** Medidas del nodo. El centro es de donde salen los cables. */
const NODO_ANCHO = 160;
const NODO_ALTO = 62;

/** El nodo de Internet es redondo; su centro está a media caja. */
const WEB_ANCHO = 160;
const WEB_ALTO = 74;

/** Un gesto que se mueve menos que esto no fue un arrastre, fue un clic. */
const UMBRAL_CLIC = 4;

type MachineNodeMapProps = {
  machines: MachineDetail[];
  links: MachineLink[];
  selected: number;
  busy: boolean;
  /** La dirección del jugador, la que se ve desde fuera. */
  gameIp: number | null | undefined;
  onSelect: (machine: number) => void;
  onMove: (machine: number, x: number, y: number) => void;
  onLink: (from: number, to: number) => void;
  onUnlink: (link: MachineLink) => void;
};

type Arrastre = {
  machine: number;
  /** Distancia entre donde se agarró y la esquina del nodo, para que no salte al cogerlo. */
  dx: number;
  dy: number;
  movido: number;
};

/**
 * El mapa: dónde está cada equipo, qué está cableado a qué, y cuál sale a Internet.
 *
 * Está montado como el de las redes de organización —mismo gesto de arrastre, mismas bocas
 * LAN, mismo nodo de Internet colgando de los que están expuestos—, con una diferencia en
 * lo que decide esa exposición. Allí es un campo, `zone = 'public'`, porque un NPC se
 * declara público a mano. Aquí no hace falta declarar nada: **un equipo sale a Internet si
 * tiene tarjeta de red puesta**, y eso ya está en su ficha.
 *
 * Es la misma regla que da nombre al kit «servidor sin salida»: sin tarjeta, el equipo
 * existe y habla por LAN con los suyos, pero no lo alcanza nadie de fuera. Sacar la tarjeta
 * y ver desaparecer su línea a Internet es la explicación más corta de para qué sirve.
 *
 * La posición de los equipos se guarda en el servidor. La del nodo de Internet **no**: no
 * es un equipo, es una referencia para leer el dibujo, y cada uno la pone donde le estorbe
 * menos. Es lo mismo que hace el mapa de organizaciones.
 */
export function MachineNodeMap({
  machines,
  links,
  selected,
  busy,
  gameIp,
  onSelect,
  onMove,
  onLink,
  onUnlink,
}: MachineNodeMapProps) {
  useIdioma();
  const lienzo = useRef<HTMLDivElement>(null);

  const [arrastre, setArrastre] = useState<Arrastre | null>(null);
  const [cableando, setCableando] = useState<number | null>(null);
  const [web, setWeb] = useState({ x: 24, y: 150 });
  const [moviendoWeb, setMoviendoWeb] = useState<{ dx: number; dy: number } | null>(null);

  /**
   * Las posiciones mientras se arrastra. Sale de las props y solo se separa de ellas
   * durante el gesto: si el nodo esperase a que respondiera el servidor para moverse, el
   * arrastre iría a tirones.
   */
  const [posiciones, setPosiciones] = useState<Record<number, { x: number; y: number }>>({});

  useEffect(() => {
    setPosiciones(
      Object.fromEntries(
        machines.map(({ machine }) => [machine.id, { x: machine.map_x, y: machine.map_y }]),
      ),
    );
  }, [machines]);

  const posicionDe = (id: number) => posiciones[id] ?? { x: 0, y: 0 };
  const centroDe = (id: number) => {
    const { x, y } = posicionDe(id);
    return { x: x + NODO_ANCHO / 2, y: y + NODO_ALTO / 2 };
  };
  const centroWeb = { x: web.x + WEB_ANCHO / 2, y: web.y + WEB_ALTO / 2 };

  /** Con tarjeta de red puesta. La placa va aparte, en `board`, y no cuenta. */
  const tieneSalida = (detail: MachineDetail) =>
    detail.components.some((pieza) => pieza.kind === 'net');

  const expuestos = machines.filter(tieneSalida);

  const nombreDe = (id: number) =>
    machines.find(({ machine }) => machine.id === id)?.machine.name ?? tr("Equipo desconocido");

  /**
   * Qué ocupa cada boca del equipo, en orden.
   *
   * La salida a Internet va primera y cuenta como una: un equipo con tarjeta de red ya está
   * enchufado a algo aunque ese algo no se dibuje como cable. Es la misma cuenta que hace el
   * mapa de organizaciones, donde un nodo público lleva `INTERNET` entre sus conexiones.
   */
  const ocupantesDe = (detail: MachineDetail) => {
    const peers = links.flatMap((link) => {
      if (link.from_server === detail.machine.id) {
        return [nombreDe(link.to_server)];
      }

      if (link.to_server === detail.machine.id) {
        return [nombreDe(link.from_server)];
      }

      return [];
    });

    return tieneSalida(detail) ? ['INTERNET', ...peers] : peers;
  };

  const puntoEnLienzo = (event: { clientX: number; clientY: number }) => {
    const marco = lienzo.current?.getBoundingClientRect();
    return marco ? { x: event.clientX - marco.left, y: event.clientY - marco.top } : null;
  };

  const empezarArrastre = (event: React.PointerEvent, id: number) => {
    if (busy) {
      return;
    }

    const punto = puntoEnLienzo(event);

    if (!punto) {
      return;
    }

    const { x, y } = posicionDe(id);
    event.currentTarget.setPointerCapture(event.pointerId);
    setArrastre({ machine: id, dx: punto.x - x, dy: punto.y - y, movido: 0 });
  };

  const moverPuntero = (event: React.PointerEvent) => {
    const punto = puntoEnLienzo(event);

    if (!punto) {
      return;
    }

    if (moviendoWeb) {
      setWeb({
        x: Math.max(0, Math.round(punto.x - moviendoWeb.dx)),
        y: Math.max(0, Math.round(punto.y - moviendoWeb.dy)),
      });

      return;
    }

    if (!arrastre) {
      return;
    }

    const x = Math.max(0, Math.round(punto.x - arrastre.dx));
    const y = Math.max(0, Math.round(punto.y - arrastre.dy));
    const previa = posicionDe(arrastre.machine);

    setArrastre({
      ...arrastre,
      movido: arrastre.movido + Math.abs(x - previa.x) + Math.abs(y - previa.y),
    });
    setPosiciones((actuales) => ({ ...actuales, [arrastre.machine]: { x, y } }));
  };

  const soltarArrastre = (id: number) => {
    if (!arrastre || arrastre.machine !== id) {
      return;
    }

    // Un clic no es un arrastre: si apenas se movió, lo que quería era seleccionar o
    // cablear, y guardar la posición sería escribir en el servidor sin que nada cambiara.
    if (arrastre.movido <= UMBRAL_CLIC) {
      setArrastre(null);

      if (cableando !== null && cableando !== id) {
        onLink(cableando, id);
        setCableando(null);
      } else if (cableando === id) {
        setCableando(null);
      } else {
        onSelect(id);
      }

      return;
    }

    const { x, y } = posicionDe(id);
    setArrastre(null);
    onMove(id, x, y);
  };

  if (!machines.length) {
    return null;
  }

  const masBajo = Math.max(
    ...machines.map(({ machine }) => posicionDe(machine.id).y + NODO_ALTO),
    expuestos.length ? web.y + WEB_ALTO : 0,
  );

  return (
    <div
      className={`nodemap${cableando !== null ? ' is-linking' : ''}`}
      ref={lienzo}
      style={{ height: `${Math.max(masBajo + 40, 260)}px` }}
      onPointerMove={moverPuntero}
      onPointerUp={() => setMoviendoWeb(null)}
    >
      <span className="nodemap__legend">
        {cableando === null
          ? tr("Arrastra para colocar · pulsa una boca libre y luego otro equipo para cablear")
          : tr("Pulsa el equipo con el que conectar · pulsa otra vez la boca para cancelar")}
      </span>

      <svg className="nodemap__cables" aria-hidden="true">
        {/* La salida a Internet: no es un cable que se pueda quitar desde aquí, se quita
            sacando la tarjeta de red. Por eso va punteada y no responde al ratón. */}
        {expuestos.map(({ machine }) => {
          const desde = centroDe(machine.id);

          return (
            <line
              className="nodemap__uplink"
              key={`web-${machine.id}`}
              x1={desde.x}
              y1={desde.y}
              x2={centroWeb.x}
              y2={centroWeb.y}
            />
          );
        })}

        {links.map((link) => {
          const a = centroDe(link.from_server);
          const b = centroDe(link.to_server);

          return (
            <line
              className="nodemap__cable"
              key={link.id}
              x1={a.x}
              y1={a.y}
              x2={b.x}
              y2={b.y}
              onClick={() => !busy && onUnlink(link)}
            >
              <title>{tr("Quitar este cable")}</title>
            </line>
          );
        })}
      </svg>

      {expuestos.length > 0 && (
        <div
          className="nodemap__web"
          style={{ left: `${web.x}px`, top: `${web.y}px`, width: `${WEB_ANCHO}px` }}
          onPointerDown={(event) => {
            const punto = puntoEnLienzo(event);

            if (!punto) {
              return;
            }

            event.currentTarget.setPointerCapture(event.pointerId);
            setMoviendoWeb({ dx: punto.x - web.x, dy: punto.y - web.y });
          }}
        >
          <span className="nodemap__web-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24">
              <path d="M4 7h16M4 12h16M4 17h16M7 4v16M12 4v16M17 4v16" />
            </svg>
          </span>
          <b>INTERNET</b>
          {/*
            Ya no dice «la IP expuesta»: desde que cada maquina lleva la suya, esa frase
            describia algo que no existe. Lo que queda aqui es la direccion de la cuenta, que
            es por la que te conocen el perfil y el ranking hasta que la fase 3 la derive del
            gateway.
          */}
          <small>{tr("cuenta ·")} {formatIp(gameIp)}</small>
        </div>
      )}

      {machines.map((detail) => {
        const { machine, slots, board } = detail;
        const { x, y } = posicionDe(machine.id);
        const bocas = slots.net_total;
        const ocupantes = ocupantesDe(detail);
        const salida = tieneSalida(detail);

        return (
          <div
            className={[
              'nodemap__node',
              salida ? 'nodemap__node--public' : 'nodemap__node--internal',
              machine.id === selected ? 'is-selected' : '',
              cableando === machine.id ? 'is-source' : '',
            ]
              .filter(Boolean)
              .join(' ')}
            key={machine.id}
            style={{ left: `${x}px`, top: `${y}px`, width: `${NODO_ANCHO}px` }}
            onPointerDown={(event) => empezarArrastre(event, machine.id)}
            onPointerUp={() => soltarArrastre(machine.id)}
          >
            <small className="nodemap__zone">{salida ? 'salida' : 'interno'}</small>
            <b className="nodemap__name">{machine.name}</b>
            {/*
              La direccion por la que se llega a esta maquina, y solo esa: la publica si sale
              a Internet, la privada si no. Enseñar las dos a la vez seria pedirle al jugador
              que averigue cual usar; la que vale es la que corresponde a como esta hoy.
            */}
            <small className="nodemap__ip">
              {salida ? formatIp(machine.public_ip) : `lan ${formatIp(machine.lan_ip)}`}
            </small>
            <small className="nodemap__board">{board?.name ?? tr("sin placa")}</small>

            <span className="nodemap__ports">
              {Array.from({ length: Math.max(0, bocas) }).map((_, indice) => {
                const ocupante = ocupantes[indice];
                const ocupada = Boolean(ocupante);

                return (
                  <em
                    key={indice}
                    className={[
                      ocupada ? 'is-used' : '',
                      ocupante === 'INTERNET' ? 'is-uplink' : '',
                    ]
                      .filter(Boolean)
                      .join(' ')}
                    title={ocupada ? tr("Ocupada: {0}", ocupante) : tr("Boca libre: pulsa para cablear")}
                    onPointerDown={(event) => {
                      // Sin esto el gesto lo captura el nodo y empieza un arrastre.
                      event.stopPropagation();
                    }}
                    onClick={(event) => {
                      event.stopPropagation();

                      if (busy || ocupada) {
                        return;
                      }

                      setCableando((actual) => (actual === machine.id ? null : machine.id));
                    }}
                  />
                );
              })}
              {bocas === 0 && <i className="nodemap__noports">{tr("sin bocas")}</i>}
            </span>
          </div>
        );
      })}
    </div>
  );
}
