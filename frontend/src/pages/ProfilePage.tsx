import { tr, useIdioma } from '../i18n/text';
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
  useIdioma();
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
      <h2>{tr("Perfil")}</h2>

      <IdentityCard profile={profile} clan={clan} />

      <BiographyCard biography={profile.biography} onSaved={load} />

      <SiteCard profile={profile} onSaved={load} />

      <div className="profile__columns">
        <StatList
          title={tr("números")}
          stats={[
            { label: tr("puesto"), value: profile.rank < 0 ? tr("sin clasificar") : `#${profile.rank}` },
            { label: tr("experiencia"), value: formatCount(profile.exp), accent: true },
            { label: tr("entradas"), value: formatCount(profile.hacks) },
            { label: tr("academia"), value: formatCount(certifications) },
          ]}
        />

        <StatList
          title={tr("cuenta")}
          stats={[
            { label: tr("identificador"), value: `#${profile.id}` },
            { label: tr("desde"), value: formatDate(profile.date_joined) },
            { label: tr("gastado en piezas"), value: formatMoney(profile.money_hardware) },
          ]}
        />
      </div>

      {machine && (
        <StatList
          title={tr("tu equipo")}
          stats={[
            { label: tr("procesador"), value: formatCount(machine.cpu) },
            { label: tr("memoria"), value: formatCount(machine.ram) },
            { label: tr("disco"), value: `${formatCount(machine.hdd)} MB` },
            { label: tr("red"), value: formatCount(machine.net) },
          ]}
        />
      )}
    </>
  );
}
