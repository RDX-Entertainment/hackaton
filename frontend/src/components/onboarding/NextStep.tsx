import { Link } from 'react-router-dom';

import { t } from '../../i18n';
import type { Onboarding } from '../../utils/api';
import { formatIp } from '../../utils/ip';
import { RUTA_DEL_PASO } from './circuito';

/**
 * Qué toca ahora, y el botón que lleva allí.
 *
 * Es lo que la portada no decía. Antes eran tres números —dinero, procesos, correo— y las
 * noticias: un panel de estado para quien ya sabe jugar, y una pared para quien acaba de
 * registrarse.
 *
 * **Una sola cosa que hacer, no cinco.** Se enseña el primer paso que falta y nada más; el
 * resto del camino queda en el riel, que informa sin pedir nada.
 */
export function NextStep({ onboarding }: { onboarding: Onboarding }) {
  const paso = onboarding.current;

  if (!paso) {
    return (
      <section className="nextstep nextstep--done">
        <h3>{t('onboarding.done.title')}</h3>
        <p className="muted">{t('onboarding.done.hint')}</p>
      </section>
    );
  }

  // Quien está compilando no tiene nada que hacer salvo esperar, así que se le dice eso y
  // se le quita el botón: empujarle a compilar otra vez sería mandarle contra el error
  // COMPILATION_ACTIVE, que el propio servidor le devolvería.
  const esperando = paso === 'compile' && onboarding.compiling;

  return (
    <section className="nextstep">
      {/*
        El título y la explicación salen del catálogo de idiomas por el código del paso. Un
        paso escrito en el panel no está en ningún catálogo: entonces habla con lo que lleve
        escrito, que es lo que trae `hint`.
      */}
      <h3>{t(`onboarding.${paso}.title`) ?? paso}</h3>
      <p>
        {esperando
          ? t('onboarding.compile.waiting')
          : (t(`onboarding.${paso}.hint`) ?? onboarding.rules?.hint)}
      </p>

      {!esperando && (
        <Link
          className="nextstep__go"
          to={`/${onboarding.rules?.focus_view ?? RUTA_DEL_PASO[paso]?.slice(1) ?? ''}`}
        >
          {t(`onboarding.${paso}.action`) ?? t('tutorial.go')}
        </Link>
      )}

      {paso === 'intrusion' && onboarding.practice.length > 0 && (
        <div className="nextstep__practice">
          <b>{t('onboarding.practice.title')}</b>
          <ul>
            {onboarding.practice.map((objetivo) => (
              <li key={objetivo.ip}>
                <code>{formatIp(objetivo.ip)}</code>
                <small> {objetivo.name}</small>
              </li>
            ))}
          </ul>
        </div>
      )}
    </section>
  );
}
