import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { Loading } from '../../components/Notice';
import { StatusMessage } from '../../components/StatusMessage';
import {
  adminApi,
  softwareApi,
  type AdminSoftwareInput,
  type AdminUser,
  type OwnedFile,
  type SoftwareType,
} from '../../utils/api';
import { useStatusMessage } from '../../utils/useStatusMessage';

const blank = (): AdminSoftwareInput => ({
  name: '', soft_type: 1, version: 1, size: 30, ram: 50, running: false,
});

export function SoftwareAdminPage() {
  const status = useStatusMessage();
  const [users, setUsers] = useState<AdminUser[]>();
  const [types, setTypes] = useState<SoftwareType[]>([]);
  const [userId, setUserId] = useState<number>();
  const [files, setFiles] = useState<OwnedFile[]>([]);
  const [draft, setDraft] = useState<AdminSoftwareInput>(blank);
  const [editing, setEditing] = useState<number>();
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    adminApi.listUsers().then((items) => {
      setUsers(items);
      setUserId((current) => current ?? items[0]?.id);
    }).catch(status.reportError);
    softwareApi.listSoftwareTypes().then(setTypes).catch(status.reportError);
  }, [status.reportError]);

  const loadFiles = useCallback(() => {
    if (userId) adminApi.listUserSoftware(userId).then(setFiles).catch(status.reportError);
  }, [status.reportError, userId]);

  useEffect(() => { loadFiles(); }, [loadFiles]);

  const selected = users?.find((user) => user.id === userId);
  const set = <K extends keyof AdminSoftwareInput>(key: K, value: AdminSoftwareInput[K]) =>
    setDraft((current) => ({ ...current, [key]: value }));
  const reset = () => { setEditing(undefined); setDraft(blank()); };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!userId) return;
    setBusy(true);
    try {
      if (editing) {
        await adminApi.editUserSoftware(userId, editing, draft);
        status.report('Software actualizado.');
      } else {
        await adminApi.createUserSoftware(userId, draft);
        status.report('Software añadido.');
      }
      reset();
      loadFiles();
    } catch (error) { status.reportError(error); } finally { setBusy(false); }
  };

  const run = async (work: () => Promise<void>, message: string) => {
    setBusy(true);
    try { await work(); status.report(message); loadFiles(); } catch (error) { status.reportError(error); } finally { setBusy(false); }
  };

  if (!users) return <Loading />;

  return (
    <>
      <h2>Software de usuarios</h2>
      <p className="muted">Gestiona el inventario y los programas ejecutándose de cualquier usuario.</p>
      <StatusMessage text={status.message} as="block" />

      <label className="admin-software__user">
        Usuario
        <select value={userId ?? ''} onChange={(event) => { setUserId(Number(event.target.value)); reset(); }}>
          {users.map((user) => <option key={user.id} value={user.id}>{user.login} · #{user.id}</option>)}
        </select>
      </label>

      <h3>{selected?.login ?? 'Usuario'} · archivos ({files.length})</h3>
      <div className="table-wrap">
        <table>
          <thead><tr><th>nombre</th><th>tipo</th><th>versión</th><th>disco</th><th>RAM</th><th>estado</th><th /></tr></thead>
          <tbody>
            {files.map((file) => (
              <tr key={file.id}>
                <td>{file.name}</td>
                <td>{types.find((type) => type.id === file.type)?.name ?? file.type}</td>
                <td>v{file.version}</td><td>{file.size}</td><td>{file.ram}</td>
                <td>{file.running ? 'ejecutándose' : 'en disco'}</td>
                <td>
                  <button type="button" disabled={busy} onClick={() => void run(() => adminApi.setUserSoftwareRunning(userId!, file.id, !file.running), file.running ? 'Programa retirado de memoria.' : 'Programa cargado en memoria.')}>{file.running ? 'retirar' : 'ejecutar'}</button>
                  <button type="button" disabled={busy} onClick={() => { setEditing(file.id); setDraft({ name: file.name, soft_type: file.type, version: file.version, size: file.size, ram: file.ram, running: file.running }); }}>editar</button>
                  <button type="button" disabled={busy} onClick={() => void run(() => adminApi.removeUserSoftware(userId!, file.id), 'Software eliminado.')}>eliminar</button>
                </td>
              </tr>
            ))}
            {!files.length && <tr><td colSpan={7}>Este usuario no tiene software.</td></tr>}
          </tbody>
        </table>
      </div>

      <form className="compose admin-software__form" onSubmit={submit}>
        <h3>{editing ? 'Editar software' : 'Añadir software'}</h3>
        <label>Nombre<input required maxLength={25} value={draft.name} onChange={(event) => set('name', event.target.value)} /></label>
        <label>Tipo<select value={draft.soft_type} onChange={(event) => set('soft_type', Number(event.target.value))}>{types.map((type) => <option key={type.id} value={type.id}>{type.name}</option>)}</select></label>
        <label>Versión<input required min={1} type="number" value={draft.version} onChange={(event) => set('version', Number(event.target.value))} /></label>
        <label>Disco<input required min={1} type="number" value={draft.size} onChange={(event) => set('size', Number(event.target.value))} /></label>
        <label>RAM<input required min={0} type="number" value={draft.ram} onChange={(event) => set('ram', Number(event.target.value))} /></label>
        <label className="admin-software__check"><input type="checkbox" checked={draft.running} onChange={(event) => set('running', event.target.checked)} /> Ejecutar al guardar</label>
        <p><button disabled={busy}>{editing ? 'guardar cambios' : 'añadir software'}</button>{editing && <button type="button" onClick={reset}>cancelar</button>}</p>
      </form>
    </>
  );
}
