import type { HackKind, Remote, RemoteFile } from '../../utils/api';
import { formatIp } from '../../utils/ip';
import { AccessTab } from './AccessTab';
import { ForceTab } from './ForceTab';
import { PublicTab } from './PublicTab';

export type RemoteTab = 'public' | 'access' | 'force';

const TABS: { key: RemoteTab; label: string; hint: string }[] = [
  { key: 'public', label: 'public', hint: 'lo que enseña a cualquiera' },
  { key: 'access', label: 'acceso', hint: 'ssh: entrar y lo que hay dentro' },
  { key: 'force', label: 'force', hint: 'romperla por la fuerza' },
];

type RemotePageProps = {
  remote: Remote;
  tab: RemoteTab;
  files: RemoteFile[];
  log: string;
  busy: boolean;
  onTab: (tab: RemoteTab) => void;
  onSignIn: (user: string, password: string) => Promise<boolean>;
  onCrack: (kind: HackKind) => void;
  onExploit: () => void;
  onAnalyze: () => void;
  onToggleBookmark: () => void;
  onDownload: (file: RemoteFile) => void;
  onDelete: (file: RemoteFile) => void;
  onRewriteLog: (text: string) => void;
  onWipeLog: () => void;
  /** Ir a otra máquina: lo piden los enlaces que declara un sitio. */
  onVisit: (ip: number) => void;
};

export function RemotePage({
  remote,
  tab,
  files,
  log,
  busy,
  onTab,
  onSignIn,
  onCrack,
  onExploit,
  onAnalyze,
  onToggleBookmark,
  onDownload,
  onDelete,
  onRewriteLog,
  onWipeLog,
  onVisit,
}: RemotePageProps) {
  return (
    <section className="viewport">
      <header className="viewport__head">
        <span className={remote.down ? 'viewport__dot is-down' : 'viewport__dot'} aria-hidden="true" />

        <span className="viewport__title">
          {remote.name || 'máquina sin nombre'}
          <small>{formatIp(remote.ip)}</small>
        </span>

        <nav className="viewport__tabs" aria-label="Secciones de la máquina">
          {TABS.map((one) => (
            <button
              type="button"
              key={one.key}
              className={one.key === tab ? `vtab vtab--${one.key} is-active` : `vtab vtab--${one.key}`}
              title={one.hint}
              aria-current={one.key === tab}
              onClick={() => onTab(one.key)}
            >
              {one.label}
              {one.key === 'access' && remote.access && (
                <span className="vtab__flag" aria-hidden="true">
                  ●
                </span>
              )}
            </button>
          ))}
        </nav>
      </header>

      <div className="viewport__body">
        {tab === 'public' && (
          <PublicTab remote={remote} onToggleBookmark={onToggleBookmark} onVisit={onVisit} />
        )}

        {tab === 'access' && (
          <AccessTab
            remote={remote}
            files={files}
            log={log}
            onSignIn={onSignIn}
            onDownload={onDownload}
            onDelete={onDelete}
            onRewriteLog={onRewriteLog}
            onWipeLog={onWipeLog}
          />
        )}

        {tab === 'force' && (
          <ForceTab
            remote={remote}
            busy={busy}
            onCrack={onCrack}
            onExploit={onExploit}
            onAnalyze={onAnalyze}
          />
        )}
      </div>
    </section>
  );
}
