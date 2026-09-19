import { tr, useIdioma } from '../../i18n/text';
import { useEffect, useState } from 'react';

import type { Remote, SiteAction } from '../../utils/api';
import { formatCount } from '../../utils/format';
import { formatIp } from '../../utils/ip';
import { SiteNavigator } from './SiteNavigator';
import { useBankSession } from './useBankSession';
import { SitePayDialog } from './SitePayDialog';

type PublicTabProps = {
  remote: Remote;
  onToggleBookmark: () => void;
  onVisit: (ip: number) => void;
};

/**
 * Lo que cualquiera ve al llamar a una puerta: quién vive ahí y si contesta.
 * Nada de esto exige haber entrado, y por eso es la pestaña que siempre carga.
 */
export function PublicTab({ remote, onToggleBookmark, onVisit }: PublicTabProps) {
  useIdioma();
  const [paying, setPaying] = useState<SiteAction | null>(null);
  const { holder: client, handle: onBank, restore: onRestore } = useBankSession(remote.bank, remote.ip);

  // Un cuadro de pago pertenece al sitio que lo abriÃ³. Al navegar a otra IP no puede quedar
  // por encima de la nueva pÃ¡gina ni conservar una orden dirigida a la mÃ¡quina anterior.
  useEffect(() => {
    setPaying(null);
  }, [remote.ip]);

  const rows = [
    { key: tr("dirección"), value: formatIp(remote.ip) },
    { key: 'titular', value: remote.kind === 'npc' ? tr("servidor NPC · tipo {0}", remote.type) : 'jugador' },
    { key: 'estado', value: remote.down ? tr("caído") : 'responde' },
    { key: 'puertos', value: remote.scanned ? 'escaneados' : tr("sin escanear") },
    {
      key: 'acceso',
      // Entrar en el banco no es entrar en la máquina: una cosa es ser cliente y otra tener
      // una sesión en el servidor. Se dicen las dos porque no son la misma.
      value: client
        ? tr("cliente · {0}", client)
        : remote.access
          ? tr("atacante · {0}", remote.access_as ?? 'root')
          : tr("sin sesión"),
    },
  ];

  return (
    <div className="page">
      <div className="page__banner">
        <pre className="page__art" aria-hidden="true">
          {String.raw` ___  _   _ ___ _    ___ ___ ___
| _ \| | | | _ ) |  |_ _/ __/ _ \
|  _/| |_| | _ \ |__ | | (_| (_) |
|_|   \___/|___/____|___\___\___/`}
        </pre>

        <div className="page__id">
          <h3>{remote.name || tr("máquina sin nombre")}</h3>
          <p className={remote.down ? 'page__state is-down' : 'page__state'}>
            {remote.down ? tr("no responde") : tr("en línea")}
          </p>
        </div>

        <button
          type="button"
          className={remote.bookmarked ? 'page__mark is-on' : 'page__mark'}
          aria-pressed={remote.bookmarked}
          onClick={onToggleBookmark}
        >
          <span aria-hidden="true">★</span>
          {remote.bookmarked ? 'guardada' : tr("guardar")}
        </button>
      </div>

      <SiteNavigator
        ip={remote.ip}
        name={remote.name}
        down={remote.down}
        html={remote.web}
        site={remote.site}
        allowed={remote.allowed}
        onVisit={onVisit}
        onPay={setPaying}
        onBank={onBank}
        onRestore={onRestore}
      />

      <SitePayDialog action={paying} onClose={() => setPaying(null)} />

      <dl className="page__facts">
        {rows.map((row) => (
          <div key={row.key}>
            <dt>{row.key}</dt>
            <dd>{row.value}</dd>
          </div>
        ))}
      </dl>

      {remote.analysis ? (
        <div className="page__analysis">
          <h4>{tr("Lo que devolvió el análisis")}</h4>

          <dl>
            <div>
              <dt>{tr("proceso")}</dt>
              <dd>
                {formatCount(remote.analysis.cpu_min)} – {formatCount(remote.analysis.cpu_max)} MHz
              </dd>
            </div>
            <div>
              <dt>{tr("memoria")}</dt>
              <dd>
                {formatCount(remote.analysis.ram_min)} – {formatCount(remote.analysis.ram_max)} MB
              </dd>
            </div>
          </dl>

          <p className="page__note">
            {tr("Es una horquilla, no una medida: cuanto mejor tu analizador, más estrecha.")} </p>
        </div>
      ) : (
        <p className="page__note">
          {tr("Nadie ha analizado esta máquina todavía. Lánzalo desde")} <b>force</b> {tr("y el resultado aparecerá aquí.")} </p>
      )}
    </div>
  );
}
