import { createContext, useContext, useMemo, type ReactNode } from 'react';

import { useOnboarding } from '../components/onboarding/useOnboarding';
import { t, useIdioma } from '../i18n';
import type { Onboarding } from '../utils/api';

/**
 * El tutorial, tal y como lo ve una pantalla.
 *
 * El servidor ya veta por su cuenta —el bloqueo vive ahí, o se salta con `curl`—. Esto es lo
 * otro que hace falta: **explicarlo antes**. Un botón que se deja pulsar para contestar que
 * no se puede enseña a base de choques; apagado y con su motivo enseña lo que hay que hacer.
 *
 * Y se apaga, no se esconde. Quien no ve una pantalla cree que no existe; quien la ve
 * apagada sabe que hay algo que conseguir.
 */
export type Tutorial = {
  /** El paso actual, o `null` si el jugador ya está suelto. */
  paso: string | null;

  /** Si el paso actual bloquea algo. Con el tutorial apagado es `false` siempre. */
  manda: boolean;

  /** Si esa pantalla se puede abrir ahora. */
  permiteVista: (path: string) => boolean;

  /** Si esa acción se puede hacer ahora. */
  permiteAccion: (code: string) => boolean;

  /** El sitio donde tiene que mirar el jugador, si el paso señala alguno. */
  foco: string | null;

  /** Por qué está apagado lo que está apagado. Una frase, ya traducida. */
  motivo: string;

  onboarding?: Onboarding;
  recargar: () => void;
};

const SUELTO: Tutorial = {
  paso: null,
  manda: false,
  permiteVista: () => true,
  permiteAccion: () => true,
  foco: null,
  motivo: '',
  recargar: () => {},
};

const Contexto = createContext<Tutorial>(SUELTO);

/**
 * La frase que explica el bloqueo.
 *
 * Primero el título del paso del catálogo de idiomas —que es lo que este cliente sabe decir
 * en la lengua del jugador— y, si el paso no está en el catálogo porque lo escribió alguien
 * en el panel esta mañana, la frase que venga escrita en él. Quedarse mudo sería lo único
 * peor que hablar un solo idioma.
 */
function motivoDe(onboarding?: Onboarding): string {
  if (!onboarding?.current) {
    return '';
  }

  const titulo =
    t(`onboarding.${onboarding.current}.title`) ??
    onboarding.rules?.hint ??
    onboarding.current;

  // `t` devuelve `null` cuando no hay traducción, así que la plantilla lleva respaldo: un
  // catálogo incompleto no puede dejar un botón apagado y sin explicación.
  const plantilla = t('tutorial.locked') ?? 'Ahora no: {paso}';

  return plantilla.replace('{paso}', titulo);
}

export function TutorialProvider({ children }: { children: ReactNode }) {
  const idioma = useIdioma();
  const { onboarding, reload } = useOnboarding();

  const valor = useMemo<Tutorial>(() => {
    const rules = onboarding?.rules ?? null;
    const vistas = rules?.views ?? null;
    const acciones = rules?.actions ?? null;

    return {
      paso: onboarding?.current ?? null,
      manda: Boolean(vistas || acciones),
      permiteVista: (path) => !vistas || vistas.includes(path),
      permiteAccion: (code) => !acciones || acciones.includes(code),
      foco: rules?.focus_anchor ?? null,
      motivo: motivoDe(onboarding),
      onboarding,
      recargar: reload,
    };
  }, [onboarding, reload, idioma]);

  return <Contexto.Provider value={valor}>{children}</Contexto.Provider>;
}

export function useTutorial(): Tutorial {
  return useContext(Contexto);
}

/**
 * Lo que hace falta para un botón: si se puede pulsar y, si no, por qué.
 *
 * Devuelve también si **este** es el sitio señalado, que es lo que enciende el foco. Las dos
 * cosas juntas porque siempre se piden juntas: el botón que el paso resalta es justo el que
 * el paso deja pulsar.
 */
export function useAccion(code: string): {
  permitida: boolean;
  motivo: string | null;
  destacada: boolean;
} {
  const tutorial = useTutorial();
  const permitida = tutorial.permiteAccion(code);

  return {
    permitida,
    motivo: permitida ? null : tutorial.motivo,
    destacada: tutorial.foco === code,
  };
}
