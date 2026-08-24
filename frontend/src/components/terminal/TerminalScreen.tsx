import { useState } from 'react';

import { useAppSelector } from '../../app/hooks';

import { Hero } from '../Hero';
import { LoginForm } from '../LoginForm';
import { MacWindow } from '../MacWindow';
import { ModeSwitch, type AuthMode } from '../ModeSwitch';
import { Terminal } from './Terminal';

const STORAGE_KEY = 'legacy.authMode';

/** Remembers which way in you picked last time. */
function readMode(): AuthMode {
  try {
    return localStorage.getItem(STORAGE_KEY) === 'normal' ? 'normal' : 'console';
  } catch {
    // Private windows and blocked site data both land here.
    return 'console';
  }
}

function writeMode(mode: AuthMode): void {
  try {
    localStorage.setItem(STORAGE_KEY, mode);
  } catch {
    // Not remembering the choice is not worth an error.
  }
}

type TerminalScreenProps = {
  /** Console mode: run this as soon as the session opens. */
  initialCommand?: string;
  /** Normal mode: open on the register fields. */
  startRegistering?: boolean;
};

/**
 * The whole signed-out experience.
 *
 * Normal mode is a hero: title and slogan on the left, the window on the right.
 * Console mode collapses the hero, which lets the window slide to the middle —
 * the movement is a consequence of the layout, not a scripted animation, so
 * there is only ever one source of truth for where things are.
 *
 * Signing in from the shell retires the mode switch: it collapses away rather
 * than unmounting, so it animates out instead of blinking out of existence.
 */
export function TerminalScreen({ initialCommand, startRegistering }: TerminalScreenProps) {
  const [mode, setMode] = useState<AuthMode>(readMode);
  // Authenticating inside the shell ends the choice: from then on there is a
  // session, and the only way onwards is `systemctl start UI`.
  const signedIn = useAppSelector((state) => state.auth.token) !== null;

  const change = (next: AuthMode) => {
    setMode(next);
    writeMode(next);
  };

  const isConsole = mode === 'console';
  const title = isConsole ? 'guest@hackaton: ~ — bash' : 'hacker experience — entrar';

  return (
    <main className={`auth auth--${mode}`}>
      <Hero hidden={isConsole} />

      <div className="auth__window">
        <MacWindow
          title={title}
          toolbar={
            <div className={`mode-switch-slot${signedIn ? ' is-gone' : ''}`}>
              <ModeSwitch mode={mode} onChange={change} disabled={signedIn} />
            </div>
          }
        >
          {isConsole ? (
            <Terminal initialCommand={initialCommand} />
          ) : (
            <LoginForm startRegistering={startRegistering} />
          )}
        </MacWindow>
      </div>
    </main>
  );
}
