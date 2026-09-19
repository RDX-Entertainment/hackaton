import { tr, useIdioma } from '../../i18n/text';
import { useEffect, useState } from 'react';
import { softwareApi, type OwnedFile } from '../../utils/api';

export function RemoteUpload({ busy, onUpload }: { busy: boolean; onUpload: (id: number) => void }) {
  useIdioma();
  const [files, setFiles] = useState<OwnedFile[]>([]);
  const [selected, setSelected] = useState('');
  const [error, setError] = useState('');
  useEffect(() => {
    let current = true;
    softwareApi.listSoftware().then((items) => { if (current) setFiles(items.filter((file) => file.type !== 19 && file.type !== 31 && file.type < 90)); })
      .catch(() => { if (current) setError(tr("No se pudo cargar tu disco para subir archivos.")); });
    return () => { current = false; };
  }, []);
  return <div className="webedit__actions"><label>{tr("Subir desde tu disco")} <select value={selected} onChange={(e) => setSelected(e.target.value)}>
    <option value="">{tr("Elige un archivo")}</option>{files.map((file) => <option key={file.id} value={file.id}>{file.name} · {file.size} MB</option>)}
  </select></label><button disabled={busy || !selected} onClick={() => onUpload(Number(selected))}>{tr("Subir archivo")}</button>{error && <small role="alert">{error}</small>}</div>;
}
