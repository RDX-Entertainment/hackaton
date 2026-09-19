import { tr, useIdioma } from '../../i18n/text';
import type { RemoteFile } from '../../utils/api';
import { formatCount } from '../../utils/format';

type RemoteFileListProps = {
  files: RemoteFile[];
  onDownload: (file: RemoteFile) => void;
  onDelete: (file: RemoteFile) => void;
  canDelete?: boolean;
  busy?: boolean;
};

export function RemoteFileList({ files, onDownload, onDelete, canDelete = true, busy = false }: RemoteFileListProps) {
  useIdioma();
  if (!files.length) {
    return <p className="rfiles__empty">{tr("Sin ficheros, o ninguno visible desde aquí.")}</p>;
  }

  return (
    <ul className="rfiles">
      {files.map((file) => (
        <li className="rfile" key={file.id}>
          <span className="rfile__icon" aria-hidden="true">
            {file.folder ? '▤' : '▪'}
          </span>

          <span className="rfile__text">
            <span className="rfile__name">
              {file.name}
              <small>v{file.version}</small>
            </span>
            <span className="rfile__spec">
              {formatCount(file.size)} MB · {formatCount(file.ram)} {tr("MB en memoria · tipo")}{' '}
              {file.type}
            </span>
          </span>

          <span className="rfile__actions">
            <button type="button" disabled={busy} onClick={() => onDownload(file)}>
              {tr("bajar")} </button>
            <button type="button" disabled={busy || !canDelete} onClick={() => onDelete(file)}>
              {tr("borrar")} </button>
          </span>
        </li>
      ))}
    </ul>
  );
}
