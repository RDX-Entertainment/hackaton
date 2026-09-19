import { tr, useIdioma } from '../../i18n/text';
import { useState } from 'react';

import type { Clan, Profile } from '../../utils/api';
import { daysSince } from '../../utils/format';
import { formatIp, maskIp } from '../../utils/ip';

type IdentityCardProps = {
  profile: Profile;
  clan: Clan | null;
};

/**
 * Who you are, written the way a shell would: a prompt line rather than a
 * banner. The address starts masked — this is the page most likely to end up
 * in a screenshot, and revealing it is one click.
 */
export function IdentityCard({ profile, clan }: IdentityCardProps) {
  useIdioma();
  const [revealed, setRevealed] = useState(false);
  const days = daysSince(profile.date_joined);

  return (
    <section className="identity">
      <p className="identity__prompt">
        <span className="identity__sigil">$</span> whoami
      </p>

      <div className="identity__row">
        <span className="identity__avatar" aria-hidden="true">
          {profile.login.slice(0, 2).toUpperCase()}
        </span>

        <div className="identity__who">
          <h3 className="identity__login">{profile.login}</h3>
          <p className="identity__meta">
            {clan ? (
              <span className="identity__clan">
                [{clan.nick}] {clan.name}
              </span>
            ) : (
              <span className="identity__nobody">{tr("sin clan")}</span>
            )}
            {days !== null && <span className="identity__days"> · {days} {tr("días dentro")}</span>}
          </p>
        </div>

        <span className="identity__address">
          <code className="identity__ip">
            {revealed ? formatIp(profile.game_ip) : maskIp(profile.game_ip)}
          </code>
          <button
            type="button"
            className="identity__reveal"
            aria-pressed={revealed}
            aria-label={revealed ? tr("Ocultar la dirección") : tr("Mostrar la dirección")}
            onClick={() => setRevealed((current) => !current)}
          >
            {revealed ? tr("ocultar") : tr("mostrar")}
          </button>
        </span>
      </div>
    </section>
  );
}
