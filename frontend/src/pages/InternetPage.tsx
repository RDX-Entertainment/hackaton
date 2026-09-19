import { tr, useIdioma, idiomaActual } from '../i18n/text';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useLocation } from 'react-router-dom';
import { useTutorial } from '../app/tutorial';

import { AddressBar, suggestionsFrom } from '../components/internet/AddressBar';
import { BookmarkBar } from '../components/internet/BookmarkBar';
import { Directory } from '../components/internet/Directory';
import { RemotePage, type RemoteTab } from '../components/internet/RemotePage';
import { StartPage } from '../components/internet/StartPage';
import { StatusMessage } from '../components/StatusMessage';
import { ProcessProgress } from '../components/ProcessProgress';
import {
  internetApi,
  arsenalApi,
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

/** Todo lo que se encola responde con un id o con el otro, nunca con los dos. */
type Queued = { process_id?: number; id?: number };

export function InternetPage() {
  useIdioma();
  const status = useStatusMessage();
  const trail = useBrowserHistory();
  const location = useLocation();
  const tutorial = useTutorial();
  const practiceOpened = useRef('');
  const navigation = useRef(0);
  const currentIp = useRef<number | undefined>(undefined);
  const operationTarget = useRef<{ ip: number; tab: RemoteTab } | undefined>(undefined);

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
  const [processId, setProcessId] = useState<number>();
  const [accessProcessId, setAccessProcessId] = useState<number>();
  const [submitting, setSubmitting] = useState(false);

  const loadSides = useCallback(() => {
    internetApi.listTargets().then(setTargets).catch(() => {});
    internetApi.listHistory().then(setHistory).catch(() => {});
    internetApi.listBookmarks().then(setBookmarks).catch(() => {});
  }, []);

  useEffect(() => {
    loadSides();
  }, [loadSides]);

  /** Todo lo que hay pasado el login solo se lee con acceso, así que carga junto. */
  const loadInsides = useCallback(async (machine: Remote, request: number) => {
    const [items, remoteLog] = await Promise.all([
      machine.permissions.includes('read_files') ? internetApi.listFiles(machine.ip).catch(() => []) : Promise.resolve([]),
      machine.permissions.includes('read_logs') ? internetApi.readLog(machine.ip).catch(() => ({ text: '' })) : Promise.resolve({ text: '' }),
    ]);
    if (request !== navigation.current) return;
    setFiles(items); setLog(remoteLog.text);
  }, []);

  /**
   * Pedir una máquina. `record` distingue ir a un sitio de volver a uno: atrás no
   * puede empujar una entrada nueva o no habría manera de salir de la pila.
   */
  const open = useCallback(
    async (ip: number, record = true) => {
      const request = ++navigation.current;
      currentIp.current = ip;
      setAddress(formatIp(ip));
      setFiles([]);
      setLog('');
      setLoading(true);
      setNotFound(undefined);
      status.clear();

      try {
        const machine = await internetApi.navigate(ip);
        if (request !== navigation.current) return;
        setRemote(machine);

        // Con sesión abierta lo que interesa está dentro; sin ella, la cara pública.
        setTab(machine.access ? 'force' : 'public');

        if (machine.access) {
          await loadInsides(machine, request);
        }

        if (record) {
          trail.push(ip);
        }

        return machine;
      } catch (error) {
        if (request !== navigation.current) return;
        setRemote(undefined);

        // Que no haya nadie en esa IP lo cuenta la portada con su 404: repetirlo arriba en
        // rojo seria decir dos veces lo mismo. Los demas fallos si necesitan el aviso.
        if (isNotFound(error)) {
          setNotFound(ip);
        } else {
          status.reportError(error);
        }
      } finally {
        if (request === navigation.current) setLoading(false);
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

  useEffect(() => {
    const params = new URLSearchParams(location.search);
    const direct = params.get('ip');
    if (direct && practiceOpened.current !== location.key) {
      const ip = Number(direct);
      if (Number.isInteger(ip) && ip > 0 && ip <= 4294967295) {
        practiceOpened.current = location.key; void open(ip);
      }
      return;
    }
    const wanted = params.get('practice');
    if (!wanted || practiceOpened.current === location.key) return;
    const target = tutorial.onboarding?.practice.find((one) => String(one.ip) === wanted);
    if (!target) return;
    practiceOpened.current = location.key;
    void open(target.ip).then((machine) => {
      if (machine && !machine.access) setTab('force');
    });
  }, [location.key, location.search, tutorial.onboarding?.practice, open]);

  /** Avisa con el id que haya usado el servidor para la tarea. */
  const announce = async (work: Promise<Queued>, label: string, seeksAccess = false, resultTab: RemoteTab = tab) => {
    const ip = remote?.ip;
    setSubmitting(true);
    try {
      const queued = await work;
      const id = queued.process_id ?? queued.id;
      if (id) {
        setProcessId(id);
        setAccessProcessId(id);
        if (ip !== undefined) operationTarget.current = { ip, tab: seeksAccess ? 'force' : resultTab };
      }
      status.report(tr("{0} encolado como tarea #{1}. Ocurre cuando pase el worker.", label, id));
    } catch (error) {
      status.reportError(error);
    } finally {
      setSubmitting(false);
    }
  };

  const signIn = async (user: string, password: string): Promise<boolean> => {
    if (!remote) {
      return false;
    }

    try {
      await internetApi.login(remote.ip, user, password);
      status.report(tr("Acceso concedido."));
      await open(remote.ip, false);
      return true;
    } catch (error) {
      status.reportError(error);
      return false;
    }
  };

  const publish = async (content: string) => {
    if (!remote) return;
    const ip = remote.ip;
    const request = navigation.current;
    setSubmitting(true);
    try {
      const saved = await arsenalApi.modifyWeb(ip, content);
      if (currentIp.current !== ip || navigation.current !== request) return;
      // Recargar también Webscript: no basta con sustituir el HTML de la vista previa.
      const refreshed = await open(ip, false);
      if (currentIp.current !== ip || navigation.current !== request + 1) return;
      if (refreshed) {
        setTab('public');
        status.report(tr("HTML publicado en {0} · revisión {1}.", formatIp(ip), saved.revision));
      } else {
        status.report(tr("HTML guardado. No se pudo recargar el sitio; vuelve a abrir la IP."));
      }
    } finally {
      setSubmitting(false);
    }
  };

  const crack = async (kind: HackKind, softwareId?: number) => {
    if (!remote) {
      return;
    }

    setSubmitting(true);
    try {
      const result = await internetApi.hack(remote.ip, kind, softwareId);
      setProcessId(result.process_id);
      setAccessProcessId(result.process_id);
      operationTarget.current = { ip: remote.ip, tab: 'force' };
      status.report(
        result.attacker_stats
        ? tr("{0} #{1} · eficacia {2}{3} · sigilo {4} · probabilidad {5} · {6} s.", kind === 'scan' ? tr("Escaneo") : tr("Fuerza bruta"), result.process_id, result.attacker_stats.effectiveness, kind === 'scan' ? '' : tr(" · penetración {0}", result.attacker_stats.penetration), result.attacker_stats.stealth, result.estimated_chance, result.seconds)
        : tr("Tarea #{0} · tu v{1} contra su ", result.process_id, result.attacker_version) +
          tr("v{0}. Pasa el worker y vuelve a conectar.", result.defender_version),
      );
    } catch (error) {
      status.reportError(error);
    } finally {
      setSubmitting(false);
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
    [bookmarks, history, targets, idiomaActual()],
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
          {processId && (
            <ProcessProgress
              processId={processId}
              onFinished={(process) => {
                tutorial.recargar();
                setAccessProcessId(undefined);
                if (process.status !== 'succeeded') {
                  status.report(process.result_message || tr("La operación no consiguió su objetivo."));
                  return;
                }
                const target = operationTarget.current;
                if (!target || currentIp.current !== target.ip) return;
                void (async () => {
                  const refreshed = await open(target.ip, false);
                  if (!refreshed || currentIp.current !== target.ip) return;
                  setTab(target.tab);
                  status.report(process.result_message || (process.status === 'succeeded' ? tr("Operación completada. Acciones y permisos actualizados.") : tr("La operación no consiguió su objetivo.")));
                })();
              }}
            />
          )}

          {remote ? (
            <RemotePage
              remote={remote}
              tab={tab}
              files={files}
              log={log}
              busy={loading || submitting || Boolean(accessProcessId)}
              onTab={setTab}
              onSignIn={signIn}
              onCrack={(kind, softwareId) => void crack(kind, softwareId)}
              onIntrude={(capability, softwareId) => void announce(internetApi.intrude(remote.ip, capability, softwareId), tr("Intrusión"), true)}
              onPublish={publish}
              onUpload={(id) => void announce(internetApi.transferFile(remote.ip, id, 'upload'), tr("Subida"))}
              onAnalyze={() => void announce(internetApi.analyze(remote.ip), tr("Análisis"))}
              onToggleBookmark={() => void toggleBookmark()}
              onVisit={(target) => void open(target)}
              onDownload={(file) =>
                void announce(internetApi.transferFile(remote.ip, file.id, 'download'), tr("Descarga"))
              }
              onDelete={(file) =>
                void announce(internetApi.removeFile(remote.ip, file.id), tr("Borrado"))
              }
              onRewriteLog={(text) =>
                void announce(internetApi.editLog(remote.ip, text), tr("Reescritura del registro"))
              }
              onWipeLog={() =>
                void announce(internetApi.editLog(remote.ip, ''), tr("Borrado del registro"))
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
