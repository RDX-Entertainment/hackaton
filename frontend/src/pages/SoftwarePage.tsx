import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { EmptyNotice } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import {
  processesApi,
  researchApi,
  softwareApi,
  type OwnedFile,
  type SoftwareType,
} from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

/**
 * Every verb on this screen is a legacy process action, so the buttons queue
 * work rather than doing it.
 */
const ACTION = {
  hide: 4,
  seek: 5,
  install: 13,
  uninstall: 14,
  park: 18,
} as const;

const MAX_NAME_LENGTH = 25;

export function SoftwarePage() {
  const status = useStatusMessage();

  const [files, setFiles] = useState<OwnedFile[]>([]);
  const [types, setTypes] = useState<SoftwareType[]>([]);
  const [name, setName] = useState('');
  const [type, setType] = useState('1');
  const [size, setSize] = useState('30');
  const [ram, setRam] = useState('50');

  const load = useCallback(() => {
    softwareApi.listSoftware().then(setFiles).catch(() => {});
  }, []);

  useEffect(() => {
    load();
    softwareApi.listSoftwareTypes().then(setTypes).catch(() => {});
  }, [load]);

  const queue = async (action: number, softwareId: number, label: string) => {
    try {
      await processesApi.startProcess(action, undefined, softwareId);
      status.report(`${label} queued. Run the worker to advance it.`);
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  const research = async (softwareId: number) => {
    try {
      const queued = await researchApi.startResearch(softwareId);
      status.report(`Research queued as task #${queued.process_id}.`);
    } catch (error) {
      status.reportError(error);
    }
  };

  const write = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await softwareApi.createSoftware(name, Number(type), Number(size), Number(ram));
      setName('');
      status.report('Software written.');
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Software</h2>

      <form className="compose" onSubmit={write}>
        <label>
          Name
          <input
            value={name}
            maxLength={MAX_NAME_LENGTH}
            onChange={(event) => setName(event.target.value)}
            required
          />
        </label>

        <label>
          Type
          <select value={type} onChange={(event) => setType(event.target.value)}>
            {types.map((softwareType) => (
              <option key={softwareType.id} value={softwareType.id}>
                {softwareType.name}
              </option>
            ))}
          </select>
        </label>

        <label>
          Size (MB)
          <input
            value={size}
            inputMode="numeric"
            onChange={(event) => setSize(event.target.value)}
            required
          />
        </label>

        <label>
          RAM
          <input
            value={ram}
            inputMode="numeric"
            onChange={(event) => setRam(event.target.value)}
            required
          />
        </label>

        <button>Write it</button>
        <StatusMessage text={status.message} />
      </form>

      {files.map((file) => (
        <article className="forum-post" key={file.id}>
          <b>
            {file.name} v{file.version}
          </b>
          <small>
            {' '}
            · {file.size} MB · {file.ram} RAM · {file.running ? 'running' : 'not running'}
            {file.hidden && ` · hidden with v${file.hidden_with}`}
          </small>

          {file.running ? (
            <button onClick={() => void queue(ACTION.uninstall, file.id, 'Uninstall')}>
              Uninstall
            </button>
          ) : (
            <button onClick={() => void queue(ACTION.install, file.id, 'Install')}>
              Install
            </button>
          )}

          {file.hidden ? (
            <button onClick={() => void queue(ACTION.seek, file.id, 'Seek')}>Seek</button>
          ) : (
            <button onClick={() => void queue(ACTION.hide, file.id, 'Hide')}>Hide</button>
          )}

          <button onClick={() => void research(file.id)}>Research</button>
          <button onClick={() => void queue(ACTION.park, file.id, 'Park on external drive')}>
            Park
          </button>
        </article>
      ))}

      {!files.length && (
        <EmptyNotice>Nothing on disk yet. Write something, or go take it.</EmptyNotice>
      )}
    </>
  );
}
