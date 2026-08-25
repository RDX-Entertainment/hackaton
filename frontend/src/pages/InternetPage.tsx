import { useCallback, useEffect, useMemo, useState } from 'react';

import { AddressBar, suggestionsFrom } from '../components/internet/AddressBar';
import { BookmarkBar } from '../components/internet/BookmarkBar';
import { Directory } from '../components/internet/Directory';
import { RemotePage, type RemoteTab } from '../components/internet/RemotePage';
import { StartPage } from '../components/internet/StartPage';
import { StatusMessage } from '../components/StatusMessage';
import {
  internetApi,
  processesApi,
  type HackKind,
  type Remote,
  type RemoteFile,
  type Target,
  type Visited,
} from '../utils/api';
import { isNotFound } from '../utils/errors';
import { formatIp, parseIp } from '../utils/ip';
import { useBrowserHistory } from '../utils/useBrowserHistory';
import { useStatusMessage } from '../utils/useStatusMessage';

/** El número de acción del legado para entrar por un exploit de FTP o SSH. */
const EXPLOIT_ACTION = 16;

/** Todo lo que se encola responde con un id o con el otro, nunca con los dos. */
type Queued = { process_id?: number; id?: number };

export function InternetPage() {
  const status = useStatusMessage();
  const trail = useBrowserHistory();

  const [address, setAddress] = useState('');
  const [remote, setRemote] = useState<Remote>();
  const [tab, setTab] = useState<RemoteTab>('public');
  const [files, setFiles] = useState<RemoteFile[]>([]);
  const [log, setLog] = useState('');
  const [targets, setTargets] = useState<Target[]>([]);
  const [history, setHistory] = useState<Visited[]>([]);
  const [bookmarks, setBookmarks] = useState<Visited[]>([]);
  const [loading, setLoading] = useState(false);
  const [notFound, setNotFound] = useState<number>();

  const loadSides = useCallback(() => {
    internetApi.listTargets().then(setTargets).catch(() => {});
    internetApi.listHistory().then(setHistory).catch(() => {});
    internetApi.listBookmarks().then(setBookmarks).catch(() => {});
  }, []);

  useEffect(() => {
    loadSides();
  }, [loadSides]);

  /** Todo lo que hay pasado el login solo se lee con acceso, así que carga junto. */
  const loadInsides = useCallback(async (ip: number) => {
    try {
      setFiles(await internetApi.listFiles(ip));
    } catch {
      setFiles([]);
    }

    try {
      const remoteLog = await internetApi.readLog(ip);
      setLog(remoteLog.text);
    } catch {
      setLog('');
    }
  }, []);

  /**
   * Pedir una máquina. `record` distingue ir a un sitio de volver a uno: atrás no
   * puede empujar una entrada nueva o no habría manera de salir de la pila.
   */
  const open = useCallback(
    async (ip: number, record = true) => {
      setAddress(formatIp(ip));
      setFiles([]);
      setLog('');
      setLoading(true);
      setNotFound(undefined);
      status.clear();

      try {
        const machine = await internetApi.navigate(ip);
        setRemote(machine);

        // Con sesión abierta lo que interesa está dentro; sin ella, la cara pública.
        setTab(machine.access ? 'access' : 'public');

        if (machine.access) {
          await loadInsides(ip);
        }

        if (record) {
          trail.push(ip);
        }
      } catch (error) {
        setRemote(undefined);

        // Que no haya nadie en esa IP lo cuenta la portada con su 404: repetirlo arriba en
        // rojo seria decir dos veces lo mismo. Los demas fallos si necesitan el aviso.
        if (isNotFound(error)) {
          setNotFound(ip);
        } else {
          status.reportError(error);
        }
      } finally {
        setLoading(false);
      }

      loadSides();
    },
    // `trail` cambia en cada render y volvería a crear esta función sin parar.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [loadInsides, loadSides, status],
  );

  const step = (ip: number | null) => {
    if (ip !== null) {
      void open(ip, false);
    }
  };

  /** Avisa con el id que haya usado el servidor para la tarea. */
  const announce = async (work: Promise<Queued>, label: string) => {
    try {
      const queued = await work;
      const id = queued.process_id ?? queued.id;
      status.report(`${label} encolado como tarea #${id}. Ocurre cuando pase el worker.`);
    } catch (error) {
      status.reportError(error);
    }
  };

  const signIn = async (user: string, password: string): Promise<boolean> => {
    if (!remote) {
      return false;
    }

    try {
      await internetApi.login(remote.ip, user, password);
      status.report('Acceso concedido.');
      await open(remote.ip, false);
      return true;
    } catch (error) {
      status.reportError(error);
      return false;
    }
  };

  const crack = async (kind: HackKind) => {
    if (!remote) {
      return;
    }

    try {
      const result = await internetApi.hack(remote.ip, kind);
      status.report(
        `Tarea #${result.process_id} · tu v${result.attacker_version} contra su ` +
          `v${result.defender_version}. Pasa el worker y vuelve a conectar.`,
      );
    } catch (error) {
      status.reportError(error);
    }
  };

  const toggleBookmark = async () => {
    if (!remote) {
      return;
    }

    try {
      if (remote.bookmarked) {
        await internetApi.removeBookmark(remote.ip);
      } else {
        await internetApi.addBookmark(remote.ip);
      }

      await open(remote.ip, false);
    } catch (error) {
      status.reportError(error);
    }
  };

  const dropBookmark = async (ip: number) => {
    try {
      await internetApi.removeBookmark(ip);

      if (remote?.ip === ip) {
        await open(ip, false);
      } else {
        loadSides();
      }
    } catch (error) {
      status.reportError(error);
    }
  };

  const known = useMemo(
    () => suggestionsFrom(bookmarks, history, targets),
    [bookmarks, history, targets],
  );

  return (
    <>
      <h2>Internet</h2>

      <div className="net">
        <div className="browser">
          <AddressBar
            address={address}
            known={known}
            loading={loading}
            canBack={trail.canBack}
            canForward={trail.canForward}
            onAddressChange={setAddress}
            onGo={(ip) => void open(ip)}
            onBack={() => step(trail.back())}
            onForward={() => step(trail.forward())}
            onReload={() => {
              const wanted = remote?.ip ?? parseIp(address);

              if (wanted !== null && wanted !== undefined) {
                void open(wanted, false);
              }
            }}
          />

          <BookmarkBar
            bookmarks={bookmarks}
            current={remote?.ip ?? null}
            onVisit={(ip) => void open(ip)}
            onRemove={(ip) => void dropBookmark(ip)}
          />

          <StatusMessage text={status.message} as="block" />

          {remote ? (
            <RemotePage
              remote={remote}
              tab={tab}
              files={files}
              log={log}
              busy={loading}
              onTab={setTab}
              onSignIn={signIn}
              onCrack={(kind) => void crack(kind)}
              onExploit={() =>
                void announce(
                  processesApi.startProcess(EXPLOIT_ACTION, remote.id, undefined, remote.kind),
                  'Exploit',
                )
              }
              onAnalyze={() => void announce(internetApi.analyze(remote.ip), 'Análisis')}
              onToggleBookmark={() => void toggleBookmark()}
              onVisit={(target) => void open(target)}
              onDownload={(file) =>
                void announce(internetApi.transferFile(remote.ip, file.id, 'download'), 'Descarga')
              }
              onDelete={(file) =>
                void announce(internetApi.removeFile(remote.ip, file.id), 'Borrado')
              }
              onRewriteLog={(text) =>
                void announce(internetApi.editLog(remote.ip, text), 'Reescritura del registro')
              }
              onWipeLog={() =>
                void announce(internetApi.editLog(remote.ip, ''), 'Borrado del registro')
              }
            />
          ) : (
            <StartPage history={history} notFound={notFound} onVisit={(ip) => void open(ip)} />
          )}
        </div>

        <Directory
          targets={targets}
          current={remote?.ip ?? null}
          onVisit={(ip) => void open(ip)}
        />
      </div>
    </>
  );
}
