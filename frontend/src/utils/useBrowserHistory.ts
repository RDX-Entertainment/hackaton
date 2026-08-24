import { useState } from 'react';

/**
 * La pila de un navegador, no la lista de sitios visitados.
 *
 * El servidor ya guarda por dónde has pasado, y eso es el historial: un registro
 * que crece y se queda. Esto es otra cosa — dónde estás dentro de una sesión de
 * navegación — y por eso vive solo en el cliente. Volver atrás no borra nada allí:
 * la visita ya ocurrió y la máquina remota se acuerda.
 *
 * Ir a un sitio nuevo estando a mitad de la pila corta lo que había delante, que es
 * lo que hace cualquier navegador.
 */
export type BrowserHistory = {
  current: number | null;
  canBack: boolean;
  canForward: boolean;
  push: (ip: number) => void;
  back: () => number | null;
  forward: () => number | null;
};

type Stack = {
  entries: number[];
  index: number;
};

const EMPTY: Stack = { entries: [], index: -1 };

export function useBrowserHistory(): BrowserHistory {
  const [stack, setStack] = useState<Stack>(EMPTY);

  const push = (ip: number) =>
    setStack(({ entries, index }) => {
      const kept = entries.slice(0, index + 1);

      // Recargar el sitio donde ya estás no añade una entrada.
      if (kept[kept.length - 1] === ip) {
        return { entries: kept, index: kept.length - 1 };
      }

      return { entries: [...kept, ip], index: kept.length };
    });

  // Devuelven la dirección a la que hay que ir para que quien llama la visite. El
  // salto y la visita son dos cosas: una mueve el cursor, la otra pide la máquina.
  const step = (delta: number): number | null => {
    const wanted = stack.index + delta;

    if (wanted < 0 || wanted >= stack.entries.length) {
      return null;
    }

    setStack((current) => ({ ...current, index: wanted }));

    return stack.entries[wanted];
  };

  return {
    current: stack.index >= 0 ? (stack.entries[stack.index] ?? null) : null,
    canBack: stack.index > 0,
    canForward: stack.index >= 0 && stack.index < stack.entries.length - 1,
    push,
    back: () => step(-1),
    forward: () => step(1),
  };
}
