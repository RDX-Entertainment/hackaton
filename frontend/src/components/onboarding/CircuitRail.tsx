import { Link } from 'react-router-dom';

import { t } from '../../i18n';
import type { Onboarding } from '../../utils/api';
import { RUTA_DEL_PASO } from './circuito';

/**
 * El circuito de un programa, de un vistazo: investigar, diseñar, compilar, cargar y entrar.
 *
 * Las cinco piezas existían desde hace semanas repartidas en cinco pantallas, y ninguna
 * decía que fuese un camino ni en qué punto estabas. El riel es lo único que comparten:
 * se pinta igual en Arsenal, en Software y en Investigación, así que ir de una a otra no
 * es cambiar de sitio, es avanzar un paso.
 *
 * Lo hecho se marca y sigue siendo un enlace: volver a un paso terminado es normal —se
 * compila un segundo programa— y bloquearlo sería confundir «ya lo hiciste» con «no puedes».
 */
export function CircuitRail({ onboarding }: { onboarding?: Onboarding }) {
  if (!onboarding) {
    return null;
  }

  return (
    <nav className="circuit" aria-label={t('onboarding.rail.title') ?? 'Circuito'}>
      <ol className="circuit__steps">
        {onboarding.steps.map((step, index) => {
          const actual = onboarding.current === step.code;
          const estado = step.done ? ' is-done' : actual ? ' is-current' : '';

          return (
            <li className={`circuit__step${estado}`} key={step.code}>
              <Link to={RUTA_DEL_PASO[step.code] ?? '/'} aria-current={actual ? 'step' : undefined}>
                <span className="circuit__index" aria-hidden="true">
                  {step.done ? '✓' : index + 1}
                </span>
                <span className="circuit__label">{t(`onboarding.${step.code}.step`)}</span>
              </Link>
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
