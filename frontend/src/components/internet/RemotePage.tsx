import { tr, useIdioma } from '../../i18n/text';
import type { HackKind, Remote, RemoteFile } from '../../utils/api';
import { formatIp } from '../../utils/ip';
import { AccessTab } from './AccessTab';
import { ServicePanel } from './ServicePanel';
import { WebEditor } from './WebEditor';
import { RemoteUpload } from './RemoteUpload';
import { PublicTab } from './PublicTab';
import { RemoteFileList } from './RemoteFileList';

export type RemoteTab = 'public' | 'access' | 'files' | 'force' | 'webedit';

const TABS: { key: RemoteTab; label: string; hint: string }[] = [
  { key: 'public', label: 'sitio', get hint() { return tr("lo que enseña a cualquiera"); } },
  { key: 'force', get label() { return tr("servicios"); }, get hint() { return tr("elige el punto de entrada"); } },
  { key: 'access', label: 'SSH', get hint() { return tr("entrar y revisar el registro"); } },
  { key: 'files', label: 'almacenamiento', get hint() { return tr("software y ficheros de la máquina"); } },
  { key: 'webedit', get label() { return tr("editar HTML"); }, get hint() { return tr("modificar el sitio de este nodo"); } },
];

type RemotePageProps = {
  remote: Remote;
  tab: RemoteTab;
  files: RemoteFile[];
  log: string;
  busy: boolean;
  onTab: (tab: RemoteTab) => void;
  onSignIn: (user: string, password: string) => Promise<boolean>;
  onCrack: (kind: HackKind, softwareId?: number) => void;
  onIntrude: (capability: string, softwareId: number) => void;
  onPublish: (content: string) => Promise<void>;
  onUpload: (id: number) => void;
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
  onIntrude,
  onPublish,
  onUpload,
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
          {TABS.filter((one) => (one.key !== 'files' || remote.permissions.includes('read_files')) && (one.key !== 'webedit' || remote.permissions.includes('read_web'))).map((one) => (
            <button
              type="button"
              key={one.key}
              data-guia={`red.${one.key}`}
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
            {remote.permissions.includes('read_files') ? (
              <section className="page__block page__block--first">
                <h4>{tr("Software encontrado")}</h4>
                {remote.permissions.includes('write_files') && <RemoteUpload busy={busy || remote.down} onUpload={onUpload} />}
                <RemoteFileList files={files} onDownload={onDownload} onDelete={onDelete} canDelete={remote.permissions.includes('write_files')} busy={busy || remote.down} />
              </section>
            ) : (
              <p className="page__note">{tr("Necesitas acceso a la máquina para inspeccionar sus ficheros.")}</p>
            )}
          </div>
        )}

        {tab === 'force' && (
          <ServicePanel
            remote={remote}
            busy={busy}
            onCrack={onCrack}
            onIntrude={onIntrude}
            onAnalyze={onAnalyze}
            onTab={onTab}
          />
        )}
        {tab === 'webedit' && remote.permissions.includes('read_web') && <WebEditor key={remote.ip} remote={remote} busy={busy} onPublish={onPublish} />}
      </div>
    </section>
  );
}
