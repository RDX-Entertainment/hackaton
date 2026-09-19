import { tr, useIdioma } from '../i18n/text';
import { useEffect, useRef, useState } from 'react';

import { processesApi, type ProcessSnapshot } from '../utils/api';

type ProcessProgressProps = {
  processId: number;
  title?: string;
  initialSeconds?: number;
  onFinished?: (process: ProcessSnapshot) => void;
};

/** Una sola tarea se consulta por id cada segundo; no hace falta mantener un WebSocket. */
export function ProcessProgress({ processId, onFinished, title, initialSeconds = 0 }: ProcessProgressProps) {
  useIdioma();
  const [process, setProcess] = useState<ProcessSnapshot>();
  const [sampledAt, setSampledAt] = useState<number>();
  const [now, setNow] = useState(() => performance.now());
  const [visible, setVisible] = useState(() => !document.hidden);
  const onFinishedRef = useRef(onFinished);
  const finishedRef = useRef(false);

  useEffect(() => {
    onFinishedRef.current = onFinished;
  }, [onFinished]);

  useEffect(() => {
    let alive = true;
    let reading = false;
    let timer: number | undefined;
    finishedRef.current = false;
    setProcess(undefined);
    setSampledAt(undefined);

    const load = async () => {
      if (!alive || reading || document.hidden || finishedRef.current) return;
      reading = true;
      window.clearTimeout(timer);
      try {
        const current = await processesApi.getProcess(processId);
        if (!alive) return;

        setProcess(current);
        const sampled = performance.now();
        setSampledAt(sampled);
        setNow(sampled);
        if (current.completed && !finishedRef.current) {
          finishedRef.current = true;
          onFinishedRef.current?.(current);
        }
      } catch {
        // La pantalla ya muestra el error de lanzamiento; un fallo momentaneo de sondeo no lo
        // debe reemplazar cada segundo.
      } finally {
        reading = false;
        if (alive && !document.hidden && !finishedRef.current) timer = window.setTimeout(() => void load(), 1000);
      }
    };

    const visibility = () => {
      setVisible(!document.hidden);
      if (document.hidden) window.clearTimeout(timer);
      else void load();
    };

    void load();
    document.addEventListener('visibilitychange', visibility);
    return () => {
      alive = false;
      window.clearTimeout(timer);
      document.removeEventListener('visibilitychange', visibility);
    };
  }, [processId]);

  // El servidor confirma el estado una vez por segundo. Entre dos muestras, el reloj local
  // anticipa el siguiente avance para que la barra no vaya a saltos.
  useEffect(() => {
    if (!visible || !process || process.id !== processId || process.completed || process.paused) return;
    const timer = window.setInterval(() => setNow(performance.now()), 50);
    return () => window.clearInterval(timer);
  }, [visible, process?.id, process?.completed, process?.paused, processId]);

  const current = process?.id === processId ? process : undefined;

  const ideal = current?.ideal_seconds ?? initialSeconds;
  const reportedWorked = current?.completed
    ? ideal
    : Math.max(0, ideal - (current?.seconds_left ?? ideal));
  const elapsedSinceSample = current && !current.completed && !current.paused && sampledAt
    ? Math.min(1.1, Math.max(0, (now - sampledAt) / 1_000))
    : 0;
  const worked = Math.min(ideal, reportedWorked + elapsedSinceSample);
  const percentage = current?.completed
    ? 100
    : ideal > 0
      ? Math.min(99, (worked / ideal) * 100)
      : 0;
  const state = current?.completed
    ? tr(current.status === 'succeeded' ? 'completado' : current.status === 'cancelled' ? 'cancelled' : 'failed')
    : current?.paused ? tr("en pausa") : current?.seconds_left === 0 ? 'finalizando…' : tr("en curso");

  return (
    <div className={`process-progress${current?.paused ? ' is-paused' : ''}${current?.completed ? ' is-finished' : ''}`} data-process-id={processId}>
      <p>
        <b>{title ?? tr("Tarea #{0}", processId)}</b><span>{Math.floor(percentage)}%</span>
      </p>
      <div
        className="process-progress__track"
        role="progressbar"
        aria-label={title ?? tr("Tarea #{0}", processId)}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(percentage)}
      >
        <span className="process-progress__fill" style={{ transform: `scaleX(${percentage / 100})` }} />
      </div>
      <small>{state}{ideal > 0 && ` · ${Math.floor(worked)} / ${ideal} s`}{title && tr(" · tarea #{0}", processId)}</small>
      {current?.completed && current.result_message && <small role="status">{current.result_message}</small>}
    </div>
  );
}
