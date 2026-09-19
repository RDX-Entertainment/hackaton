import { tr, useIdioma } from '../../i18n/text';
import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { Loading } from '../../components/Notice';
import { ModuleAdmin } from '../../components/admin/ModuleAdmin';
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
  useIdioma();
  const [tab,setTab] = useState<'files'|'modules'>('files');
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
        status.report(tr("Software actualizado."));
      } else {
        await adminApi.createUserSoftware(userId, draft);
        status.report(tr("Software añadido."));
      }
      reset();
      loadFiles();
    } catch (error) { status.reportError(error); } finally { setBusy(false); }
  };

  const run = async (work: () => Promise<void>, message: string) => {
    setBusy(true);
    try { await work(); status.report(message); loadFiles(); } catch (error) { status.reportError(error); } finally { setBusy(false); }
  };

  const tabs = <nav className="mesas"><button onClick={() => setTab('files')}>{tr("Software de usuarios")}</button><button onClick={() => setTab('modules')}>{tr("Módulos")}</button></nav>;
  if (tab === 'modules') return <>{tabs}<ModuleAdmin /></>;
  if (!users) return <>{tabs}<StatusMessage text={status.message} /><Loading /></>;

  return (
    <>
      {tabs}
      <h2>{tr("Software de usuarios")}</h2>
      <p className="muted">{tr("Gestiona el inventario y los programas ejecutándose de cualquier usuario.")}</p>
      <StatusMessage text={status.message} as="block" />

      <label className="admin-software__user">
        {tr("Usuario")} <select value={userId ?? ''} onChange={(event) => { setUserId(Number(event.target.value)); reset(); }}>
          {users.map((user) => <option key={user.id} value={user.id}>{user.login} · #{user.id}</option>)}
        </select>
      </label>

      <h3>{selected?.login ?? tr("Usuario")} {tr("· archivos (")}{files.length})</h3>
      <div className="table-wrap">
        <table>
          <thead><tr><th>{tr("nombre")}</th><th>{tr("tipo")}</th><th>{tr("versión")}</th><th>{tr("disco")}</th><th>RAM</th><th>{tr("estado")}</th><th /></tr></thead>
          <tbody>
            {files.map((file) => (
              <tr key={file.id}>
                <td>{file.name}</td>
                <td>{types.find((type) => type.id === file.type)?.name ?? file.type}</td>
                <td>v{file.version}</td><td>{file.size}</td><td>{file.ram}</td>
                <td>{file.running ? tr("ejecutándose") : tr("en disco")}</td>
                <td>
                  <button type="button" disabled={busy} onClick={() => void run(() => adminApi.setUserSoftwareRunning(userId!, file.id, !file.running), file.running ? tr("Programa retirado de memoria.") : tr("Programa cargado en memoria."))}>{file.running ? 'retirar' : 'ejecutar'}</button>
                  <button type="button" disabled={busy} onClick={() => { setEditing(file.id); setDraft({ name: file.name, soft_type: file.type, version: file.version, size: file.size, ram: file.ram, running: file.running }); }}>{tr("editar")}</button>
                  <button type="button" disabled={busy} onClick={() => void run(() => adminApi.removeUserSoftware(userId!, file.id), tr("Software eliminado."))}>{tr("eliminar")}</button>
                </td>
              </tr>
            ))}
            {!files.length && <tr><td colSpan={7}>{tr("Este usuario no tiene software.")}</td></tr>}
          </tbody>
        </table>
      </div>

      <form className="compose admin-software__form" onSubmit={submit}>
        <h3>{editing ? tr("Editar software") : tr("Añadir software")}</h3>
        <label>{tr("Nombre")}<input required maxLength={25} value={draft.name} onChange={(event) => set('name', event.target.value)} /></label>
        <label>{tr("Tipo")}<select value={draft.soft_type} onChange={(event) => set('soft_type', Number(event.target.value))}>{types.map((type) => <option key={type.id} value={type.id}>{type.name}</option>)}</select></label>
        <label>{tr("Versión")}<input required min={1} type="number" value={draft.version} onChange={(event) => set('version', Number(event.target.value))} /></label>
        <label>{tr("Disco")}<input required min={1} type="number" value={draft.size} onChange={(event) => set('size', Number(event.target.value))} /></label>
        <label>RAM<input required min={0} type="number" value={draft.ram} onChange={(event) => set('ram', Number(event.target.value))} /></label>
        <label className="admin-software__check"><input type="checkbox" checked={draft.running} onChange={(event) => set('running', event.target.checked)} /> {tr("Ejecutar al guardar")}</label>
        <p><button disabled={busy}>{editing ? tr("guardar cambios") : tr("añadir software")}</button>{editing && <button type="button" onClick={reset}>{tr("cancelar")}</button>}</p>
      </form>
    </>
  );
}
