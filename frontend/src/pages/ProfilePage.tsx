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
      <h2>Perfil</h2>

      <IdentityCard profile={profile} clan={clan} />

      <BiographyCard biography={profile.biography} onSaved={load} />

      <SiteCard profile={profile} onSaved={load} />

      <div className="profile__columns">
        <StatList
          title="números"
          stats={[
            { label: 'puesto', value: profile.rank < 0 ? 'sin clasificar' : `#${profile.rank}` },
            { label: 'experiencia', value: formatCount(profile.exp), accent: true },
            { label: 'entradas', value: formatCount(profile.hacks) },
            { label: 'academia', value: formatCount(certifications) },
          ]}
        />

        <StatList
          title="cuenta"
          stats={[
            { label: 'identificador', value: `#${profile.id}` },
            { label: 'desde', value: formatDate(profile.date_joined) },
            { label: 'gastado en piezas', value: formatMoney(profile.money_hardware) },
          ]}
        />
      </div>

      {machine && (
        <StatList
          title="tu equipo"
          stats={[
            { label: 'procesador', value: formatCount(machine.cpu) },
            { label: 'memoria', value: formatCount(machine.ram) },
            { label: 'disco', value: `${formatCount(machine.hdd)} MB` },
            { label: 'red', value: formatCount(machine.net) },
          ]}
        />
      )}
    </>
  );
}
