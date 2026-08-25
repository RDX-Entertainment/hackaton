import { useCallback, useEffect, useState } from 'react';

import type { SiteAction, SiteDocument } from '../../utils/api';
import { parseIp } from '../../utils/ip';
import { SiteFrame } from './SiteFrame';
import type { SiteFill, SiteMessage } from './siteRuntime';

type SiteNavigatorProps = {
  ip: number;
  name: string;
  down: boolean;
  html: string | null;
  site: SiteDocument | null;
  /** Lo que esta web tiene permitido hacer. Lo que no esté aquí no se obedece. */
  allowed: string[];
  onVisit: (ip: number) => void;
  onPay: (action: SiteAction) => void;
  /** Las órdenes de banca las atiende la sesión, que responde con datos para la página. */
  onBank: (message: SiteMessage, reply: (fill: SiteFill) => void) => void;
  /** Repinta lo que la sesión ya sabe en la página que acaba de cargar. */
  onRestore: (reply: (fill: SiteFill) => void) => void;
};

/**
 * Un sitio con navegación propia.
 *
 * Los botones son del autor y viven dentro de la página, con su HTML y su CSS. Lo que pasa
 * aquí fuera es solo obedecerlos: el marco avisa del clic y esto decide qué hacer, porque
 * cambiar de máquina o pedir un PIN son cosas del juego y no del sitio que se está mirando.
 *
 * El navegador no pinta ninguna navegación propia: las secciones las enlaza el sitio desde
 * su HTML, con sus botones y su CSS. Antes había unas pestañas aquí fuera y lo único que
 * hacían era repetir la navegación que el sitio ya traía.
 *
 * El precio de eso es que una página sin ningún enlace hacia ella queda inalcanzable. Es
 * responsabilidad de quien escribe el sitio, igual que en la web de verdad.
 */
export function SiteNavigator({
  ip,
  name,
  down,
  html,
  site,
  allowed,
  onVisit,
  onPay,
  onBank,
  onRestore,
}: SiteNavigatorProps) {
  const pages = site?.pages ?? [];
  const [openId, setOpenId] = useState(pages[0]?.id ?? '');
  const [ignored, setIgnored] = useState(false);

  useEffect(() => {
    setOpenId(pages[0]?.id ?? '');
    setIgnored(false);
    // Al cambiar de máquina se vuelve a su portada, no a la página que se miraba en la otra.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [ip]);

  const handle = useCallback(
    (message: SiteMessage, reply: (fill: SiteFill) => void) => {
      // Una orden que esta web no tiene concedida no se obedece, esté escrita o no. Que el
      // servidor la mande en la lista es lo único que la hace real.
      if (!allowed.includes(message.kind)) {
        return;
      }

      if (message.kind === 'link') {
        setOpenId(message.target);
        return;
      }

      if (message.kind === 'go_to') {
        const target = parseIp(message.target);

        if (target !== null) {
          onVisit(target);
        }

        return;
      }

      if (message.kind.startsWith('bank_')) {
        onBank(message, reply);

        // El banco decide a dónde llevar al cliente después de cada operación.
        if (message.params.then) {
          setOpenId(message.params.then);
        }

        return;
      }

      if (message.kind === 'transfer' || message.kind === 'charge') {
        onPay({
          kind: message.kind,
          target: message.target,
          label: message.params.label ?? 'Pago',
          params: message.params,
        });
      }
    },
    [allowed, onVisit, onPay, onBank],
  );

  const open = pages.find((page) => page.id === openId) ?? pages[0];

  return (
    <>
      {site?.redirect && !ignored && (
        <div className="site-redirect">
          <span>
            Este sitio te manda a <b>{site.redirect}</b>.
          </span>
          <span>
            <button type="button" onClick={() => setIgnored(true)}>
              Quedarme
            </button>
            <button
              type="button"
              className="is-primary"
              onClick={() => {
                const target = parseIp(site.redirect ?? '');
                if (target !== null) onVisit(target);
              }}
            >
              Ir
            </button>
          </span>
        </div>
      )}

      <SiteFrame
        ip={ip}
        html={open?.html ?? html}
        name={name}
        down={down}
        onAction={down ? undefined : handle}
        onReady={onRestore}
      />
    </>
  );
}
