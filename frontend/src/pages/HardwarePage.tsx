import { useCallback, useEffect, useState } from 'react';

import { AccountSelect } from '../components/AccountSelect';
import { Drawer } from '../components/hardware/Drawer';
import { KitShop } from '../components/hardware/KitShop';
import { MachineShop } from '../components/hardware/MachineShop';
import { MachineCard } from '../components/hardware/MachineCard';
import { Shop } from '../components/hardware/Shop';
import { Loading } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import {
  financesApi,
  hardwareApi,
  type Component,
  type Kit,
  type MachineDetail,
  type MachineQuote,
  type Part,
  type UnknownRow,
} from '../utils/api';
import { formatMoney } from '../utils/format';
import { useStatusMessage } from '../utils/useStatusMessage';

export function HardwarePage() {
  const status = useStatusMessage();

  const [machines, setMachines] = useState<MachineDetail[]>();
  const [components, setComponents] = useState<Component[]>([]);
  const [parts, setParts] = useState<Part[]>([]);
  const [kits, setKits] = useState<Kit[]>([]);
  const [quote, setQuote] = useState<MachineQuote | null>(null);
  const [accounts, setAccounts] = useState<UnknownRow[]>([]);
  const [account, setAccount] = useState('');
  const [busy, setBusy] = useState(false);

  const reload = useCallback(async () => {
    const [detail, owned, next] = await Promise.all([
      hardwareApi.listMachineDetail(),
      hardwareApi.listComponents(),
      hardwareApi.machineQuote(),
    ]);

    setMachines(detail);
    setComponents(owned);
    setQuote(next);
  }, []);

  const loadAccounts = useCallback(() => {
    financesApi.listAccounts().then(setAccounts).catch(() => {});
  }, []);

  useEffect(() => {
    reload().catch((error) => status.reportError(error));
    loadAccounts();
    hardwareApi.listShopParts().then(setParts).catch(() => {});
    hardwareApi.listKits().then(setKits).catch(() => {});
  }, [reload, loadAccounts, status]);

  const run = async (work: () => Promise<string>) => {
    setBusy(true);

    try {
      status.report(await work());
      await reload();
    } catch (error) {
      status.reportError(error);
    } finally {
      setBusy(false);
    }
  };

  const install = (component: Component, machine: number) =>
    void run(async () => {
      await hardwareApi.installComponent(component.id, machine);
      return `${component.name} instalada.`;
    });

  const uninstall = (component: Component) =>
    void run(async () => {
      await hardwareApi.uninstallComponent(component.id);
      return `${component.name} al cajón.`;
    });

  const buy = (part: Part) => {
    if (!account) {
      status.report('Elige antes la cuenta que paga.');
      return;
    }

    void run(async () => {
      const bought = await hardwareApi.buyPart(part.id, Number(account));
      loadAccounts();
      return `${bought.name} comprada por ${formatMoney(bought.paid)}. Está en el cajón.`;
    });
  };

  const rename = (machine: number, name: string) =>
    void run(async () => {
      await hardwareApi.renameMachine(machine, name);
      return `El equipo se llama ahora ${name}.`;
    });

  const scrap = (machine: number, name: string) =>
    void run(async () => {
      await hardwareApi.scrapMachine(machine);
      return `${name} dado de baja.`;
    });

  const buyMachine = (name: string) => {
    if (!account) {
      status.report('Elige antes la cuenta que paga.');
      return;
    }

    void run(async () => {
      const bought = await hardwareApi.buyMachine(Number(account), name);
      loadAccounts();
      return `${bought.name} montado por ${formatMoney(bought.paid)}. Nace vacío.`;
    });
  };

  const buyKit = (kit: Kit) => {
    if (!account) {
      status.report('Elige antes la cuenta que paga.');
      return;
    }

    void run(async () => {
      const bought = await hardwareApi.buyKit(kit.id, Number(account));
      loadAccounts();
      return `${bought.kit} montado en ${bought.machine_name} por ${formatMoney(bought.paid)}.`;
    });
  };

  if (!machines) {
    return <Loading />;
  }

  const drawer = components.filter((part) => part.installed_in === null);

  return (
    <>
      <h2>Hardware</h2>
      <StatusMessage text={status.message} as="block" />

      {machines.map((detail) => (
        <MachineCard
          key={detail.machine.id}
          detail={detail}
          only={machines.length === 1}
          busy={busy}
          onUninstall={uninstall}
          onRename={rename}
          onScrap={scrap}
        />
      ))}

      <h3>Cajón</h3>
      <Drawer components={drawer} machines={machines} busy={busy} onInstall={install} />

      <h3>Tienda</h3>
      <AccountSelect accounts={accounts} value={account} onChange={setAccount} />
      <MachineShop quote={quote} busy={busy} onBuy={buyMachine} />
      <KitShop kits={kits} busy={busy} onBuy={buyKit} />
      <Shop parts={parts} busy={busy} onBuy={buy} />
    </>
  );
}
