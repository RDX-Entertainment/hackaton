import { useEffect, useRef, useState } from 'react';

import { formatIp } from '../../utils/ip';
import { attach, fill, type SiteFill, type SiteMessage } from './siteRuntime';

/**
 * El contenedor donde se pinta la página que sirve una máquina.
 *
 * Va en un `iframe` con `sandbox`. La única autorización que lleva es `allow-same-origin`, y
 * **nunca `allow-scripts`**: sin lo segundo el navegador no ejecuta un guion ahí dentro
 * aunque lo encuentre escrito. Esa es la diferencia con lo que había antes, que era justo al
 * revés —`allow-scripts` sin `allow-same-origin`, porque el juego metía su propio guion en el
 * marco para enterarse de los clics—. Ese trabajo lo hace ahora la aplicación desde fuera
 * (ver `siteRuntime`), así que el permiso sobra, y quitarlo convierte «el servidor rechaza
 * `<script>` al guardar» en «además, no correría».
 *
 * Lo que `allow-same-origin` concede es que la aplicación llegue al DOM del marco. En el par
 * conocido como peligroso los dos permisos van juntos, y lo son porque un guion de dentro
 * puede quitarle el `sandbox` a su propio marco. Sin `allow-scripts` no hay quien lo intente.
 *
 * El aislamiento de estilos sigue igual y es lo que hace que esto funcione: cada sitio trae su
 * propio CSS —fondos claros, tipografías con serifa, lo que sea— sin pisar el del juego, y el
 * del juego no se cuela dentro para uniformarlo todo.
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

  /** Lo que hay que soltar del documento anterior antes de engancharse al siguiente. */
  const release = useRef<(() => void) | null>(null);

  // Las dos llamadas viven en referencias porque quien las usa es el `onLoad` del marco, y
  // ese no se vuelve a montar cuando el padre se repinta.
  const action = useRef(onAction);
  const ready = useRef(onReady);
  action.current = onAction;
  ready.current = onReady;

  useEffect(() => () => release.current?.(), []);

  /**
   * El marco acaba de pintar una página. Es el único momento en que existe su documento:
   * cambiar de página lo reemplaza entero, y con él se va todo lo que hubiera enganchado.
   */
  const bound = () => {
    release.current?.();
    release.current = null;

    // Nulo en los marcos de solo lectura, que van con el `sandbox` cerrado del todo y por
    // eso quedan en un origen opaco. Ahí no hay nada que enganchar.
    const inside = frame.current?.contentDocument;

    if (!inside) {
      return;
    }

    const reply = (data: SiteFill) => {
      const current = frame.current?.contentDocument;

      if (current) {
        fill(current, data);
      }
    };

    const onSiteAction = action.current;

    if (onSiteAction) {
      release.current = attach(inside, (message) => onSiteAction(message, reply));
    }

    ready.current?.(reply);
  };

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
        sandbox={onAction ? 'allow-same-origin' : ''}
        loading="lazy"
        srcDoc={page}
        onLoad={bound}
      />
    </div>
  );
}
