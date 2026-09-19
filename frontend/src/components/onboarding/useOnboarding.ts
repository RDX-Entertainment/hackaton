import { useCallback, useEffect, useRef, useState } from 'react';

import { onboardingApi, type Onboarding } from '../../utils/api';

/**
 * El paso en el que está el jugador, preguntado al servidor.
 *
 * Se recarga cada pocos segundos porque el circuito avanza **fuera de React**: quien compila
 * espera a que el worker termine, y una portada que dijera «compila el diseño» diez minutos
 * después de haberlo compilado sería peor que no decir nada.
 *
 * Un fallo conserva la última respuesta: una desconexión no completa el tutorial ni quita
 * las restricciones que el jugador estaba viendo. Antes de la primera respuesta no hay datos.
 */
export function useOnboarding(intervalMs = 4000) {
  const [onboarding, setOnboarding] = useState<Onboarding>();
  const revision = useRef(0);
  const completed = useRef(false);

  const reload = useCallback(() => {
    if (completed.current) return;
    const request = ++revision.current;
    onboardingApi
      .getOnboarding()
      .then((value) => {
        if (request !== revision.current) return;
        completed.current = value.completed === true;
        setOnboarding(value);
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    if (onboarding?.completed) return;
    reload();
    window.addEventListener('game:changed', reload);
    const timer = intervalMs ? window.setInterval(reload, intervalMs) : undefined;
    return () => {
      ++revision.current;
      window.clearInterval(timer);
      window.removeEventListener('game:changed', reload);
    };
  }, [reload, intervalMs, onboarding?.completed]);

  return { onboarding, reload };
}
