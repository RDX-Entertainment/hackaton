import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { StatusMessage } from '../components/StatusMessage';
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

  const load = useCallback(() => {
    processesApi.listProcesses().then(setTasks).catch(() => setTasks([]));
  }, []);

  useEffect(() => {
    load();
    processesApi.getProcessActions().then(setActions).catch(() => setActions([]));
  }, [load]);

  const start = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await processesApi.startProcess(
        Number(action),
        victim ? Number(victim) : undefined,
        undefined,
        info,
      );
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
      </form>

      {tasks.map((task) => {
        const id = Number(task.id);

        return (
          <article className="forum-post" key={id}>
            <b>
              Task #{id} · {formatCell(task.details || task.action)}
            </b>
            <p>
              {formatCell(task.worked_seconds)}/{formatCell(task.ideal_seconds)} seconds ·{' '}
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

            {!task.completed && (
              <>
                {task.paused ? (
                  <button onClick={() => void command(id, 'resume')}>Resume</button>
                ) : (
                  <button onClick={() => void command(id, 'pause')}>Pause</button>
                )}
                <button onClick={() => void command(id, 'delete')}>Delete</button>
              </>
            )}
          </article>
        );
      })}
    </>
  );
}
