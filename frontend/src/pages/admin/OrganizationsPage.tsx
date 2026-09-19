import { tr, useIdioma, idiomaActual } from '../../i18n/text';
import { FormEvent, useCallback, useEffect, useMemo, useState } from 'react';

import { AccountsBoard } from '../../components/admin/AccountsBoard';
import { AdminCard } from '../../components/admin/AdminCard';
import { AdminLoadError } from '../../components/admin/AdminLoadError';
import { ComputerEditorDialog } from '../../components/admin/ComputerEditorDialog';
import { OrganizationNodeMap } from '../../components/admin/OrganizationNodeMap';
import { SiteEditorDialog } from '../../components/admin/SiteEditorDialog';
import { Loading } from '../../components/Notice';
import { StatusMessage } from '../../components/StatusMessage';
import {
  adminApi,
  type BankAccount,
  type CreateOrganizationServerInput,
  type Kit,
  type NetworkZone,
  type Organization,
  type OrganizationBehavior,
  type OrganizationInput,
  type OrganizationNetwork,
  type OrganizationNetworkInput,
  type OrganizationNetworkNpc,
  type OrganizationNpcCatalog,
  type Site,
  type Part,
  type OrganizationComputerConfiguration,
} from '../../utils/api';
import { formatMoney } from '../../utils/format';
import { formatIp } from '../../utils/ip';
import { useStatusMessage } from '../../utils/useStatusMessage';

const ZONES: { value: NetworkZone; label: string; hint: string }[] = [
  { value: 'public', get label() { return tr("Pública"); }, get hint() { return tr("visible desde Internet"); } },
  { value: 'dmz', label: 'DMZ', get hint() { return tr("perímetro de la organización"); } },
  { value: 'internal', get label() { return tr("Interna"); }, get hint() { return tr("reservada para red privada"); } },
  { value: 'restricted', get label() { return tr("Restringida"); }, get hint() { return tr("activos especialmente sensibles"); } },
];

const emptyOrganization = (): OrganizationInput => ({
  slug: '', name: '', category: '', description: '', active: true, behaviors: [],
});

/**
 * El comportamiento que convierte a una organización en banco. No es una etiqueta: la clave
 * ajena de `bank_accounts` lo exige para dejar abrir una cuenta, y lo protege mientras haya
 * alguna abierta.
 */
const BANKING = 'banking';

function isBank(behaviors: OrganizationBehavior[]) {
  return behaviors.some((behavior) => behavior.code === BANKING);
}

function withBanking(behaviors: OrganizationBehavior[], wanted: boolean): OrganizationBehavior[] {
  if (wanted === isBank(behaviors)) return behaviors;
  return wanted
    ? [...behaviors, { code: BANKING, config: {} }]
    : behaviors.filter((behavior) => behavior.code !== BANKING);
}

