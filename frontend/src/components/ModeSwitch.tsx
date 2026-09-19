import { tr, useIdioma } from '../i18n/text';
export type AuthMode = 'normal' | 'console';

const MODES: { value: AuthMode; label: string }[] = [
  { value: 'normal', label: 'Normal' },
  { value: 'console', get label() { return tr("Console"); } },
];

type ModeSwitchProps = {
  mode: AuthMode;
  onChange: (mode: AuthMode) => void;
  /** Once a session exists the choice is gone, so the tabs stop responding. */
  disabled?: boolean;
};

/** Segmented control in the title bar: a plain form, or the shell. */
export function ModeSwitch({ mode, onChange, disabled = false }: ModeSwitchProps) {
  useIdioma();
  return (
    <div className="mode-switch" role="tablist" aria-label={tr("Modo de entrada")}>
      {MODES.map((option) => (
        <button
          key={option.value}
          type="button"
          role="tab"
          aria-selected={mode === option.value}
          disabled={disabled}
          tabIndex={disabled ? -1 : undefined}
          className={mode === option.value ? 'is-active' : undefined}
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}
