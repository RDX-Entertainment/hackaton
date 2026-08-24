import type { ReactNode } from 'react';

type AdminCardProps = {
  prefix: string;
  brand: string;
  note?: string;
  accent?: 'green' | 'lime';
  children: ReactNode;
};

export function AdminCard({ prefix, brand, note, accent = 'green', children }: AdminCardProps) {
  return (
    <section className={`adcard adcard--${accent}`}>
      <header className="adcard__head">
        <span className="adcard__brand">
          {prefix}
          <b>{brand}</b> <small>C.A.</small>
        </span>

        {note && <span className="adcard__note">{note}</span>}
      </header>

      {children}
    </section>
  );
}
