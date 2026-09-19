import { tr, useIdioma } from '../i18n/text';
import { useCallback, useEffect, useState } from 'react';

import { useAppDispatch, useAppSelector } from '../app/hooks';
import { authenticated } from '../app/authSlice';
import { AccountSelect } from '../components/AccountSelect';
import { Drawer } from '../components/hardware/Drawer';
import { CompatibleParts } from '../components/hardware/CompatibleParts';
import { GatewayControl } from '../components/hardware/GatewayControl';
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
  authApi,
  hardwareApi,
  type Component,
  type ComponentKind,
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
  useIdioma();
  const status = useStatusMessage();
  const dispatch = useAppDispatch();
  const session = useAppSelector((state) => state.auth);
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
  const [compatible, setCompatible] = useState<{ machine: number; kind: ComponentKind | null } | null>(null);

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
      return tr("{0} instalada.", component.name);
    });

  const uninstall = (component: Component) =>
    void run(async () => {
      await hardwareApi.uninstallComponent(component.id);
      return tr("{0} al cajón.", component.name);
    });

  const buy = (part: Part, machine?: number) => {
    if (!account) {
      status.report(tr("Elige antes la cuenta que paga."));
      return;
    }

    void run(async () => {
      const bought = await hardwareApi.buyPart(part.id, Number(account), machine);
      loadAccounts();
      return tr("{0} comprada por {1}. {2}", bought.name, formatMoney(bought.paid), machine ? tr("Instalada en el equipo.") : tr("Está en el cajón."));
    });
  };

  const rename = (machine: number, name: string) =>
    void run(async () => {
      await hardwareApi.renameMachine(machine, name);
      return tr("El equipo se llama ahora {0}.", name);
    });

  const setGateway = (machine: number) =>
    void run(async () => {
      await hardwareApi.setGateway(machine);
      if (session.token) {
        const user = await authApi.me();
        dispatch(authenticated({ token: session.token, user }));
      }
      return tr("Máquina principal actualizada. Ya puedes retirar la última tarjeta de red de la anterior.");
    });

  const buySwitch = () => void run(async () => {
    const bought = await hardwareApi.buySwitch(Number(account));
    loadAccounts(); setSelected(bought.machine);
    return tr("Switch LAN de 8 puertos añadido al mapa por {0}.", formatMoney(bought.paid));
  });

  const scrap = (machine: number, name: string) =>
    void run(async () => {
      await hardwareApi.scrapMachine(machine);
      return tr("{0} dado de baja.", name);
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
      return tr("Cable tendido.");
    });

  const unlink = (cable: MachineLink) =>
    void run(async () => {
      await hardwareApi.deleteLink(cable.id);
      return tr("Cable retirado.");
    });

  const buyMachine = (name: string) => {
    if (!account) {
      status.report(tr("Elige antes la cuenta que paga."));
      return;
    }

    void run(async () => {
      const bought = await hardwareApi.buyMachine(Number(account), name);
      loadAccounts();
      return tr("{0} montado por {1}. Nace vacío.", bought.name, formatMoney(bought.paid));
    });
  };

  const buyKit = (kit: Kit) => {
    if (!account) {
      status.report(tr("Elige antes la cuenta que paga."));
      return;
    }

    void run(async () => {
      const bought = await hardwareApi.buyKit(kit.id, Number(account));
      loadAccounts();
      return tr("{0} montado en {1} por {2}.", bought.kit, bought.machine_name, formatMoney(bought.paid));
    });
  };

  if (!machines) {
    return <Loading />;
  }

  const drawer = components.filter((part) => part.installed_in === null);
  const actual = machines.find((detail) => detail.machine.id === selected) ?? machines[0];
  const compatibleMachine = machines.find((detail) => detail.machine.id === compatible?.machine);

  return (
    <>
      <h2>Hardware</h2>
      <StatusMessage text={status.message} as="block" />

      <MachineSelector
        machines={machines.map((detail) => detail.machine)}
        selected={selected}
        onSelect={setSelected}
      />

      {actual && actual.machine.node_kind !== 'switch' && (
        <GatewayControl detail={actual} machines={machines} busy={busy} onChange={setGateway} />
      )}

      {actual && actual.machine.node_kind !== 'switch' && (
        <MachineCard
          key={actual.machine.id}
          detail={actual}
          only={machines.filter((one) => one.machine.node_kind !== 'switch').length === 1}
          busy={busy}
          onUninstall={uninstall}
          onRename={rename}
          onScrap={scrap}
          onAdd={(kind) => setCompatible({ machine: actual.machine.id, kind })}
        />
      )}

      {actual?.machine.node_kind === 'switch' && <section className="panel machine__readiness">
        <h3>{actual.machine.name} · Switch LAN</h3><p>{actual.machine.switch_ports} {tr("puertos · Activo · no consume una plaza de cómputo.")}</p>
        <button disabled={busy} onClick={() => scrap(actual.machine.id, actual.machine.name)}>{tr("Retirar switch y sus cables")}</button>
      </section>}

      {compatible && compatibleMachine && <CompatibleParts detail={compatibleMachine} owned={components} parts={parts}
        busy={busy} account={account} kind={compatible.kind} onClose={() => setCompatible(null)} onInstall={install} onBuy={buy} />}

      <h3>{tr("Red")}</h3>
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

      <h3>{tr("Cajón")}</h3>
      <Drawer components={drawer} machines={machines.filter((one) => one.machine.node_kind !== 'switch')} busy={busy} onInstall={install} />

      <h3>{tr("Tienda")}</h3>
      <AccountSelect accounts={accounts} value={account} onChange={setAccount} />
      <MachineShop quote={quote} busy={busy} onBuy={buyMachine} />
      <KitShop kits={kits} busy={busy} onBuy={buyKit} />
      <Shop parts={parts} busy={busy} onBuy={buy} canBuyExtras={Boolean(account)} onBuySwitch={buySwitch} />
    </>
  );
}
