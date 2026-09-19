import { tr, useIdioma } from '../../i18n/text';
import { useEffect, useState, type FormEvent } from 'react';

import { formatCount } from '../../utils/format';
import { SiteFrame } from './SiteFrame';

/**
 * El mismo tope que aplica el servidor, y en la misma unidad.
 *
 * El servidor mide bytes UTF-8. Contar caracteres aquí haría mentir al contador en
 * cuanto la página lleve acentos — una `ó` ocupa dos — y la rechazaría al guardar
 * después de decir que quedaba sitio.
 */
const MAX_BYTES = 16384;

const encoder = new TextEncoder();

function bytesOf(text: string): number {
  return encoder.encode(text).length;
}

export type EditorLayout = 'split' | 'full';

const LAYOUTS: { value: EditorLayout; label: string; hint: string }[] = [
  { value: 'split', get label() { return tr("mitad y mitad"); }, get hint() { return tr("código y vista, uno al lado del otro"); } },
  { value: 'full', get label() { return tr("pantalla completa"); }, get hint() { return tr("la vista entera, y el código debajo"); } },
];

const RULES = [
  'Un solo documento: el CSS va dentro, en su <style>.',
  'Sin guiones. Ni <script>, ni onclick, ni URLs javascript:.',
  'Sin cargas de fuera: una imagen ajena sería un contador de visitas.',
];

type SiteEditorProps = {
  ip: number;
  name: string;
  web: string;
  busy: boolean;
  error: string;
  onSave: (web: string) => void;
};

/**
 * Escribir la página que sirve una máquina, con lo que se está escribiendo pintado al
 * lado. La vista previa usa el mismo marco aislado que el navegador, así que lo que ves
 * aquí es exactamente lo que verá quien te visite — incluido lo que se rompa.
 */
export function SiteEditor({ ip, name, web, busy, error, onSave }: SiteEditorProps) {
  useIdioma();
  const [draft, setDraft] = useState(web);
  const [live, setLive] = useState(web);
  const [layout, setLayout] = useState<EditorLayout>('split');

  // Otra máquina trae otra página, así que el borrador la sigue. Depende también de `ip`
  // porque dos sitios distintos pueden tener el mismo contenido — dos vacíos, sin ir más
  // lejos — y entonces cambiar de uno a otro no movería `web` y el borrador se quedaría.
  useEffect(() => {
    setDraft(web);
    setLive(web);
  }, [web, ip]);

  // La vista previa va un respiro por detrás: recrear el marco en cada tecla lo haría
  // parpadear y no dejaría leer nada.
  useEffect(() => {
    const timer = setTimeout(() => setLive(draft), 400);
    return () => clearTimeout(timer);
  }, [draft]);

  const size = bytesOf(draft);
  const over = size > MAX_BYTES;
  const dirty = draft !== web;

  const save = (event: FormEvent) => {
    event.preventDefault();
    onSave(draft);
  };

  return (
    <form className={`editor editor--${layout}`} onSubmit={save}>
      <div className="editor__modes">
        <span className="editor__modeslabel">{tr("vista")}</span>

        {LAYOUTS.map((one) => (
          <button
            type="button"
            key={one.value}
            className={layout === one.value ? 'editor__mode is-active' : 'editor__mode'}
            title={one.hint}
            aria-pressed={layout === one.value}
            onClick={() => setLayout(one.value)}
          >
            <span className={`editor__glyph editor__glyph--${one.value}`} aria-hidden="true" />
            {one.label}
          </button>
        ))}
      </div>

      <div className="editor__code">
        <textarea
          className="editor__area"
          value={draft}
          spellCheck={false}
          onChange={(event) => setDraft(event.target.value)}
        />

        <ul className="editor__rules">
          {RULES.map((rule) => (
            <li key={rule}>{tr(rule)}</li>
          ))}
        </ul>

        {error && <p className="editor__error">{error}</p>}

        <div className="editor__foot">
          <span className={over ? 'editor__count is-over' : 'editor__count'}>
            {over
              ? tr("{0} bytes de más", formatCount(size - MAX_BYTES))
              : tr("{0} bytes libres", formatCount(MAX_BYTES - size))}
          </span>

          <span className="editor__buttons">
            <button type="button" disabled={busy || !dirty} onClick={() => setDraft(web)}>
              {tr("descartar")} </button>
            <button className="is-primary" disabled={busy || !dirty || over}>
              {busy ? tr("guardando…") : tr("publicar")}
            </button>
          </span>
        </div>
      </div>

      <div className="editor__preview">
        <SiteFrame ip={ip} html={live} name={name} down={false} />
      </div>
    </form>
  );
}
