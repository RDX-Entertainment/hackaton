import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type FormEvent,
  type KeyboardEvent,
  type ReactNode,
} from 'react';
import { useNavigate } from 'react-router-dom';

import { authenticated } from '../../app/authSlice';
import { useAppDispatch, useAppSelector } from '../../app/hooks';
import { authApi } from '../../utils/api';
import { messageFrom } from '../../utils/errors';
import { formatIp } from '../../utils/ip';
import { findCommand } from './commands';
import type { Answers, Line, LineKind, Step } from './types';

const HOST = 'hackaton';

type Pending = {
  steps: Step[];
  index: number;
  answers: Answers;
  then: (answers: Answers) => Promise<void> | void;
};

type TerminalProps = {
  /** Run this on mount, as if it had been typed. */
  initialCommand?: string;
};

export function Terminal({ initialCommand }: TerminalProps) {
  const dispatch = useAppDispatch();
  const navigate = useNavigate();
  const token = useAppSelector((state) => state.auth.token);
  const user = useAppSelector((state) => state.auth.user);

  // Commands run from callbacks that outlive the render they were built in, so
  // the session they read has to come from a ref rather than a closed-over prop.
  const account = useRef({ token, user });
  useEffect(() => {
    account.current = { token, user };
  }, [token, user]);

  const promptUser = user?.login ?? 'guest';
  const prompt = `${promptUser}@${HOST}:~$`;

  const [lines, setLines] = useState<Line[]>([]);
  const [input, setInput] = useState('');
  const [pending, setPending] = useState<Pending | null>(null);
  const [busy, setBusy] = useState(false);
  const [history, setHistory] = useState<string[]>([]);
  const [historyCursor, setHistoryCursor] = useState<number | null>(null);

  const nextId = useRef(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);

  const append = useCallback((line: Omit<Line, 'id'>) => {
    setLines((current) => [...current, { ...line, id: (nextId.current += 1) }]);
  }, []);

  const print = useCallback(
    (text: string, kind: LineKind = 'output') => append({ kind, text }),
    [append],
  );

  const printNode = useCallback((node: ReactNode) => append({ kind: 'output', node }), [append]);

  const clear = useCallback(() => setLines([]), []);

  // New output scrolls into view, and clicking anywhere focuses the input.
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ block: 'end' });
  }, [lines, pending]);

  const session = useRef({
    login: async (identifier: string, password: string) => {
      const result = await authApi.login(identifier, password);
      dispatch(authenticated(result));
      announceSession.current(result.user.login, result.user.game_ip);
    },
    register: async (username: string, email: string, password: string) => {
      const result = await authApi.register(username, password, email);
      dispatch(authenticated(result));
      announceSession.current(result.user.login, result.user.game_ip);
    },
    /** `systemctl start UI` and its shortcut both land here. */
    startUi: () => {
      if (!account.current.token) {
        printRef.current('No hay sesión. Autentícate antes con `login`.', 'error');
        return;
      }

      printRef.current('Starting hacker-experience-ui.service …', 'success');
      navigate('/');
    },
    identity: () => {
      const current = account.current.user;
      return current ? { login: current.login, ip: current.game_ip } : null;
    },
  });

  // Both are refs for the same reason `account` is: `session` is built once and
  // must not capture a stale `print`.
  const printRef = useRef(print);
  printRef.current = print;

  const announceSession = useRef((login: string, ip: number) => {
    printRef.current(`Autenticado como ${login} · ip ${formatIp(ip)}`, 'success');
    printRef.current(
      'La interfaz no está levantada. Arráncala con `systemctl start UI`, o `start-ui`.',
      'hint',
    );
  });

  const ask = useCallback((steps: Step[], then: Pending['then']) => {
    setPending({ steps, index: 0, answers: {}, then });
  }, []);

  const dispatchCommand = useCallback(
    async (raw: string, recorded: string[]) => {
      const trimmed = raw.trim();
      if (!trimmed) {
        return;
      }

      const [name, ...args] = trimmed.split(/\s+/);
      const command = findCommand(name);

      if (!command) {
        print(`${name}: orden no encontrada. Escribe \`help\`.`, 'error');
        return;
      }

      const rest = trimmed.slice(name.length).trim();

      setBusy(true);
      try {
        await command.run({
          args,
          rest,
          print,
          printNode,
          clear,
          ask,
          history: recorded,
          session: session.current,
        });
      } catch (error) {
        print(messageFrom(error), 'error');
      } finally {
        setBusy(false);
      }
    },
    [ask, clear, print, printNode],
  );

  /** Answers the question in flight, and runs the command once they are all in. */
  const answerStep = useCallback(
    async (value: string) => {
      if (!pending) {
        return;
      }

      const step = pending.steps[pending.index];
      append({ kind: 'input', text: `${step.label}: ${step.mask ? '•'.repeat(value.length) : value}` });

      const answers = { ...pending.answers, [step.key]: value };
      const next = pending.index + 1;

      if (next < pending.steps.length) {
        setPending({ ...pending, index: next, answers });
        return;
      }

      setPending(null);
      setBusy(true);

      try {
        await pending.then(answers);
      } catch (error) {
        print(messageFrom(error), 'error');
      } finally {
        setBusy(false);
      }
    },
    [append, pending, print],
  );

  const submit = async (event: FormEvent) => {
    event.preventDefault();

    const value = input;
    setInput('');
    setHistoryCursor(null);

    if (pending) {
      await answerStep(value);
      return;
    }

    append({ kind: 'input', text: `${prompt} ${value}` });

    const recorded = value.trim() ? [...history, value.trim()] : history;
    setHistory(recorded);

    await dispatchCommand(value, recorded);
  };

  /** Arrow keys walk the history; Ctrl+L clears, as in a real shell. */
  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.ctrlKey && event.key.toLowerCase() === 'l') {
      event.preventDefault();
      clear();
      return;
    }

    if (pending || !history.length) {
      return;
    }

    if (event.key === 'ArrowUp') {
      event.preventDefault();
      const cursor = historyCursor === null ? history.length - 1 : Math.max(0, historyCursor - 1);
      setHistoryCursor(cursor);
      setInput(history[cursor]);
      return;
    }

    if (event.key === 'ArrowDown') {
      event.preventDefault();

      if (historyCursor === null) {
        return;
      }

      const cursor = historyCursor + 1;
      if (cursor >= history.length) {
        setHistoryCursor(null);
        setInput('');
        return;
      }

      setHistoryCursor(cursor);
      setInput(history[cursor]);
    }
  };

  // The opening banner, plus whatever the route asked to run.
  const started = useRef(false);
  useEffect(() => {
    if (started.current) {
      return;
    }
    started.current = true;

    print('HackerOS 0.1.0 · sesión de invitado', 'success');
    print('Escribe `help` para ver qué se puede hacer aquí.', 'hint');

    const boot = async () => {
      append({ kind: 'input', text: `${prompt} neofetch` });
      await dispatchCommand('neofetch', []);

      if (initialCommand) {
        append({ kind: 'input', text: `${prompt} ${initialCommand}` });
        await dispatchCommand(initialCommand, []);
      }
    };

    void boot();
    // `prompt` is deliberately absent: the banner runs once, as guest.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [append, dispatchCommand, initialCommand, print]);

  const step = pending?.steps[pending.index];

  return (
    <div className="term" onClick={() => inputRef.current?.focus()}>
      {/*
        The entry line is the last child of the scrollback, not a bar pinned to
        the bottom of the window. A question therefore appears directly under
        the command that asked it, which is how a real shell reads.
      */}
      <div className="term__flow">
        {lines.map((line) => (
          <div className={`term__out term__out--${line.kind}`} key={line.id}>
            {line.node ?? <pre>{line.text}</pre>}
          </div>
        ))}

        <form className="term__entry" onSubmit={submit}>
          <label className="term__label" htmlFor="term-input">
            {step ? (
              <span className="term__question">{step.label}:</span>
            ) : (
              <>
                <span className="term__user">{`${promptUser}@${HOST}`}</span>
                <span className="term__sep">:</span>
                <span className="term__path">~</span>
                <span className="term__sigil">$</span>
              </>
            )}
          </label>

          <input
            id="term-input"
            ref={inputRef}
            className="term__input"
            value={input}
            onChange={(event) => setInput(event.target.value)}
            onKeyDown={onKeyDown}
            type={step?.mask ? 'password' : 'text'}
            autoComplete={step?.mask ? 'current-password' : 'off'}
            autoCapitalize="off"
            autoCorrect="off"
            spellCheck={false}
            disabled={busy}
            autoFocus
          />

          {/* Only while something is in flight: an idle blinking mark next to a
              real text cursor reads as a rendering glitch, not as a caret. */}
          {busy && (
            <span className="term__busy" aria-hidden="true">
              …
            </span>
          )}
        </form>

        <div ref={bottomRef} />
      </div>
    </div>
  );
}
