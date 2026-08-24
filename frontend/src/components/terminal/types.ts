import type { ReactNode } from 'react';

/** How a scrollback line is coloured. */
export type LineKind = 'input' | 'output' | 'error' | 'success' | 'hint';

export type Line = {
  id: number;
  kind: LineKind;
  /** Plain text, rendered in a <pre> so art keeps its shape. */
  text?: string;
  /** An arbitrary block, for output that is a component rather than a string. */
  node?: ReactNode;
};

/** One question in a multi-step command such as `login`. */
export type Step = {
  key: string;
  label: string;
  /** Echoes as dots and hides the typed characters. */
  mask?: boolean;
};

export type Answers = Record<string, string>;

/** What a command may do to the terminal it runs in. */
export type CommandContext = {
  /** Arguments after the command name, already split on whitespace. */
  args: string[];
  /** The raw text after the command name, for commands that want it whole. */
  rest: string;
  print: (text: string, kind?: LineKind) => void;
  printNode: (node: ReactNode) => void;
  clear: () => void;
  /** Asks a sequence of questions, then runs `then` with the answers. */
  ask: (steps: Step[], then: (answers: Answers) => Promise<void> | void) => void;
  /** Commands entered so far, oldest first. */
  history: string[];
  /** What a command can do to the session. */
  session: {
    login: (identifier: string, password: string) => Promise<void>;
    register: (username: string, email: string, password: string) => Promise<void>;
    /**
     * Leaves the shell for the graphical interface. Authenticating does not do
     * this on its own: you land back at a prompt, and starting the UI is a
     * separate, deliberate command.
     */
    startUi: () => void;
    /** Who is signed in right now, or null. */
    identity: () => { login: string; ip: number } | null;
  };
};

export type CommandGroup = 'sesión' | 'sistema' | 'extras';

export type Command = {
  name: string;
  group: CommandGroup;
  /** One line, shown by `help`. */
  summary: string;
  usage?: string;
  /** Alternative names that reach the same command. */
  aliases?: string[];
  run: (context: CommandContext) => void | Promise<void>;
};
