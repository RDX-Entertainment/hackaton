import { tr, useIdioma } from '../i18n/text';
import type { ReactNode } from 'react';

type NoticeProps = {
  children: ReactNode;
};

/** Esperando la primera carga. */
export function Loading({ children = tr("Cargando…") }: { children?: ReactNode }) {
  useIdioma();
  return <p className="muted">{children}</p>;
}

/**
 * Una carga que falló del todo, no una acción que fue rechazada.
 *
 * Lleva la forma del sistema —franja roja a la izquierda— porque el rojo
 * significa pérdida: algo no se ha hecho.
 */
export function ErrorNotice({ children }: NoticeProps) {
  useIdioma();
  return (
    <p className="nota nota--error">
      <span>{children}</span>
    </p>
  );
}

/** No hay nada que enseñar, pero tampoco ha ido nada mal. */
export function EmptyNotice({ children }: NoticeProps) {
  useIdioma();
  return <p className="muted">{children}</p>;
}
