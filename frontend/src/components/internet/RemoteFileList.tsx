import type { RemoteFile } from '../../utils/api';
import { formatCount } from '../../utils/format';

type RemoteFileListProps = {
  files: RemoteFile[];
  onDownload: (file: RemoteFile) => void;
  onDelete: (file: RemoteFile) => void;
};

export function RemoteFileList({ files, onDownload, onDelete }: RemoteFileListProps) {
  if (!files.length) {
    return <p className="rfiles__empty">Sin ficheros, o ninguno visible desde aquí.</p>;
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
              {formatCount(file.size)} MB · {formatCount(file.ram)} MB en memoria · tipo{' '}
              {file.type}
            </span>
          </span>

          <span className="rfile__actions">
            <button type="button" onClick={() => onDownload(file)}>
              bajar
            </button>
            <button type="button" onClick={() => onDelete(file)}>
              borrar
            </button>
          </span>
        </li>
      ))}
    </ul>
  );
}
