import { useCallback, useEffect, useMemo, useState, type FormEvent } from 'react';

import { EmptyNotice, Loading } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import { arsenalApi, type Program, type ProgramStats } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

const PROPERTY_FIELDS: { key: keyof Pick<ProgramStats, 'effectiveness' | 'penetration' | 'stealth' | 'detection' | 'removal' | 'stability'>; label: string }[] = [
  { key: 'effectiveness', label: 'eficacia' }, { key: 'penetration', label: 'penetración' },
  { key: 'stealth', label: 'sigilo' }, { key: 'detection', label: 'detección' },
  { key: 'removal', label: 'eliminación' }, { key: 'stability', label: 'estabilidad' },
];
const RESOURCE_FIELDS: { key: keyof Pick<ProgramStats, 'cpu' | 'ram' | 'net' | 'weight'>; label: string }[] = [
  { key: 'cpu', label: 'CPU' }, { key: 'ram', label: 'RAM' }, { key: 'net', label: 'red' }, { key: 'weight', label: 'peso en disco' },
];

export function LabPage() {
  const status = useStatusMessage();
  const [programs, setPrograms] = useState<Program[]>();
  const [selectedId, setSelectedId] = useState<number>();
  const [draft, setDraft] = useState<ProgramStats>();
  const [busy, setBusy] = useState(false);

  const load = useCallback(() => {
    arsenalApi.listLabPrograms().then((items) => {
      setPrograms(items);
      setSelectedId((current) => current ?? items[0]?.software_id);
    }).catch(status.reportError);
  }, [status.reportError]);

  useEffect(() => { load(); }, [load]);

  const selected = useMemo(() => programs?.find((program) => program.software_id === selectedId), [programs, selectedId]);
  useEffect(() => { if (selected) setDraft(selected.stats); }, [selected]);

  const set = (key: keyof ProgramStats, value: number) => setDraft((current: ProgramStats | undefined) => current && ({ ...current, [key]: value }));
  const save = async (event: FormEvent) => {
    event.preventDefault();
    if (!selected || !draft) return;
    setBusy(true);
    try { await arsenalApi.tuneProgram(selected.software_id, draft); status.report('Perfil del programa actualizado.'); load(); }
    catch (error) { status.reportError(error); } finally { setBusy(false); }
  };

  if (!programs) return <Loading />;
  if (!programs.length) return <><h2>Lab</h2><EmptyNotice>Este usuario aún no tiene software con perfil. Entra con una cuenta de laboratorio o compila un programa desde Arsenal.</EmptyNotice></>;

  return (
    <>
      <h2>Lab de software</h2>
      <p className="muted">Aquí se prueban propiedades. La versión no altera el resultado: las operaciones leen estos valores al iniciarse.</p>
      <StatusMessage text={status.message} as="block" />
      <label className="lab__picker">Programa
        <select value={selectedId ?? ''} onChange={(event) => setSelectedId(Number(event.target.value))}>
          {programs.map((program) => <option key={program.software_id} value={program.software_id}>{program.name} · {program.capability_name ?? 'sin capacidad'} · {program.role}</option>)}
        </select>
      </label>
      {selected && draft && (
        <form className="compose lab__form" onSubmit={save}>
          <h3>{selected.name} · {selected.capability_name}</h3>
          <p className="muted">{selected.running ? 'En memoria' : 'En disco: instálalo antes de lanzar una operación.'}</p>
          <fieldset><legend>Propiedades (0–100)</legend>{PROPERTY_FIELDS.map((field) => <label key={field.key}>{field.label}<input type="number" min={0} max={100} value={draft[field.key]} onChange={(event) => set(field.key, Number(event.target.value))} /></label>)}</fieldset>
          <fieldset><legend>Coste de ejecución</legend>{RESOURCE_FIELDS.map((field) => <label key={field.key}>{field.label}<input type="number" min={1} value={draft[field.key]} onChange={(event) => set(field.key, Number(event.target.value))} /></label>)}</fieldset>
          <button disabled={busy}>guardar perfil</button>
        </form>
      )}
    </>
  );
}
