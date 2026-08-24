import type { ReactNode } from 'react';

type NoticeProps = {
  children: ReactNode;
};

/** Waiting on a first load. */
export function Loading({ children = 'Loading…' }: { children?: ReactNode }) {
  return <p>{children}</p>;
}

/** A load that failed outright, as opposed to an action that was refused. */
export function ErrorNotice({ children }: NoticeProps) {
  return <p className="error">{children}</p>;
}

/** Nothing to show, but nothing went wrong either. */
export function EmptyNotice({ children }: NoticeProps) {
  return <p className="muted">{children}</p>;
}
