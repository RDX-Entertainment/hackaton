import { tr, useIdioma } from '../../i18n/text';
import type { HackKind, Remote, RemoteFile } from '../../utils/api';
import { formatIp } from '../../utils/ip';
import { AccessTab } from './AccessTab';
import { ForceTab } from './ForceTab';
import { PublicTab } from './PublicTab';
import { RemoteFileList } from './RemoteFileList';

export type RemoteTab = 'public' | 'access' | 'files' | 'force';

const TABS: { key: RemoteTab; label: string; hint: string }[] = [
  { key: 'public', label: 'public', get hint() { return tr("lo que enseña a cualquiera"); } },
  { key: 'access', get label() { return tr("acceso"); }, get hint() { return tr("ssh: entrar y lo que hay dentro"); } },
  { key: 'files', label: 'files', get hint() { return tr("software y ficheros de la máquina"); } },
  { key: 'force', label: 'force', get hint() { return tr("romperla por la fuerza"); } },
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
  useIdioma();
  return (
    <section className="viewport">
      <header className="viewport__head">
        <span className={remote.down ? 'viewport__dot is-down' : 'viewport__dot'} aria-hidden="true" />

        <span className="viewport__title">
          {remote.name || tr("máquina sin nombre")}
          <small>{formatIp(remote.ip)}</small>
        </span>

        <nav className="viewport__tabs" aria-label={tr("Secciones de la máquina")}>
          {TABS.filter((one) => one.key !== 'files' || remote.access).map((one) => (
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
            log={log}
            onSignIn={onSignIn}
            onRewriteLog={onRewriteLog}
            onWipeLog={onWipeLog}
          />
        )}

        {tab === 'files' && (
          <div className="page">
            {remote.access ? (
              <section className="page__block page__block--first">
                <h4>{tr("Software encontrado")}</h4>
                <RemoteFileList files={files} onDownload={onDownload} onDelete={onDelete} />
              </section>
            ) : (
              <p className="page__note">{tr("Necesitas acceso a la máquina para inspeccionar sus ficheros.")}</p>
            )}
          </div>
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
