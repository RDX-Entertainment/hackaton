import { useCallback, useEffect, useState } from 'react';

import { BiographyCard } from '../components/profile/BiographyCard';
import { IdentityCard } from '../components/profile/IdentityCard';
import { SiteCard } from '../components/profile/SiteCard';
import { StatList } from '../components/profile/StatList';
import { Loading } from '../components/Notice';
import {
  accountApi,
  certificationsApi,
  clansApi,
  hardwareApi,
  type Clan,
  type Machine,
  type Profile,
} from '../utils/api';
import { formatCount, formatDate, formatMoney } from '../utils/format';

export function ProfilePage() {
  const [profile, setProfile] = useState<Profile>();
  const [clan, setClan] = useState<Clan | null>(null);
  const [machine, setMachine] = useState<Machine>();
  const [certifications, setCertifications] = useState(0);

  const load = useCallback(() => {
    accountApi.getProfile().then(setProfile);
  }, []);

  useEffect(() => {
    load();

    // The rest of the page comes from endpoints the game already exposes, so
    // none of this needed a new one. Each failure is silent: having no clan
    // should not take the whole page down.
    clansApi.getMyClan().then(setClan).catch(() => setClan(null));
    hardwareApi.listMachines().then((machines) => setMachine(machines[0])).catch(() => {});
    certificationsApi
      .getCertifications()
      .then((all) => setCertifications(all.items.filter((item) => item.completed).length))
      .catch(() => {});
  }, [load]);

  if (!profile) {
    return <Loading />;
  }

  return (
    <>
      <h2>Profile</h2>

      <IdentityCard profile={profile} clan={clan} />

      <BiographyCard biography={profile.biography} onSaved={load} />

      <SiteCard profile={profile} onSaved={load} />

      <div className="profile__columns">
        <StatList
          title="stats"
          stats={[
            { label: 'rank', value: profile.rank < 0 ? 'sin clasificar' : `#${profile.rank}` },
            { label: 'exp', value: formatCount(profile.exp), accent: true },
            { label: 'hacks', value: formatCount(profile.hacks) },
            { label: 'certs', value: formatCount(certifications) },
          ]}
        />

        <StatList
          title="account"
          stats={[
            { label: 'uid', value: `#${profile.id}` },
            { label: 'joined', value: formatDate(profile.date_joined) },
            { label: 'hardware', value: formatMoney(profile.money_hardware) },
          ]}
        />
      </div>

      {machine && (
        <StatList
          title="lshw"
          stats={[
            { label: 'cpu', value: formatCount(machine.cpu) },
            { label: 'ram', value: formatCount(machine.ram) },
            { label: 'hdd', value: `${formatCount(machine.hdd)} MB` },
            { label: 'net', value: formatCount(machine.net) },
          ]}
        />
      )}
    </>
  );
}
