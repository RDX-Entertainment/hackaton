import { tr, useIdioma, idiomaActual } from '../i18n/text';
import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react';

export type SearchOption = {
  id: number;
  label: string;
  hint?: string;
  terms?: string;
  icon?: ReactNode;
  badge?: ReactNode;
};

type SearchSelectProps = {
  label: string;
  options: SearchOption[];
  value: number | null;
  onChange: (id: number | null) => void;
  placeholder?: string;
  emptyLabel?: string;
  noResults?: string;
};

export function SearchSelect({
  label,
  options,
  value,
  onChange,
  placeholder = 'buscar…',
  emptyLabel = tr("sin elegir"),
  noResults = tr("nada encaja"),
}: SearchSelectProps) {
  useIdioma();
  const [open, setOpen] = useState(false);
  const [text, setText] = useState('');

  const root = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) {
      setText('');
      return;
    }

    const close = (event: MouseEvent) => {
      if (!root.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };

    document.addEventListener('mousedown', close);
    return () => document.removeEventListener('mousedown', close);
  }, [open]);

  const selected = options.find((option) => option.id === value);

  const found = useMemo(() => {
    const terms = text.trim().toLowerCase().split(/\s+/).filter(Boolean);

    if (!terms.length) {
      return options;
    }

    return options.filter((option) => {
      const haystack = `${option.label} ${option.hint ?? ''} ${option.terms ?? ''}`.toLowerCase();
      return terms.every((term) => haystack.includes(term));
    });
  }, [options, text, idiomaActual()]);

  const pick = (id: number) => {
    onChange(id);
    setOpen(false);
  };

  return (
    <div className="ssel" ref={root}>
      <span className="ssel__label">{label}</span>

      <button
        type="button"
        className="ssel__toggle"
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
      >
        {selected?.icon}
        <span className={selected ? 'ssel__value' : 'ssel__value is-empty'}>
          {selected?.label ?? emptyLabel}
        </span>
        <span className="ssel__caret" aria-hidden="true">
          ▾
        </span>
      </button>

      {open && (
        <div className="ssel__menu">
          <div className="ssel__search">
            <span className="ssel__glass" aria-hidden="true">
              ⌕
            </span>
            <input
              value={text}
              placeholder={placeholder}
              autoFocus
              onChange={(event) => setText(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Escape') {
                  setOpen(false);
                }

                if (event.key === 'Enter') {
                  event.preventDefault();

                  if (found.length) {
                    pick(found[0].id);
                  }
                }
              }}
            />
          </div>

          {found.length === 0 ? (
            <p className="ssel__none">{noResults}</p>
          ) : (
            <ul className="ssel__list">
              {found.map((option) => (
                <li key={option.id}>
                  <button
                    type="button"
                    className={option.id === value ? 'ssel__item is-active' : 'ssel__item'}
                    onClick={() => pick(option.id)}
                  >
                    {option.icon}

                    <span className="ssel__text">
                      <span className="ssel__name">{option.label}</span>
                      {option.hint && <span className="ssel__hint">{option.hint}</span>}
                    </span>

                    {option.badge}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}
