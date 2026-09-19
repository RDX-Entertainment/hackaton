import { idiomaActual } from './index';
import { textCatalog } from './textCatalog';

export { useIdioma } from './index';
export { idiomaActual };

/** Source-text keys for interface copy. Never pass player-authored content here. */
export function tr(source: string, ...values: (string | number | null | undefined)[]): string {
  const entry = textCatalog[source];
  const text = entry?.[idiomaActual()] ?? source;
  return text.replace(/\{(\d+)\}/g, (placeholder, index: string) =>
    Number(index) < values.length ? String(values[Number(index)] ?? '') : placeholder,
  );
}
