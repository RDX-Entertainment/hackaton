export type Stat = {
  label: string;
  value: string;
  /** Draws the figure in the accent colour; for the one that matters most. */
  accent?: boolean;
};

type StatListProps = {
  title: string;
  stats: Stat[];
};

/**
 * Key/value rows with dot leaders, the way neofetch and htop lay out a report.
 * A grid of boxed tiles was the wrong shape for numbers this small: most of
 * them are a single digit, and the box was ten times the size of its contents.
 */
export function StatList({ title, stats }: StatListProps) {
  return (
    <section className="panel">
      <header className="panel__head">
        <span className="panel__path">{title}</span>
      </header>

      <dl className="statlist">
        {stats.map((stat) => (
          <div className="statlist__row" key={stat.label}>
            <dt className="statlist__label">{stat.label}</dt>
            <span className="statlist__leader" aria-hidden="true" />
            <dd className={stat.accent ? 'statlist__value is-accent' : 'statlist__value'}>
              {stat.value}
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