/** Categories and slugs are stable identifiers, so the UI writes the server format for the admin. */
function toCode(value: string) {
  return value
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function behaviorsOrEmpty(text: string): OrganizationBehavior[] {
  try {
    return behaviorFromText(text);
  } catch {
    return [];
  }
}

function behaviorText(behaviors: OrganizationBehavior[]) {
  return behaviors.length ? JSON.stringify(behaviors, null, 2) : '[]';
}

function behaviorFromText(text: string): OrganizationBehavior[] {
  const value: unknown = JSON.parse(text || '[]');
  if (!Array.isArray(value)) throw new Error(tr("Los comportamientos deben ser una lista JSON."));
  return value.map((item) => {
    if (!item || typeof item !== 'object' || typeof (item as { code?: unknown }).code !== 'string') {
      throw new Error(tr("Cada comportamiento requiere un campo \"code\"."));
    }
    const config = (item as { config?: unknown }).config;
    if (config !== undefined && (!config || typeof config !== 'object' || Array.isArray(config))) {
      throw new Error(tr("La configuración de cada comportamiento debe ser un objeto JSON."));
    }
    return { code: (item as { code: string }).code, config: (config ?? {}) as Record<string, unknown> };
  });
}

export function OrganizationsPage() {
  useIdioma();
  const status = useStatusMessage();
  const [organizations, setOrganizations] = useState<Organization[]>();
  const [kits, setKits] = useState<Kit[]>([]);
  const [parts, setParts] = useState<Part[]>([]);
  const [npcs, setNpcs] = useState<OrganizationNpcCatalog[]>([]);
  const [sites, setSites] = useState<Site[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [form, setForm] = useState<OrganizationInput>(emptyOrganization());
  const [behaviorJson, setBehaviorJson] = useState('[]');
  const [busy, setBusy] = useState(false);
  const [loadError, setLoadError] = useState<unknown>(null);

  const load = useCallback(async () => {
    setLoadError(null);
    try {
      const [organizationRows, kitRows, npcRows, siteRows, partRows] = await Promise.all([
        adminApi.listOrganizations(), adminApi.listKits(), adminApi.listOrganizationNpcs(), adminApi.listSites(), adminApi.listParts(),
      ]);
      setOrganizations(organizationRows);
      setKits(kitRows);
      setNpcs(npcRows);
      setSites(siteRows);
      setParts(partRows);
      setSelectedId((current) => current ?? organizationRows[0]?.id ?? null);
    } catch (error) { setLoadError(error); }
  }, []);

  useEffect(() => { void load(); }, [load]);

  const selected = useMemo(
    () => organizations?.find((organization) => organization.id === selectedId) ?? null,
    [organizations, selectedId, idiomaActual()],
  );

  const choose = (organization: Organization) => {
    setSelectedId(organization.id);
    setForm({
      slug: organization.slug, name: organization.name, category: organization.category,
      description: organization.description, active: organization.active, behaviors: organization.behaviors,
    });
    setBehaviorJson(behaviorText(organization.behaviors));
  };

  const newOrganization = () => {
    setSelectedId(null);
    setForm(emptyOrganization());
    setBehaviorJson('[]');
  };

  const run = async (work: () => Promise<string>) => {
    setBusy(true);
    try { status.report(await work()); await load(); }
    catch (error) { status.reportError(error); }
    finally { setBusy(false); }
  };

  const saveOrganization = (event: FormEvent) => {
    event.preventDefault();
    void run(async () => {
      const input = { ...form, behaviors: behaviorFromText(behaviorJson) };
      if (selected) {
        await adminApi.editOrganization(selected.id, input);
        return tr("{0} actualizada.", input.name);
      }
      const created = await adminApi.createOrganization(input);
      setSelectedId(created.id);
      return tr("{0} creada. Ahora añade su primera red.", input.name);
    });
  };

  if (loadError) return <AdminLoadError title={tr("Organizaciones")} error={loadError} onRetry={() => void load()} />;
  if (!organizations) return <Loading />;

  return (
    <>
      <h2>{tr("Organizaciones")}</h2>
      <p className="muted">{tr("Define las instituciones del juego y compón su infraestructura sin salir del panel.")}</p>
      <StatusMessage text={status.message} as="block" />

      <div className="org-layout">
        <AdminCard prefix="01/" brand="organización" note={selected ? tr("editar entidad") : tr("nueva entidad")}>
          <form className="org-form" onSubmit={saveOrganization}>
            <div className="org-form__grid">
              <label>{tr("Nombre")}<input required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder={tr("Banco Central")} /></label>
              <label>Slug<input required pattern="[a-z0-9-]+" value={form.slug} onChange={(e) => setForm({ ...form, slug: toCode(e.target.value) })} placeholder="banco-central" /></label>
              <label>{tr("Categoría")}<input required pattern="[a-z0-9-]+" value={form.category} onChange={(e) => setForm({ ...form, category: toCode(e.target.value) })} placeholder="finanzas" /><small>{tr("Se normaliza automáticamente: “Sector Público” → “sector-publico”.")}</small></label>
              <label className="org-check"><input type="checkbox" checked={form.active} onChange={(e) => setForm({ ...form, active: e.target.checked })} /> {tr("Activa en el juego")}</label>
              <label className="org-check"><input type="checkbox" checked={isBank(behaviorsOrEmpty(behaviorJson))} onChange={(e) => setBehaviorJson(behaviorText(withBanking(behaviorsOrEmpty(behaviorJson), e.target.checked)))} /> {tr("Ofrece cuentas bancarias")}</label>
            </div>
            <label>{tr("Descripción")}<textarea value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} placeholder={tr("Qué representa esta institución y cuál es su función.")} /></label>
            <details className="org-behaviors">
              <summary>{tr("Comportamientos")} <small>{tr("avanzado · JSON declarativo")}</small></summary>
              <textarea value={behaviorJson} onChange={(e) => setBehaviorJson(e.target.value)} spellCheck={false} />
              <p>{tr("Ejemplo:")} <code>[{'{'}"code":"banking","config":{'{}'}{'}'}]</code>{tr(". Se almacenan para que las acciones web futuras los interpreten.")}</p>
            </details>
            <div className="org-actions"><button type="button" onClick={newOrganization} disabled={busy}>{tr("Nueva")}</button><button className="is-primary" disabled={busy}>{selected ? tr("Guardar cambios") : tr("Crear organización")}</button></div>
          </form>
        </AdminCard>

        <AdminCard prefix="map/" brand="instituciones" note={tr("{0} registradas", organizations.length)} accent="lime">
          <div className="org-list">
            {organizations.map((organization) => <button type="button" key={organization.id} className={`org-list__item ${organization.id === selectedId ? 'is-active' : ''}`} onClick={() => choose(organization)}>
              <span><b>{organization.name}</b><small>{organization.category} · {organization.networks.length} {tr("redes")}</small></span><em>{organization.active ? 'activa' : tr("pausada")}</em>
            </button>)}
            {!organizations.length && <p className="org-empty">{tr("Crea la primera organización para comenzar.")}</p>}
          </div>
        </AdminCard>
      </div>

      {selected && <Accounts organization={selected} busy={busy} onRun={run} />}
      {selected && <Infrastructure organization={selected} kits={kits} parts={parts} npcs={npcs} sites={sites} busy={busy} onRun={run} />}
    </>
  );
}

/**
 * Las cuentas que esta organización tiene abiertas, y el alta de otra. Una organización que
 * no sea un banco también tiene dinero: el FBI cobra en Pinance sin ofrecer cuentas a nadie.
 */
function Accounts({ organization, busy, onRun }: { organization: Organization; busy: boolean; onRun: (work: () => Promise<string>) => Promise<void> }) {
  useIdioma();
  const [accounts, setAccounts] = useState<BankAccount[]>([]);

  const load = useCallback(() => {
    adminApi.listOrganizationAccounts(organization.id).then(setAccounts).catch(() => setAccounts([]));
  }, [organization.id]);

  useEffect(() => { load(); }, [load]);

  return <AdminCard prefix="03/" brand="cuentas" note={tr("{0} abiertas", accounts.length)} accent="lime">
    <p className="org-infra__intro">{tr("Un banco es una organización con «Ofrece cuentas bancarias» marcado. Cualquier organización puede tener cuenta en uno.")}</p>
    <AccountsBoard
      accounts={accounts}
      busy={busy}
      emptyMessage="Esta organización no tiene ninguna cuenta."
      onOpen={(bankId, pin) => void onRun(async () => {
        const opened = await adminApi.openOrganizationAccount(organization.id, { bank_id: bankId, password: pin });
        load();
        return tr("Cuenta {0} abierta a nombre de {1}.", opened.account, organization.name);
      })}
      onAdjust={(account, amount) => void onRun(async () => {
        const { cash } = await adminApi.adjustAccountCash(account.id, amount);
        load();
        const verbo = amount > 0 ? tr("Ingresados") : tr("Retirados");
        return tr("{0} {1} en {2}. Saldo: {3}.", verbo, formatMoney(Math.abs(amount)), account.account, formatMoney(cash));
      })}
    />
  </AdminCard>;
}

type InfrastructureProps = {
  organization: Organization; kits: Kit[]; parts: Part[]; npcs: OrganizationNpcCatalog[]; sites: Site[]; busy: boolean;
  onRun: (work: () => Promise<string>) => Promise<void>;
};

function Infrastructure(props: InfrastructureProps) {
  useIdioma();
  const { organization, kits, parts, sites, busy, onRun } = props;
  const [selectedNodeId, setSelectedNodeId] = useState<number | null>(null);
  const [connectionSource, setConnectionSource] = useState<{ network: OrganizationNetwork; npc: OrganizationNetworkNpc } | null>(null);
  const selectedNode = useMemo(() => {
    if (selectedNodeId === null) return null;
    for (const network of organization.networks) {
      const npc = network.npcs.find((candidate) => candidate.id === selectedNodeId);
      if (npc) return { network, npc };
    }
    return null;
  }, [organization.networks, selectedNodeId, idiomaActual()]);
  return <AdminCard prefix="02/" brand="infraestructura" note={organization.name} accent="lime">
    <div className="org-infra">
      <p className="org-infra__intro">{tr("Una red es interna por defecto. Un nodo pasa a Internet al instalarle una interfaz de red desde Computer.")}</p>
      <OrganizationNodeMap organization={organization} kits={kits} busy={busy} onSelectNode={({ npc }) => setSelectedNodeId(npc.id)} onRun={onRun} />
      <div className="org-networks">
      {selectedNode && <NodeInspector organization={organization} network={selectedNode.network} npc={selectedNode.npc} kits={kits} parts={parts} sites={sites} busy={busy} connectionSource={connectionSource} setConnectionSource={setConnectionSource} onRun={onRun} onRemoved={() => setSelectedNodeId(null)} />}
        {!organization.networks.length && <p className="org-empty">{tr("Todavía no hay redes. Empieza por una red pública si el servicio tendrá web.")}</p>}
        {organization.networks.map((network) => <NetworkRow key={network.id} organization={organization} network={network} busy={busy} onRun={onRun} />)}
        <CreateNetworkForm organization={organization} busy={busy} onRun={onRun} />
      </div>
    </div>
  </AdminCard>;
}

/** Borrar no se deshace: el primer clic arma el boton y el segundo ejecuta. Se desarma solo. */
function DangerButton({ label, busy, onConfirm }: { label: string; busy: boolean; onConfirm: () => void }) {
  useIdioma();
  const [armed, setArmed] = useState(false);
  useEffect(() => {
    if (!armed) return;
    const timer = setTimeout(() => setArmed(false), 4000);
    return () => clearTimeout(timer);
  }, [armed]);
  return <button type="button" className={armed ? 'org-danger is-armed' : 'org-danger'} disabled={busy} onClick={() => { if (armed) { setArmed(false); onConfirm(); } else { setArmed(true); } }}>{armed ? tr("¿Seguro?") : label}</button>;
}

/** Renombrar una red, cambiarle la zona o eliminarla. El servidor la protege mientras tenga nodos. */
function NetworkRow({ organization, network, busy, onRun }: { organization: Organization; network: OrganizationNetwork; busy: boolean; onRun: (work: () => Promise<string>) => Promise<void> }) {
  useIdioma();
  const [name, setName] = useState(network.name);
  const [zone, setZone] = useState<NetworkZone>(network.zone);
  // La recarga tras guardar trae una red nueva: sin esto el formulario seguiria mostrando lo viejo.
  useEffect(() => { setName(network.name); setZone(network.zone); }, [network]);
  const dirty = name.trim() !== network.name || zone !== network.zone;
  const save = (event: FormEvent) => {
    event.preventDefault();
    void onRun(async () => {
      await adminApi.editOrganizationNetwork(organization.id, network.id, { name: name.trim(), zone });
      return tr("Red {0} actualizada.", name.trim());
    });
  };
  const remove = () => void onRun(async () => {
    await adminApi.removeOrganizationNetwork(organization.id, network.id);
    return tr("Red {0} eliminada.", network.name);
  });
  return <form className="org-network-row" onSubmit={save}><input required value={name} onChange={(event) => setName(event.target.value)} /><select value={zone} onChange={(event) => setZone(event.target.value as NetworkZone)}>{ZONES.map((item) => <option value={item.value} key={item.value}>{item.label}</option>)}</select><small>{network.npcs.length} {network.npcs.length === 1 ? 'nodo' : 'nodos'}</small><button disabled={busy || !dirty || !name.trim()}>{tr("Guardar")}</button><DangerButton label={tr("Eliminar")} busy={busy} onConfirm={remove} /></form>;
}

/** Sin una red no hay donde colgar un nodo: el desplegable del mapa se queda vacio y no deja crear. */
function CreateNetworkForm({ organization, busy, onRun }: { organization: Organization; busy: boolean; onRun: (work: () => Promise<string>) => Promise<void> }) {
  useIdioma();
  const [draft, setDraft] = useState<OrganizationNetworkInput>({ name: '', zone: 'internal' });
  const create = (event: FormEvent) => {
    event.preventDefault();
    void onRun(async () => {
      await adminApi.createOrganizationNetwork(organization.id, draft);
      setDraft({ name: '', zone: 'internal' });
      return tr("Red {0} creada.", draft.name);
    });
  };
  return <form className="org-network-create" onSubmit={create}><b>{tr("Crear red")}</b><input required value={draft.name} placeholder={tr("Nombre de la red")} onChange={(event) => setDraft((current) => ({ ...current, name: event.target.value }))} /><select value={draft.zone} onChange={(event) => setDraft((current) => ({ ...current, zone: event.target.value as NetworkZone }))}>{ZONES.map((zone) => <option value={zone.value} key={zone.value}>{zone.label}</option>)}</select><button className="is-primary" disabled={busy || !draft.name.trim()}>{tr("Crear red")}</button></form>;
}

function NodeInspector({ organization, network, npc, kits, parts, sites, busy, connectionSource, setConnectionSource, onRun, onRemoved }: { organization: Organization; network: OrganizationNetwork; npc: OrganizationNetworkNpc; kits: Kit[]; parts: Part[]; sites: Site[]; busy: boolean; connectionSource: { network: OrganizationNetwork; npc: OrganizationNetworkNpc } | null; setConnectionSource: (node: { network: OrganizationNetwork; npc: OrganizationNetworkNpc } | null) => void; onRun: (work: () => Promise<string>) => Promise<void>; onRemoved: () => void }) {
  useIdioma();
  const [editorOpen, setEditorOpen] = useState(false);
  const [computerOpen, setComputerOpen] = useState(false);
  const [computerConfiguration, setComputerConfiguration] = useState<OrganizationComputerConfiguration | null>(null);
  const [role, setRole] = useState(npc.role);
  // Al recargar tras guardar llega otro npc: el formulario tiene que seguir al nodo seleccionado.
  useEffect(() => { setRole(npc.role); }, [npc]);
  const nodeDirty = role.trim() !== npc.role;
  const saveNode = (event: FormEvent) => {
    event.preventDefault();
    void onRun(async () => {
      await adminApi.editOrganizationNode(organization.id, network.id, npc.id, { role: role.trim(), map_x: npc.map_x, map_y: npc.map_y });
      return tr("{0} actualizado.", npc.name);
    });
  };
  const removeNode = () => void onRun(async () => {
    await adminApi.removeOrganizationNode(organization.id, network.id, npc.id);
    onRemoved();
    return tr("{0} eliminado del mapa.", npc.name);
  });
  const site = sites.find((item) => item.kind === 'npc' && item.id === npc.id);
  const selectPort = () => { if (!connectionSource) { setConnectionSource({ network, npc }); return; } if (connectionSource.npc.id === npc.id) { setConnectionSource(null); return; } void onRun(async () => { await adminApi.createOrganizationLink(organization.id, { from_npc_id: connectionSource.npc.id, to_npc_id: npc.id }); setConnectionSource(null); return tr("Conexión LAN creada."); }); };
  const openComputer = () => void onRun(async () => { setComputerConfiguration(await adminApi.getOrganizationComputer(organization.id, network.id, npc.id)); setComputerOpen(true); return ''; });
  return <section className="org-node-inspector"><header><span className={`org-zone org-zone--${npc.zone}`}>{npc.zone}</span><h3>{npc.name}</h3></header><dl><div><dt>IP</dt><dd>{formatIp(npc.ip)}</dd></div><div><dt>{tr("Red")}</dt><dd>{network.name}</dd></div><div><dt>{tr("Rol")}</dt><dd>{npc.role}</dd></div><div><dt>{tr("Equipo")}</dt><dd>{npc.computer_components ? tr("{0} piezas · {1}", npc.computer_components, npc.source_kit_id ? `kit #${npc.source_kit_id}` : 'personalizado') : tr("Sin configurar")}</dd></div></dl><div className="org-node-inspector__ports"><small>{tr("Puertos LAN")} {connectionSource ? tr("· selecciona este puerto para conectar") : ''}</small>{Array.from({ length: Math.max(0, npc.lan_available) }).map((_, index) => <button type="button" className={connectionSource ? 'is-linkable' : ''} key={index} onClick={selectPort} aria-label={tr("Puerto LAN {0}", index + 1)} />)}{!npc.lan_available && <em>{tr("Sin puertos libres")}</em>}</div><form className="org-node-edit" onSubmit={saveNode}><label>{tr("rol")}<input value={role} maxLength={40} onChange={(event) => setRole(event.target.value)} /></label><small className="org-node-edit__hint">{npc.zone === 'public' ? tr("En Internet porque tiene una interfaz de red instalada. Quítasela en Computer y dejará de responder.") : tr("Fuera de Internet. Instálale una interfaz de red en Computer para exponerlo.")}</small><button disabled={busy || !nodeDirty || !role.trim()}>{tr("Guardar nodo")}</button><DangerButton label={tr("Eliminar nodo")} busy={busy} onConfirm={removeNode} /></form><div className="org-node-inspector__menus"><button className="is-primary" type="button" onClick={openComputer}>Computer</button><button type="button" onClick={() => setEditorOpen(true)}>Web</button></div><ComputerEditorDialog open={computerOpen} title={npc.name} kits={kits} parts={parts} configuration={computerConfiguration} lanLinks={Math.max(0, npc.lan_total - npc.lan_available - (npc.zone === 'public' ? 1 : 0))} busy={busy} onClose={() => setComputerOpen(false)} onSave={(input) => void onRun(async () => { await adminApi.saveOrganizationComputer(organization.id, network.id, npc.id, input); setComputerOpen(false); return tr("PC de {0} instalado en modo creativo.", npc.name); })} /><SiteEditorDialog open={editorOpen} title={npc.name} initialWeb={site?.web ?? ''} busy={busy} onClose={() => setEditorOpen(false)} onSave={(web) => void onRun(async () => { await adminApi.saveSite('npc', npc.id, web); setEditorOpen(false); return tr("Web de {0} guardada.", npc.name); })} /></section>;
}

function Network({ organization, network, kits, npcs, sites, busy, onRun }: { organization: Organization; network: OrganizationNetwork; kits: Kit[]; npcs: OrganizationNpcCatalog[]; sites: Site[]; busy: boolean; onRun: (work: () => Promise<string>) => Promise<void> }) {
  useIdioma();
  const [server, setServer] = useState<CreateOrganizationServerInput>({ name: '', kit_id: kits[0]?.id ?? 0, role: 'server', npc_type: 70, web: '' });
  const [npcId, setNpcId] = useState('');
  const [role, setRole] = useState('operator');
  const [networkName, setNetworkName] = useState(network.name);
  const [networkZone, setNetworkZone] = useState<NetworkZone>(network.zone);
  const [selectedNpcId, setSelectedNpcId] = useState<number | null>(network.npcs[0]?.id ?? null);
  const availableNpcs = npcs.filter((npc) => npc.network_id !== network.id);
  const selectedNpc = network.npcs.find((npc) => npc.id === selectedNpcId) ?? network.npcs[0] ?? null;
  const currentWeb = sites.find((item) => item.kind === 'npc' && item.id === selectedNpc?.id)?.web ?? '';
  const [web, setWeb] = useState(currentWeb);
  const createServer = (event: FormEvent) => { event.preventDefault(); void onRun(async () => {
    const made = await adminApi.createOrganizationServer(organization.id, network.id, server);
    setServer({ ...server, name: '', web: '' });
    return tr("Servidor creado: {0} · contraseña inicial: {1}. Guárdala ahora; no se vuelve a mostrar.", formatIp(made.ip), made.password);
  }); };
  const assignNpc = (event: FormEvent) => { event.preventDefault(); if (!npcId) return; void onRun(async () => {
    await adminApi.assignOrganizationNpc(organization.id, network.id, { npc_id: Number(npcId), role }); setNpcId(''); return tr("NPC asignado a la red.");
  }); };
  return <article className="org-network">
    <header><div><span className={`org-zone org-zone--${network.zone}`}>{ZONES.find((zone) => zone.value === network.zone)?.label}</span><h3>{network.name}</h3></div><button className="org-danger" disabled={busy} onClick={() => void onRun(async () => { await adminApi.removeOrganizationNetwork(organization.id, network.id); return tr("{0} eliminada.", network.name); })}>{tr("Eliminar red")}</button></header>
    <details className="org-network-edit"><summary>{tr("Modificar esta red")}</summary><form onSubmit={(event) => { event.preventDefault(); void onRun(async () => { await adminApi.editOrganizationNetwork(organization.id, network.id, { name: networkName, zone: networkZone }); return tr("Red {0} actualizada.", networkName); }); }}><input required value={networkName} onChange={(event) => setNetworkName(event.target.value)} /><select value={networkZone} onChange={(event) => setNetworkZone(event.target.value as NetworkZone)}>{ZONES.map((zone) => <option value={zone.value} key={zone.value}>{zone.label}</option>)}</select><button disabled={busy}>{tr("Guardar red")}</button></form></details>
    <div className="org-network__machines">
      {network.npcs.map((npc) => <button type="button" className={`org-machine ${npc.id === selectedNpc?.id ? 'is-active' : ''}`} onClick={() => { setSelectedNpcId(npc.id); setWeb(sites.find((item) => item.kind === 'npc' && item.id === npc.id)?.web ?? ''); }} key={npc.id}><b>{npc.name}</b><span>{formatIp(npc.ip)} · {npc.role}{npc.source_kit_id ? ` · kit #${npc.source_kit_id}` : ''}</span></button>)}
      {!network.npcs.length && <p className="org-empty">{tr("Sin equipos todavía.")}</p>}
    </div>
    {selectedNpc && <form className="org-web-form" onSubmit={(event) => { event.preventDefault(); void onRun(async () => { await adminApi.saveSite('npc', selectedNpc.id, web); return tr("Web de {0} guardada.", selectedNpc.name); }); }}><div><b>{tr("Web de")} {selectedNpc.name}</b><small>{formatIp(selectedNpc.ip)} {tr("· HTML que verá quien visite este servidor.")}</small></div><textarea value={web} onChange={(event) => setWeb(event.target.value)} placeholder="<h1>Bienvenido</h1>" spellCheck={false} /><button className="is-primary" disabled={busy}>{tr("Guardar nodo")}</button></form>}
    <div className="org-network__actions">
      <form className="org-server-form" onSubmit={createServer}><b>{tr("Crear servidor desde un kit")}</b><input required value={server.name} onChange={(e) => setServer({ ...server, name: e.target.value })} placeholder={tr("Gateway público")} /><select required value={server.kit_id} onChange={(e) => setServer({ ...server, kit_id: Number(e.target.value) })}><option value={0} disabled>{tr("Selecciona un kit")}</option>{kits.map((kit) => <option value={kit.id} key={kit.id}>{kit.name}</option>)}</select><input value={server.role} onChange={(e) => setServer({ ...server, role: e.target.value })} placeholder={tr("Rol")} /><textarea value={server.web} onChange={(e) => setServer({ ...server, web: e.target.value })} placeholder={tr("Web inicial opcional: <h1>Servicio</h1>")} spellCheck={false} /><button className="is-primary" disabled={busy || !kits.length}>{tr("Crear servidor")}</button>{!kits.length && <small>{tr("Antes crea un kit válido en la sección Kits.")}</small>}</form>
      <form className="org-npc-form" onSubmit={assignNpc}><b>{tr("Asignar NPC existente")}</b><select value={npcId} onChange={(e) => setNpcId(e.target.value)}><option value="">{tr("Selecciona un NPC")}</option>{availableNpcs.map((npc) => <option value={npc.id} key={npc.id}>{npc.name} · {formatIp(npc.ip)}{npc.zone ? ` · ${npc.zone}` : ''}</option>)}</select><input value={role} onChange={(e) => setRole(e.target.value)} placeholder={tr("Rol")} /><button disabled={busy || !npcId}>{tr("Asignar")}</button></form>
    </div>
  </article>;
}

// Kept while its creation controls are moved into the map toolbar.
void Network;
