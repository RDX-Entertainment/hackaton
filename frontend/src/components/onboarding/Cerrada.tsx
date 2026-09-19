import { tr } from '../../i18n/text';
import { Link } from 'react-router-dom';

import { useTutorial } from '../../app/tutorial';
import { t } from '../../i18n';
import { RUTA_DEL_PASO } from './circuito';

/*
 * A dónde lleva el paso: primero lo que diga el paso —la pantalla del foco, que se escribe en
 * el panel— y sólo si no dice nada, el mapa de rutas de este cliente. Al revés, un paso
 * escrito hoy en el panel no llevaría a ninguna parte.
 */

/**
 * Una pantalla que el tutorial todavía no abre.
 *
 * Se enseña la pantalla cerrada, con su nombre y el motivo, y **el camino de vuelta**: lo que
 * hay que hacer para abrirla y el botón que lleva allí. Un «no puedes» sin la otra mitad es
 * un callejón sin salida con buenos modales.
 */
export function Cerrada({ titulo, motivo }: { titulo: string; motivo: string }) {
  const tutorial = useTutorial();
  const paso = tutorial.paso;

  return (
    <section className="cerrada">
      {/*
        La marca es un cuadro rayado dibujado con CSS y no un carácter: `▚` y compañía no
        están en JetBrains Mono, y lo que sale entonces es el glifo que le apetezca al
        navegador. Además es la misma trama que lleva lo cerrado en el carril, que es donde
        el jugador ya la ha visto.
      */}
      <span className="cerrada__marca" aria-hidden="true" />

      <h3>{titulo}</h3>
      <p className="cerrada__motivo">{motivo}</p>

      {paso && (
        <>
          <p className="cerrada__que">
            {t(`onboarding.${paso}.hint`) ?? tutorial.onboarding?.rules?.hint}
          </p>

          <Link
            className="cerrada__ir"
            to={`/${tutorial.onboarding?.rules?.focus_view ?? RUTA_DEL_PASO[paso]?.slice(1) ?? ''}`}
          >
            {t(`onboarding.${paso}.action`) ?? t('tutorial.go') ?? tr("Ir al paso")}
          </Link>
        </>
      )}
    </section>
  );
}
