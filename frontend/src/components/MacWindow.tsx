import type { ReactNode } from 'react';

type MacWindowProps = {
  /** Shown centred in the title bar, the way a terminal shows its cwd. */
  title: string;
  /** Optional control placed at the right end of the title bar. */
  toolbar?: ReactNode;
  children: ReactNode;
};

/**
 * A macOS window frame: traffic lights on the left, title in the middle, and
 * room for one control on the right. The lights are decoration, not controls,
 * so they are spans rather than buttons and are hidden from assistive tech.
 */
export function MacWindow({ title, toolbar, children }: MacWindowProps) {
  return (
    <section className="mac-window">
      <header className="mac-window__bar">
        <span className="mac-window__lights" aria-hidden="true">
          <i className="mac-window__light mac-window__light--close" />
          <i className="mac-window__light mac-window__light--min" />
          <i className="mac-window__light mac-window__light--max" />
        </span>

        <span className="mac-window__title">{title}</span>

        {toolbar && <span className="mac-window__toolbar">{toolbar}</span>}
      </header>

      <div className="mac-window__body">{children}</div>
    </section>
  );
}
