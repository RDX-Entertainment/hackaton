import { useCallback, useEffect, useState } from 'react';

import { useAppSelector } from '../app/hooks';
import { AccountSelect } from '../components/AccountSelect';
import { Drawer } from '../components/hardware/Drawer';
import { KitShop } from '../components/hardware/KitShop';
import { MachineShop } from '../components/hardware/MachineShop';
import { MachineCard } from '../components/hardware/MachineCard';
import { MachineNodeMap } from '../components/hardware/MachineNodeMap';
import { MachineSelector } from '../components/hardware/MachineSelector';
import { Shop } from '../components/hardware/Shop';
import { Loading } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import {
  financesApi,
  hardwareApi,
  type Component,
  type Kit,
  type MachineDetail,
  type MachineLink,
  type MachineQuote,
  type Part,
  type UnknownRow,
} from '../utils/api';
import { formatMoney } from '../utils/format';
import { useStatusMessage } from '../utils/useStatusMessage';

export function HardwarePage() {
  const status = useStatusMessage();
  // La direccion que se ve desde fuera. La pinta el nodo de Internet del mapa.
  const gameIp = useAppSelector((state) => state.auth.user?.game_ip);

  const [machines, setMachines] = useState<MachineDetail[]>();
  const [links, setLinks] = useState<MachineLink[]>([]);
  /**
   * El equipo que se esta viendo. 0 hasta la primera carga: entonces se elige el primero.
   * Se conserva entre recargas para que instalar una pieza no devuelva la vista al equipo 1.
   */
  const [selected, setSelected] = useState(0);
  const [components, setComponents] = useState<Component[]>([]);
  const [parts, setParts] = useState<Part[]>([]);
  const [kits, setKits] = useState<Kit[]>([]);
  const [quote, setQuote] = useState<MachineQuote | null>(null);
  const [accounts, setAccounts] = useState<UnknownRow[]>([]);
  const [account, setAccount] = useState('');
  const [busy, setBusy] = useState(false);

  const reload = useCallback(async () => {
    const [detail, owned, next, cables] = await Promise.all([
      hardwareApi.listMachineDetail(),
      hardwareApi.listComponents(),
      hardwareApi.machineQuote(),
      hardwareApi.listLinks(),
    ]);

    setMachines(detail);
    setComponents(owned);
    setQuote(next);
    setLinks(cables);

    // Si el equipo que se estaba viendo ya no existe -- se dio de baja -- la vista cae al
    // primero en vez de quedarse en blanco.
    setSelected((actual) =>
      detail.some((item) => item.machine.id === actual) ? actual : detail[0]?.machine.id ?? 0,
    );
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

  /**
   * Mover no pasa por `run`: `run` marca la pagina como ocupada y recarga entera, y
   * arrastrar un nodo no puede bloquear la interfaz ni repintarlo todo al soltar. La
   * posicion ya esta en pantalla; esto solo la deja escrita.
   */
  const move = (machine: number, x: number, y: number) => {
    setMachines((actuales) =>
      actuales?.map((item) =>
        item.machine.id === machine
          ? { ...item, machine: { ...item.machine, map_x: x, map_y: y } }
          : item,
      ),
    );

    hardwareApi.moveMachine(machine, x, y).catch((error) => status.reportError(error));
  };

  const link = (from: number, to: number) =>
    void run(async () => {
      await hardwareApi.createLink(from, to);
      return 'Cable tendido.';
    });

  const unlink = (cable: MachineLink) =>
    void run(async () => {
      await hardwareApi.deleteLink(cable.id);
      return 'Cable retirado.';
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
  const actual = machines.find((detail) => detail.machine.id === selected) ?? machines[0];

  return (
    <>
      <h2>Hardware</h2>
      <StatusMessage text={status.message} as="block" />

      <MachineSelector
        machines={machines.map((detail) => detail.machine)}
        selected={selected}
        onSelect={setSelected}
      />

      {actual && (
        <MachineCard
          key={actual.machine.id}
          detail={actual}
          only={machines.length === 1}
          busy={busy}
          onUninstall={uninstall}
          onRename={rename}
          onScrap={scrap}
        />
      )}

      <h3>Red</h3>
      <MachineNodeMap
        machines={machines}
        links={links}
        selected={selected}
        busy={busy}
        gameIp={gameIp}
        onSelect={setSelected}
        onMove={move}
        onLink={link}
        onUnlink={unlink}
      />

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
