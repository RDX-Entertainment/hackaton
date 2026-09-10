import type { ComponentType } from 'react';

import { AccessesPage } from '../pages/AccessesPage';
import { BitcoinPage } from '../pages/BitcoinPage';
import { ClanPage } from '../pages/ClanPage';
import { FinancesPage } from '../pages/FinancesPage';
import { HardwarePage } from '../pages/HardwarePage';
import { InternetPage } from '../pages/InternetPage';
import { ListPage } from '../pages/ListPage';
import { LabPage } from '../pages/LabPage';
import { MailPage } from '../pages/MailPage';
import { MissionsPage } from '../pages/MissionsPage';
import { ProcessesPage } from '../pages/ProcessesPage';
import { ProfilePage } from '../pages/ProfilePage';
import { SettingsPage } from '../pages/SettingsPage';
import { SoftwarePage } from '../pages/SoftwarePage';
import { StatsPage } from '../pages/StatsPage';
import { UniversityPage } from '../pages/UniversityPage';

/**
 * Which legacy path has a screen of its own. Anything missing here falls back
 * to the generic resource table, so adding a screen is one line rather than
 * another branch in an if-chain.
 */
export const PAGE_REGISTRY: Record<string, ComponentType> = {
  accesses: AccessesPage,
  bitcoin: BitcoinPage,
  certs: UniversityPage,
  clan: ClanPage,
  finances: FinancesPage,
  hardware: HardwarePage,
  internet: InternetPage,
  lab: LabPage,
  list: ListPage,
  mail: MailPage,
  missions: MissionsPage,
  processes: ProcessesPage,
  profile: ProfilePage,
  settings: SettingsPage,
  software: SoftwarePage,
  stats: StatsPage,
  university: UniversityPage,
};

export function screenFor(path: string): ComponentType | undefined {
  return PAGE_REGISTRY[path];
}
