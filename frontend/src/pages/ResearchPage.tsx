import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { DataTable } from '../components/DataTable';
import { StatusMessage } from '../components/StatusMessage';
import { researchApi, type UnknownRow } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

export function ResearchPage() {
  const status = useStatusMessage();

  const [rows, setRows] = useState<UnknownRow[]>([]);
  const [softwareId, setSoftwareId] = useState('');

  const load = useCallback(() => {
    researchApi.listResearch().then(setRows).catch(() => setRows([]));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const start = async (event: FormEvent) => {
    event.preventDefault();

    try {
      const queued = await researchApi.startResearch(Number(softwareId));
      setSoftwareId('');
      status.report(`Research queued as task #${queued.process_id}. Run the worker to apply it.`);
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Research</h2>

      <form className="compose" onSubmit={start}>
        <label>
          Owned software ID
          <input
            value={softwareId}
            inputMode="numeric"
            onChange={(event) => setSoftwareId(event.target.value)}
            required
          />
        </label>

        <button>Research next version</button>
        <StatusMessage text={status.message} />
      </form>

      <DataTable rows={rows} />
    </>
  );
}
