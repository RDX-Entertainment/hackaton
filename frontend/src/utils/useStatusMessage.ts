import { useCallback, useMemo, useState } from 'react';

import { messageFrom } from './errors';

export type StatusMessage = {
  /** The line currently shown to the player. Empty means nothing to say. */
  message: string;
  /** Reports a successful outcome. */
  report: (text: string) => void;
  /** Turns a rejected promise into a readable line. */
  reportError: (error: unknown) => void;
  /** Clears the line, typically before starting new work. */
  clear: () => void;
};

/**
 * Nearly every screen keeps one line of feedback under its form. Holding that
 * in a hook keeps the `catch` blocks down to a single call.
 */
export function useStatusMessage(): StatusMessage {
  const [message, setMessage] = useState('');

  const report = useCallback((text: string) => {
    setMessage(text);
  }, []);

  const reportError = useCallback((error: unknown) => {
    setMessage(messageFrom(error));
  }, []);

  const clear = useCallback(() => {
    setMessage('');
  }, []);

  // Memoised so callers can safely list it as a hook dependency.
  return useMemo(
    () => ({ message, report, reportError, clear }),
    [message, report, reportError, clear],
  );
}
