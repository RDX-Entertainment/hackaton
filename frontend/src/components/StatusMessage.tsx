type StatusMessageProps = {
  text: string;
  /** Inline sits inside a form row; block stands on its own line. */
  as?: 'inline' | 'block';
};

/** The one line of feedback under a form. Renders nothing when there is none. */
export function StatusMessage({ text, as = 'inline' }: StatusMessageProps) {
  if (!text) {
    return null;
  }

  if (as === 'block') {
    return <p className="muted">{text}</p>;
  }

  return <span className="muted">{text}</span>;
}
