import { useEffect, useRef, useState } from 'react';

import { processesApi, type ProcessSnapshot } from '../utils/api';

type ProcessProgressProps = {
  processId: number;
  onFinished?: (process: ProcessSnapshot) => void;
};

/** Una sola tarea se consulta por id cada segundo; no hace falta mantener un WebSocket. */
export function ProcessProgress({ processId, onFinished }: ProcessProgressProps) {
  const [process, setProcess] = useState<ProcessSnapshot>();
  const [sampledAt, setSampledAt] = useState<number>();
  const [now, setNow] = useState(() => Date.now());
  const onFinishedRef = useRef(onFinished);
  const finishedRef = useRef(false);

  useEffect(() => {
    onFinishedRef.current = onFinished;
  }, [onFinished]);

  useEffect(() => {
    let alive = true;
    finishedRef.current = false;

    const load = async () => {
      try {
        const current = await processesApi.getProcess(processId);
        if (!alive) return;

        setProcess(current);
        setSampledAt(Date.now());
        if (current.completed && !finishedRef.current) {
          finishedRef.current = true;
          onFinishedRef.current?.(current);
        }
      } catch {
        // La pantalla ya muestra el error de lanzamiento; un fallo momentaneo de sondeo no lo
        // debe reemplazar cada segundo.
      }
    };

    void load();
    const timer = window.setInterval(() => void load(), 1_000);
    return () => {
      alive = false;
      window.clearInterval(timer);
    };
  }, [processId]);

  // El servidor confirma el estado una vez por segundo. Entre dos muestras, el reloj local
  // anticipa el siguiente avance para que la barra no vaya a saltos.
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 50);
    return () => window.clearInterval(timer);
  }, []);

  const ideal = process?.ideal_seconds ?? 0;
  const reportedWorked = process?.completed
    ? ideal
    : Math.max(0, ideal - (process?.seconds_left ?? ideal));
  const elapsedSinceSample = process && !process.completed && !process.paused && sampledAt
    ? Math.min(1.1, Math.max(0, (now - sampledAt) / 1_000))
    : 0;
  const worked = Math.min(ideal, reportedWorked + elapsedSinceSample);
  const percentage = process?.completed
    ? 100
    : ideal > 0
      ? Math.min(100, (worked / ideal) * 100)
      : 0;
  const state = process?.completed
    ? process.status === 'succeeded' ? 'completado' : process.status
    : process?.paused ? 'en pausa' : 'en curso';

  return (
    <div className="process-progress" aria-live="polite">
      <p>
        <b>Tarea #{processId}</b> · {state}
        {ideal > 0 && ` · ${worked}/${ideal} s`}
      </p>
      <div
        className="process-progress__track"
        role="progressbar"
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(percentage)}
      >
        <span className="process-progress__fill" style={{ width: `${percentage}%` }} />
      </div>
      {process?.completed && process.result_message && <small>{process.result_message}</small>}
    </div>
  );
}
