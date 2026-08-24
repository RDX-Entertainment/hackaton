import type { Remote } from '../../utils/api';
import { formatCount } from '../../utils/format';
import { formatIp } from '../../utils/ip';
import { SiteFrame } from './SiteFrame';

type PublicTabProps = {
  remote: Remote;
  onToggleBookmark: () => void;
};

/**
 * Lo que cualquiera ve al llamar a una puerta: quién vive ahí y si contesta.
 * Nada de esto exige haber entrado, y por eso es la pestaña que siempre carga.
 */
export function PublicTab({ remote, onToggleBookmark }: PublicTabProps) {
  const rows = [
    { key: 'dirección', value: formatIp(remote.ip) },
    { key: 'titular', value: remote.kind === 'npc' ? `servidor NPC · tipo ${remote.type}` : 'jugador' },
    { key: 'estado', value: remote.down ? 'caído' : 'responde' },
    { key: 'puertos', value: remote.scanned ? 'escaneados' : 'sin escanear' },
    { key: 'acceso', value: remote.access ? 'sesión abierta' : 'sin sesión' },
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

      <SiteFrame ip={remote.ip} html={remote.web} name={remote.name} down={remote.down} />

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
