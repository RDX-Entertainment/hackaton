import { useCallback, useEffect, useState } from 'react';

import { onboardingApi, type Onboarding } from '../../utils/api';

/**
 * El paso en el que está el jugador, preguntado al servidor.
 *
 * Se recarga cada pocos segundos porque el circuito avanza **fuera de React**: quien compila
 * espera a que el worker termine, y una portada que dijera «compila el diseño» diez minutos
 * después de haberlo compilado sería peor que no decir nada.
 *
 * Un fallo deja `undefined` y no rompe la pantalla: esto acompaña al juego, no lo sustituye.
 */
export function useOnboarding(intervalMs = 8000) {
  const [onboarding, setOnboarding] = useState<Onboarding>();

  const reload = useCallback(() => {
    onboardingApi
      .getOnboarding()
      .then(setOnboarding)
      .catch(() => setOnboarding(undefined));
  }, []);

  useEffect(() => {
    reload();

    if (!intervalMs) {
      return;
    }

    const timer = window.setInterval(reload, intervalMs);
    return () => window.clearInterval(timer);
  }, [reload, intervalMs]);

  return { onboarding, reload };
}
