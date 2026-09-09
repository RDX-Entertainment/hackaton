import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { DataTable } from '../components/DataTable';
import { EmptyNotice } from '../components/Notice';
import { CircuitRail } from '../components/onboarding/CircuitRail';
import { useOnboarding } from '../components/onboarding/useOnboarding';
import { ProcessProgress } from '../components/ProcessProgress';
import { StatusMessage } from '../components/StatusMessage';
import {
  researchApi,
  softwareApi,
  type OwnedFile,
  type UnknownRow,
} from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

/**
 * Investigar sube la versión de un programa que ya tienes.
 *
 * La pantalla pedía **«Owned software ID»** en una caja de texto: para usarla había que
 * saberse de memoria el número de un fichero que no se enseña en ninguna parte. Ahora se
 * elige de una lista, que es el mismo dato dicho de una forma que se puede contestar.
 */
export function ResearchPage() {
  const status = useStatusMessage();
  const { onboarding } = useOnboarding();

  const [rows, setRows] = useState<UnknownRow[]>([]);
  const [ficheros, setFicheros] = useState<OwnedFile[]>([]);
  const [softwareId, setSoftwareId] = useState('');
  const [processId, setProcessId] = useState<number>();

  const load = useCallback(() => {
    researchApi.listResearch().then(setRows).catch(() => setRows([]));

    softwareApi
      .listSoftware()
      .then((todos) => {
        const investigables = todos.filter((fichero) => !fichero.folder);
        setFicheros(investigables);
        setSoftwareId((actual) => actual || String(investigables[0]?.id ?? ''));
      })
      .catch(() => setFicheros([]));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const start = async (event: FormEvent) => {
    event.preventDefault();

    try {
      const queued = await researchApi.startResearch(Number(softwareId));
      setProcessId(queued.process_id);
      status.report(
        `Investigación encolada como tarea #${queued.process_id} (${queued.seconds} s). ` +
          'La versión sube cuando termine.',
      );
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Investigación</h2>

      <CircuitRail onboarding={onboarding} />

      <p className="muted">
        Investigar sube la versión de un programa que ya tienes. Tarda, y hasta que no
        termina la versión sigue siendo la de antes.
      </p>

      {ficheros.length ? (
        <form className="compose" onSubmit={start}>
          <label>
            Programa
            <select value={softwareId} onChange={(event) => setSoftwareId(event.target.value)}>
              {ficheros.map((fichero) => (
                <option key={fichero.id} value={fichero.id}>
                  {fichero.name} v{fichero.version}
                  {fichero.running ? ' · en memoria' : ' · en disco'}
                </option>
              ))}
            </select>
          </label>

          <button>Investigar la siguiente versión</button>
          <StatusMessage text={status.message} />
          {processId && <ProcessProgress processId={processId} onFinished={load} />}
        </form>
      ) : (
        <EmptyNotice>
          No tienes ningún programa que investigar. Compila uno en Arsenal y vuelve.
        </EmptyNotice>
      )}

      <h3>Lo investigado</h3>
      <DataTable rows={rows} />
    </>
  );
}
