import { useState } from 'react';

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
  const [paying, setPaying] = useState<SiteAction | null>(null);
  const { holder: client, handle: onBank, restore: onRestore } = useBankSession(remote.bank);
  const rows = [
    { key: 'dirección', value: formatIp(remote.ip) },
    { key: 'titular', value: remote.kind === 'npc' ? `servidor NPC · tipo ${remote.type}` : 'jugador' },
    { key: 'estado', value: remote.down ? 'caído' : 'responde' },
    { key: 'puertos', value: remote.scanned ? 'escaneados' : 'sin escanear' },
    {
      key: 'acceso',
      // Entrar en el banco no es entrar en la máquina: una cosa es ser cliente y otra tener
      // una sesión en el servidor. Se dicen las dos porque no son la misma.
      value: client
        ? `cliente · ${client}`
        : remote.access
          ? 'sesión abierta'
          : 'sin sesión',
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
          <h3>{remote.name || 'máquina sin nombre'}</h3>
          <p className={remote.down ? 'page__state is-down' : 'page__state'}>
            {remote.down ? 'no responde' : 'en línea'}
          </p>
        </div>

        <button
          type="button"
          className={remote.bookmarked ? 'page__mark is-on' : 'page__mark'}
          aria-pressed={remote.bookmarked}
          onClick={onToggleBookmark}
        >
          <span aria-hidden="true">★</span>
          {remote.bookmarked ? 'guardada' : 'guardar'}
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
          <h4>Lo que devolvió el análisis</h4>

          <dl>
            <div>
              <dt>proceso</dt>
              <dd>
                {formatCount(remote.analysis.cpu_min)} – {formatCount(remote.analysis.cpu_max)} MHz
              </dd>
            </div>
            <div>
              <dt>memoria</dt>
              <dd>
                {formatCount(remote.analysis.ram_min)} – {formatCount(remote.analysis.ram_max)} MB
              </dd>
            </div>
          </dl>

          <p className="page__note">
            Es una horquilla, no una medida: cuanto mejor tu analizador, más estrecha.
          </p>
        </div>
      ) : (
        <p className="page__note">
          Nadie ha analizado esta máquina todavía. Lánzalo desde <b>force</b> y el resultado
          aparecerá aquí.
        </p>
      )}
    </div>
  );
}
