import { useEffect, useRef, useState } from 'react';

import { formatIp } from '../../utils/ip';
import { isSiteMessage, withRuntime, type SiteFill, type SiteMessage } from './siteRuntime';

/**
 * El contenedor donde se pinta la página que sirve una máquina.
 *
 * Va en un `iframe` con `sandbox`. La única autorización que lleva es `allow-scripts`, y
 * **nunca `allow-same-origin`**: sin lo segundo el documento queda en un origen opaco, así
 * que no alcanza el DOM de la aplicación, ni su almacenamiento, ni sus cookies.
 *
 * Los scripts hacen falta para un solo guion, el que inyecta el juego para enterarse de los
 * clics en los botones de un sitio. El HTML del autor sigue sin poder traer código: el
 * servidor rechaza `<script>` y los `on…=` al guardar.
 *
 * El aislamiento va en las dos direcciones, y eso es lo que hace que funcione: cada sitio
 * trae su propio CSS —fondos claros, tipografías con serifa, lo que sea— sin pisar el del
 * juego, y el del juego no se cuela dentro para uniformarlo todo.
 */

/** Lo que se sirve cuando no hay nada configurado. Un servidor recién instalado. */
function blankPage(ip: number): string {
  return `<style>
    body{margin:0;background:#fff;color:#222;font:14px/1.6 -apple-system,Segoe UI,Arial,sans-serif}
    .box{max-width:34rem;margin:3rem auto;padding:0 1.5rem}
    h1{margin:0 0 .4rem;font-size:22px;font-weight:600}
    p{margin:.6rem 0;color:#555}
    code{padding:1px 5px;background:#f1f1f1;font-size:13px}
    hr{margin:1.6rem 0;border:0;border-top:1px solid #e2e2e2}
    .foot{color:#999;font-size:12px}
  </style>
  <div class="box">
    <h1>Funciona</h1>
    <p>Este servidor está en marcha pero todavía no sirve ningún contenido.</p>
    <p>Si has llegado hasta aquí buscando algo, no hay nada que ver: el disco está detrás
    del acceso, no de esta página.</p>
    <hr>
    <p class="foot">httpd &middot; ${formatIp(ip)}</p>
  </div>`;
}

type SiteFrameProps = {
  ip: number;
  html: string | null;
  name: string;
  down: boolean;
  /**
   * Se llama al pulsar un elemento con `data-action`. Sin esto el marco es solo lectura.
   * El segundo argumento devuelve datos al sitio para que rellene sus `data-bind`.
   */
  onAction?: (message: SiteMessage, reply: (fill: SiteFill) => void) => void;
  /**
   * Se llama cada vez que el marco termina de pintar una página.
   *
   * Cambiar de página reemplaza el documento entero, así que lo que se hubiera rellenado
   * antes se va con él. Quien guarda el estado es el padre, y este aviso es su oportunidad
   * de volver a ponerlo: sin esto, ir a movimientos y volver dejaba el saldo en blanco.
   */
  onReady?: (reply: (fill: SiteFill) => void) => void;
};

export function SiteFrame({ ip, html, name, down, onAction, onReady }: SiteFrameProps) {
  const [big, setBig] = useState(false);
  const frame = useRef<HTMLIFrameElement>(null);

  useEffect(() => {
    if (!onAction) {
      return;
    }

    const listener = (event: MessageEvent) => {
      // Solo se atiende a nuestro propio marco: cualquier otra ventana que mande un mensaje
      // parecido se ignora.
      if (event.source !== frame.current?.contentWindow || !isSiteMessage(event.data)) {
        return;
      }

      onAction(event.data, (fill) => frame.current?.contentWindow?.postMessage(fill, '*'));
    };

    window.addEventListener('message', listener);
    return () => window.removeEventListener('message', listener);
  }, [onAction]);

  if (down) {
    return (
      <div className="site site--down">
        <p className="site__error">
          <b>No se pudo establecer conexión</b>
          <span>La máquina de {formatIp(ip)} no responde.</span>
        </p>
      </div>
    );
  }

  const served = Boolean(html && html.trim());
  const page = served ? (html as string) : blankPage(ip);
  const source = onAction ? withRuntime(page) : page;

  return (
    <div className={big ? 'site is-big' : 'site'}>
      <div className="site__bar">
        <span className="site__label">
          {served ? 'sirviendo' : 'sin contenido'} · {formatIp(ip)}
        </span>

        <button type="button" className="site__zoom" onClick={() => setBig((one) => !one)}>
          {big ? 'reducir' : 'ampliar'}
        </button>
      </div>

      <iframe
        ref={frame}
        className="site__frame"
        title={`Página de ${name || formatIp(ip)}`}
        sandbox={onAction ? 'allow-scripts' : ''}
        loading="lazy"
        srcDoc={source}
        onLoad={() =>
          onReady?.((fill) => frame.current?.contentWindow?.postMessage(fill, '*'))
        }
      />
    </div>
  );
}
