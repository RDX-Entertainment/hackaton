import { useCallback, useEffect, useState } from 'react';

import type { SiteAction, SiteDocument } from '../../utils/api';
import { parseIp } from '../../utils/ip';
import { SiteFrame } from './SiteFrame';
import type { SiteMessage } from './siteRuntime';

type SiteNavigatorProps = {
  ip: number;
  name: string;
  down: boolean;
  html: string | null;
  site: SiteDocument | null;
  onVisit: (ip: number) => void;
  onPay: (action: SiteAction) => void;
};

/**
 * Un sitio con navegación propia.
 *
 * Los botones son del autor y viven dentro de la página, con su HTML y su CSS. Lo que pasa
 * aquí fuera es solo obedecerlos: el marco avisa del clic y esto decide qué hacer, porque
 * cambiar de máquina o pedir un PIN son cosas del juego y no del sitio que se está mirando.
 *
 * Las pestañas de arriba las pinta el navegador, no el sitio. Son la garantía de que una
 * página declarada se pueda alcanzar aunque su autor no haya puesto ningún enlace hacia ella.
 */
export function SiteNavigator({ ip, name, down, html, site, onVisit, onPay }: SiteNavigatorProps) {
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
    (message: SiteMessage) => {
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

      if (message.kind === 'transfer' || message.kind === 'charge') {
        onPay({
          kind: message.kind,
          target: message.target,
          label: message.params.label ?? 'Pago',
          params: message.params,
        });
      }
    },
    [onVisit, onPay],
  );

  const open = pages.find((page) => page.id === openId) ?? pages[0];
  const named = pages.filter((page) => page.title);

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

      {named.length > 1 && (
        <nav className="site-tabs">
          {named.map((page) => (
            <button
              type="button"
              key={page.id}
              className={page.id === open?.id ? 'is-active' : ''}
              onClick={() => setOpenId(page.id)}
            >
              {page.title}
            </button>
          ))}
        </nav>
      )}

      <SiteFrame
        ip={ip}
        html={open?.html ?? html}
        name={name}
        down={down}
        onAction={down ? undefined : handle}
      />
    </>
  );
}
