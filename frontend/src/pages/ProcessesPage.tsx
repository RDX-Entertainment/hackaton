import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { StatusMessage } from '../components/StatusMessage';
import { ProcessProgress } from '../components/ProcessProgress';
import {
  processesApi,
  type ProcessAction,
  type ProcessCommand,
  type UnknownRow,
} from '../utils/api';
import { formatCell } from '../utils/format';
import { useStatusMessage } from '../utils/useStatusMessage';

const MAX_INFO_LENGTH = 30;

export function ProcessesPage() {
  const status = useStatusMessage();

  const [tasks, setTasks] = useState<UnknownRow[]>([]);
  const [actions, setActions] = useState<ProcessAction[]>([]);
  const [action, setAction] = useState('1');
  const [victim, setVictim] = useState('');
  const [info, setInfo] = useState('');
  const [startedId, setStartedId] = useState<number>();

  const load = useCallback(() => {
    processesApi.listProcesses().then(setTasks).catch(() => setTasks([]));
  }, []);

  useEffect(() => {
    load();
    processesApi.getProcessActions().then(setActions).catch(() => setActions([]));
  }, [load]);

  // El worker actualiza la cola fuera de React; recargar evita una pantalla congelada entre
  // que una tarea vence y el worker deja su resultado.
  useEffect(() => {
    const timer = window.setInterval(load, 5_000);
    return () => window.clearInterval(timer);
  }, [load]);

  const start = async (event: FormEvent) => {
    event.preventDefault();

    try {
      const started = await processesApi.startProcess(
        Number(action),
        victim ? Number(victim) : undefined,
        undefined,
        info,
      );
      setStartedId(started.id);
      setInfo('');
      status.report('Task started. Run the worker to advance queued tasks.');
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  const command = async (id: number, name: ProcessCommand) => {
    try {
      await processesApi.commandProcess(id, name);
      status.report(`Task ${name}d.`);
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Processes</h2>

      <form className="compose" onSubmit={start}>
        <label>
          Action
          <select value={action} onChange={(event) => setAction(event.target.value)}>
            {actions.map((available) => (
              <option key={available.id} value={available.id}>
                {available.name} ({available.id})
              </option>
            ))}
          </select>
        </label>

        <label>
          Victim ID (optional)
          <input
            value={victim}
            inputMode="numeric"
            onChange={(event) => setVictim(event.target.value)}
          />
        </label>

        <label>
          Info
          <input
            value={info}
            onChange={(event) => setInfo(event.target.value)}
            maxLength={MAX_INFO_LENGTH}
          />
        </label>

        <button>Start task</button>
        <StatusMessage text={status.message} />
        {startedId && <ProcessProgress processId={startedId} onFinished={load} />}
      </form>

      <section className="processes" aria-label="Procesos actuales">
        <h3>En curso</h3>
        {tasks.filter((task) => !task.completed).length === 0 && (
          <p className="muted">No hay procesos activos.</p>
        )}
        {tasks.filter((task) => !task.completed).map((task) => {
          const id = Number(task.id);
          const actionId = Number(task.action);
          const actionName = actionId === 91
            ? 'Compilar software'
            : actions.find((available) => available.id === actionId)?.name ?? formatCell(task.details || task.action);

          return (
            <article className="forum-post" key={id}>
              <b>Tarea #{id} · {actionName}</b>
              <ProcessProgress processId={id} onFinished={load} />
              {task.paused ? (
                <button onClick={() => void command(id, 'resume')}>Resume</button>
              ) : (
                <button onClick={() => void command(id, 'pause')}>Pause</button>
              )}
              <button onClick={() => void command(id, 'delete')}>Delete</button>
            </article>
          );
        })}
      </section>

      <details className="processes__completed">
        <summary>Procesos terminados ({tasks.filter((task) => task.completed).length})</summary>
        <div className="processes__completed-list">
          {tasks.filter((task) => task.completed).map((task) => {
        const id = Number(task.id);
        const actionId = Number(task.action);
        const actionName = actionId === 91
          ? 'Compilar software'
          : actions.find((available) => available.id === actionId)?.name ?? formatCell(task.details || task.action);
        const secondsLeft = Math.max(0, Number(task.seconds_left ?? 0));
        const idealSeconds = Number(task.ideal_seconds ?? 0);
        const workedSeconds = task.completed
          ? Number(task.worked_seconds ?? idealSeconds)
          : Math.max(0, idealSeconds - secondsLeft);

        return (
          <article className="forum-post" key={id}>
            <b>
              Tarea #{id} · {actionName}
            </b>
            <p>
              {workedSeconds}/{idealSeconds} segundos
              {!task.completed && !task.paused && ` · faltan ${secondsLeft} s`} ·{' '}
              {/*
                A finished task now says how it finished. `status` is only worth reading once
                it is over: a legacy action is inserted as `queued` and never updated until
                the worker resolves it, so before that the pause flag is the honest answer.
              */}
              {task.completed
                ? formatCell(task.status) || 'completed'
                : task.paused
                  ? 'paused'
                  : 'running'}
            </p>

            {/* Only a modular operation carries a reason; every other action leaves it empty. */}
            {Boolean(task.result_message) && (
              <p className={task.status === 'failed' ? 'error' : 'muted'}>
                {formatCell(task.result_code)} — {formatCell(task.result_message)}
              </p>
            )}

          </article>
        );
          })}
        </div>
      </details>
    </>
  );
}
