import { tr, useIdioma } from '../../i18n/text';
import { HttpError } from '../../utils/api/client';
import { messageFrom } from '../../utils/errors';

export function AdminLoadError({ title, error, onRetry }: { title: string; error: unknown; onRetry: () => void }) {
  useIdioma();
  const status = error instanceof HttpError ? error.status : null;
  return <>
    <h2>{title}</h2>
    <section className="panel" role="alert" style={{ padding: 20 }}>
      <h3>{tr("No se pudieron cargar los datos")}</h3>
      <p>{status === 401 ? tr("La sesión ha caducado. Vuelve a iniciar sesión.")
        : status === 403 || status === 404 ? tr("El servidor no permitió consultar este recurso. Si acaban de darte acceso de administrador, pulsa Reintentar.")
        : tr("Falló la conexión o la respuesta del servidor. Esto no significa que hayas perdido el permiso de administrador.")}</p>
      <p className="muted">{status ? `HTTP ${status} · ` : ''}{messageFrom(error)}</p>
      <button type="button" onClick={onRetry}>{tr("Reintentar carga")}</button>
    </section>
  </>;
}
