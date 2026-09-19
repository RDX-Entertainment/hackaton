import { tr, useIdioma } from '../../i18n/text';
import { useEffect, useRef, useState } from 'react';

import { adminApi, type Chipset } from '../../utils/api';
import { messageFrom } from '../../utils/errors';

type ChipsetPickerProps = {
  chipsets: Chipset[];
  value: number | null;
  onChange: (id: number | null) => void;
  onCreated: () => Promise<void>;
  label?: string;
  emptyLabel?: string;
};

export function ChipsetPicker({
  chipsets,
  value,
  onChange,
  onCreated,
  label = 'chipset',
  emptyLabel = tr("sin chipset"),
}: ChipsetPickerProps) {
  useIdioma();
  const [open, setOpen] = useState(false);
  const [name, setName] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  const root = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) {
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

  const selected = chipsets.find((chipset) => chipset.id === value);

  const create = async () => {
    if (!name.trim()) {
      return;
    }

    setBusy(true);
    setError('');

    try {
      const made = await adminApi.createChipset(name.trim());
      await onCreated();
      onChange(made.id);
      setName('');
    } catch (cause) {
      setError(messageFrom(cause));
    } finally {
      setBusy(false);
    }
  };

  const remove = async (chipset: Chipset) => {
    setBusy(true);
    setError('');

    try {
      await adminApi.removeChipset(chipset.id);
      await onCreated();

      if (value === chipset.id) {
        onChange(null);
      }
    } catch (cause) {
      setError(messageFrom(cause));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="chipset" ref={root}>
      <span className="chipset__label">{label}</span>

      <button
        type="button"
        className="chipset__toggle"
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
      >
        <span className={selected ? 'chipset__value' : 'chipset__value is-empty'}>
          {selected?.name ?? emptyLabel}
        </span>
        <span className="chipset__caret" aria-hidden="true">
          ▾
        </span>
      </button>

      {open && (
        <div className="chipset__menu">
          <div className="chipset__create">
            <input
              value={name}
              placeholder={tr("nuevo chipset")}
              autoFocus
              onChange={(event) => setName(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') {
                  event.preventDefault();
                  void create();
                }

                if (event.key === 'Escape') {
                  setOpen(false);
                }
              }}
            />
            <button
              type="button"
              className="chipset__add"
              disabled={busy || !name.trim()}
              aria-label={tr("Crear chipset")}
              onClick={() => void create()}
            >
              +
            </button>
          </div>

          {error && <p className="chipset__error">{error}</p>}

          <ul className="chipset__list">
            <li>
              <button
                type="button"
                className={value === null ? 'chipset__item is-active' : 'chipset__item'}
                onClick={() => {
                  onChange(null);
                  setOpen(false);
                }}
              >
                {emptyLabel}
              </button>
            </li>

            {chipsets.map((chipset) => (
              <li key={chipset.id}>
                <button
                  type="button"
                  className={value === chipset.id ? 'chipset__item is-active' : 'chipset__item'}
                  onClick={() => {
                    onChange(chipset.id);
                    setOpen(false);
                  }}
                >
                  {chipset.name}
                </button>

                <button
                  type="button"
                  className="chipset__remove"
                  disabled={busy}
                  aria-label={tr("Borrar {0}", chipset.name)}
                  onClick={() => void remove(chipset)}
                >
                  ×
                </button>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
