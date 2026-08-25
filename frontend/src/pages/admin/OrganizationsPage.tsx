import { FormEvent, useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { AdminCard } from '../../components/admin/AdminCard';
import { ComputerEditorDialog } from '../../components/admin/ComputerEditorDialog';
import { OrganizationNodeMap } from '../../components/admin/OrganizationNodeMap';
import { SiteEditorDialog } from '../../components/admin/SiteEditorDialog';
import { Loading } from '../../components/Notice';
import { StatusMessage } from '../../components/StatusMessage';
import {
  adminApi,
  type CreateOrganizationServerInput,
  type Kit,
  type NetworkZone,
  type Organization,
  type OrganizationBehavior,
  type OrganizationInput,
  type OrganizationNetwork,
  type OrganizationNetworkNpc,
  type OrganizationNpcCatalog,
  type Site,
  type Part,
  type OrganizationComputerConfiguration,
} from '../../utils/api';
import { formatIp } from '../../utils/ip';
import { useStatusMessage } from '../../utils/useStatusMessage';

const ZONES: { value: NetworkZone; label: string; hint: string }[] = [
  { value: 'public', label: 'Pública', hint: 'visible desde Internet' },
  { value: 'dmz', label: 'DMZ', hint: 'perímetro de la organización' },
  { value: 'internal', label: 'Interna', hint: 'reservada para red privada' },
  { value: 'restricted', label: 'Restringida', hint: 'activos especialmente sensibles' },
];

const emptyOrganization = (): OrganizationInput => ({
  slug: '', name: '', category: '', description: '', active: true, behaviors: [],
});

/** Categories and slugs are stable identifiers, so the UI writes the server format for the admin. */
function toCode(value: string) {
  return value
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function behaviorText(behaviors: OrganizationBehavior[]) {
  return behaviors.length ? JSON.stringify(behaviors, null, 2) : '[]';
}

function behaviorFromText(text: string): OrganizationBehavior[] {
  const value: unknown = JSON.parse(text || '[]');
  if (!Array.isArray(value)) throw new Error('Los comportamientos deben ser una lista JSON.');
  return value.map((item) => {
    if (!item || typeof item !== 'object' || typeof (item as { code?: unknown }).code !== 'string') {
      throw new Error('Cada comportamiento requiere un campo "code".');
    }
    const config = (item as { config?: unknown }).config;
    if (config !== undefined && (!config || typeof config !== 'object' || Array.isArray(config))) {
      throw new Error('La configuración de cada comportamiento debe ser un objeto JSON.');
    }
    return { code: (item as { code: string }).code, config: (config ?? {}) as Record<string, unknown> };
  });
}

export function OrganizationsPage() {
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
  const [denied, setDenied] = useState(false);

  const load = useCallback(async () => {
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
    } catch { setDenied(true); }
  }, []);

  useEffect(() => { void load(); }, [load]);

  const selected = useMemo(
    () => organizations?.find((organization) => organization.id === selectedId) ?? null,
    [organizations, selectedId],
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
        return `${input.name} actualizada.`;
      }
      const created = await adminApi.createOrganization(input);
      setSelectedId(created.id);
      return `${input.name} creada. Ahora añade su primera red.`;
    });
  };

  if (denied) return <><h2>Organizaciones</h2><p className="muted">Esta sección requiere una cuenta administradora.</p></>;
  if (!organizations) return <Loading />;

  return (
    <>
      <h2>Organizaciones</h2>
      <p className="muted">Define las instituciones del juego y compón su infraestructura sin salir del panel.</p>
      <StatusMessage text={status.message} as="block" />

      <div className="org-layout">
        <AdminCard prefix="01/" brand="organización" note={selected ? 'editar entidad' : 'nueva entidad'}>
          <form className="org-form" onSubmit={saveOrganization}>
            <div className="org-form__grid">
              <label>Nombre<input required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder="Banco Central" /></label>
              <label>Slug<input required pattern="[a-z0-9-]+" value={form.slug} onChange={(e) => setForm({ ...form, slug: toCode(e.target.value) })} placeholder="banco-central" /></label>
              <label>Categoría<input required pattern="[a-z0-9-]+" value={form.category} onChange={(e) => setForm({ ...form, category: toCode(e.target.value) })} placeholder="finanzas" /><small>Se normaliza automáticamente: “Sector Público” → “sector-publico”.</small></label>
              <label className="org-check"><input type="checkbox" checked={form.active} onChange={(e) => setForm({ ...form, active: e.target.checked })} /> Activa en el juego</label>
            </div>
            <label>Descripción<textarea value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} placeholder="Qué representa esta institución y cuál es su función." /></label>
            <details className="org-behaviors">
              <summary>Comportamientos <small>avanzado · JSON declarativo</small></summary>
              <textarea value={behaviorJson} onChange={(e) => setBehaviorJson(e.target.value)} spellCheck={false} />
              <p>Ejemplo: <code>[{'{'}"code":"banking","config":{'{}'}{'}'}]</code>. Se almacenan para que las acciones web futuras los interpreten.</p>
            </details>
            <div className="org-actions"><button type="button" onClick={newOrganization} disabled={busy}>Nueva</button><button className="is-primary" disabled={busy}>{selected ? 'Guardar cambios' : 'Crear organización'}</button></div>
          </form>
        </AdminCard>

        <AdminCard prefix="map/" brand="instituciones" note={`${organizations.length} registradas`} accent="lime">
          <div className="org-list">
            {organizations.map((organization) => <button type="button" key={organization.id} className={`org-list__item ${organization.id === selectedId ? 'is-active' : ''}`} onClick={() => choose(organization)}>
              <span><b>{organization.name}</b><small>{organization.category} · {organization.networks.length} redes</small></span><em>{organization.active ? 'activa' : 'pausada'}</em>
            </button>)}
            {!organizations.length && <p className="org-empty">Crea la primera organización para comenzar.</p>}
          </div>
        </AdminCard>
      </div>

      {selected && <Infrastructure organization={selected} kits={kits} parts={parts} npcs={npcs} sites={sites} busy={busy} onRun={run} />}
    </>
  );
}

type InfrastructureProps = {
  organization: Organization; kits: Kit[]; parts: Part[]; npcs: OrganizationNpcCatalog[]; sites: Site[]; busy: boolean;
  onRun: (work: () => Promise<string>) => Promise<void>;
};

function Infrastructure(props: InfrastructureProps) {
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
  }, [organization.networks, selectedNodeId]);
  return <AdminCard prefix="02/" brand="infraestructura" note={organization.name} accent="lime">
    <div className="org-infra">
      <p className="org-infra__intro">Una red es interna por defecto. Un nodo pasa a Internet al instalarle una interfaz de red desde Computer.</p>
      <OrganizationNodeMap organization={organization} kits={kits} busy={busy} onSelectNode={({ npc }) => setSelectedNodeId(npc.id)} onRun={onRun} />
      <div className="org-networks">
      {selectedNode && <NodeInspector organization={organization} network={selectedNode.network} npc={selectedNode.npc} kits={kits} parts={parts} sites={sites} busy={busy} connectionSource={connectionSource} setConnectionSource={setConnectionSource} onRun={onRun} />}
        {!organization.networks.length && <p className="org-empty">Todavía no hay redes. Empieza por una red pública si el servicio tendrá web.</p>}
      </div>
    </div>
  </AdminCard>;
}

function NodeMap({ organization, kits, busy, onSelectNode, onRun }: { organization: Organization; kits: Kit[]; busy: boolean; onSelectNode: (node: { network: OrganizationNetwork; npc: OrganizationNetworkNpc }) => void; onRun: (work: () => Promise<string>) => Promise<void> }) {
  const canvas = useRef<HTMLDivElement>(null);
  const [dragging, setDragging] = useState<{ network: number; npc: OrganizationNetworkNpc } | null>(null);
  const [preview, setPreview] = useState<Record<number, { x: number; y: number }>>({});
  const [source, setSource] = useState<{ network: OrganizationNetwork; npc: OrganizationNetworkNpc } | null>(null);
  const [pointer, setPointer] = useState<{ x: number; y: number } | null>(null);
  const [creating, setCreating] = useState(false);
  const [newNode, setNewNode] = useState({ name: '', network: organization.networks[0]?.id ?? 0, kit: kits[0]?.id ?? 0 });
  const nodes = organization.networks.flatMap((network, networkIndex) => network.npcs.map((npc, nodeIndex) => ({ network, npc, x: preview[npc.id]?.x ?? (npc.map_x || 120 + networkIndex * 230), y: preview[npc.id]?.y ?? (npc.map_y || 90 + nodeIndex * 105) })));
  const position = (event: { clientX: number; clientY: number }) => {
    const rect = canvas.current?.getBoundingClientRect();
    if (!rect) return null;
    return { x: Math.round(Math.max(20, Math.min(rect.width - 170, event.clientX - rect.left - 80))), y: Math.round(Math.max(40, Math.min(rect.height - 80, event.clientY - rect.top - 28))) };
  };
  const drop = (event: { clientX: number; clientY: number }) => {
    if (!dragging || !canvas.current) return;
    const point = position(event);
    if (!point) return;
    const { network, npc } = dragging;
    setDragging(null);
    void onRun(async () => { await adminApi.editOrganizationNode(organization.id, network, npc.id, { zone: npc.zone, role: npc.role, map_x: point.x, map_y: point.y }); return `${npc.name} movido en el mapa.`; });
  };
  const beginLink = (event: React.MouseEvent, network: OrganizationNetwork, npc: OrganizationNetworkNpc) => { event.stopPropagation(); setSource({ network, npc }); setPointer(position(event)); };
  const connect = (_network: OrganizationNetwork, npc: OrganizationNetworkNpc) => { if (!source || source.npc.id === npc.id) return; void onRun(async () => { await adminApi.createOrganizationLink(organization.id, { from_npc_id: source.npc.id, to_npc_id: npc.id }); setSource(null); setPointer(null); return 'Conexión LAN creada.'; }); };
  const connectWeb = () => { if (!source) return; void onRun(async () => { await adminApi.connectOrganizationNodeToWeb(organization.id, source.network.id, source.npc.id); setSource(null); setPointer(null); return `${source.npc.name} conectado a WEB.`; }); };
  // Kept temporarily while existing saved screens migrate to OrganizationNodeMap.
  void [busy, pointer, creating, setCreating, newNode, setNewNode, beginLink, connect, connectWeb];
  return <div className="org-node-map" ref={canvas}>
    <span className="org-node-map__legend">Arrastra para mover · clic en un nodo para abrir su configuración</span>
    <svg className="org-node-map__links" aria-hidden="true">{(organization.links ?? []).map((link) => { const from = nodes.find((node) => node.npc.id === link.from_npc_id); const to = nodes.find((node) => node.npc.id === link.to_npc_id); return from && to ? <line key={link.id} x1={from.x + 80} y1={from.y + 28} x2={to.x + 80} y2={to.y + 28} /> : null; })}</svg>
    {nodes.map(({ network, npc, x, y }) => <button type="button" key={npc.id} className={`org-node org-node--${npc.zone}`} style={{ left: `${x}px`, top: `${y}px` }} onPointerDown={(event) => { event.currentTarget.setPointerCapture(event.pointerId); setDragging({ network: network.id, npc }); }} onPointerMove={(event) => { if (dragging?.npc.id === npc.id) { const point = position(event); if (point) setPreview((old) => ({ ...old, [npc.id]: point })); } }} onPointerUp={(event) => drop(event)} onClick={() => onSelectNode({ network, npc })}><small>{npc.zone}</small><b>{npc.name}</b><span>{formatIp(npc.ip)} <i className="org-lan">{Array.from({ length: Math.max(0, npc.lan_available) }).map((_, index) => <em key={index} />)}</i></span></button>)}
    {!nodes.length && <p className="org-empty">Crea una red y añade un servidor: aparecerá aquí como nodo.</p>}
  </div>;
}

void NodeMap;

function NodeInspector({ organization, network, npc, kits, parts, sites, busy, connectionSource, setConnectionSource, onRun }: { organization: Organization; network: OrganizationNetwork; npc: OrganizationNetworkNpc; kits: Kit[]; parts: Part[]; sites: Site[]; busy: boolean; connectionSource: { network: OrganizationNetwork; npc: OrganizationNetworkNpc } | null; setConnectionSource: (node: { network: OrganizationNetwork; npc: OrganizationNetworkNpc } | null) => void; onRun: (work: () => Promise<string>) => Promise<void> }) {
  const [editorOpen, setEditorOpen] = useState(false);
  const [computerOpen, setComputerOpen] = useState(false);
  const [computerConfiguration, setComputerConfiguration] = useState<OrganizationComputerConfiguration | null>(null);
  const site = sites.find((item) => item.kind === 'npc' && item.id === npc.id);
  const selectPort = () => { if (!connectionSource) { setConnectionSource({ network, npc }); return; } if (connectionSource.npc.id === npc.id) { setConnectionSource(null); return; } void onRun(async () => { await adminApi.createOrganizationLink(organization.id, { from_npc_id: connectionSource.npc.id, to_npc_id: npc.id }); setConnectionSource(null); return 'Conexión LAN creada.'; }); };
  const openComputer = () => void onRun(async () => { setComputerConfiguration(await adminApi.getOrganizationComputer(organization.id, network.id, npc.id)); setComputerOpen(true); return ''; });
  return <section className="org-node-inspector"><header><span className={`org-zone org-zone--${npc.zone}`}>{npc.zone}</span><h3>{npc.name}</h3></header><dl><div><dt>IP</dt><dd>{formatIp(npc.ip)}</dd></div><div><dt>Red</dt><dd>{network.name}</dd></div><div><dt>Rol</dt><dd>{npc.role}</dd></div><div><dt>Equipo</dt><dd>{npc.computer_components ? `${npc.computer_components} piezas · ${npc.source_kit_id ? `kit #${npc.source_kit_id}` : 'personalizado'}` : 'Sin configurar'}</dd></div></dl><div className="org-node-inspector__ports"><small>Puertos LAN {connectionSource ? '· selecciona este puerto para conectar' : ''}</small>{Array.from({ length: Math.max(0, npc.lan_available) }).map((_, index) => <button type="button" className={connectionSource ? 'is-linkable' : ''} key={index} onClick={selectPort} aria-label={`Puerto LAN ${index + 1}`} />)}{!npc.lan_available && <em>Sin puertos libres</em>}</div><div className="org-node-inspector__menus"><button className="is-primary" type="button" onClick={openComputer}>Computer</button><button type="button" onClick={() => setEditorOpen(true)}>Web</button></div><ComputerEditorDialog open={computerOpen} title={npc.name} kits={kits} parts={parts} configuration={computerConfiguration} lanLinks={Math.max(0, npc.lan_total - npc.lan_available - (npc.zone === 'public' ? 1 : 0))} busy={busy} onClose={() => setComputerOpen(false)} onSave={(input) => void onRun(async () => { await adminApi.saveOrganizationComputer(organization.id, network.id, npc.id, input); setComputerOpen(false); return `PC de ${npc.name} instalado en modo creativo.`; })} /><SiteEditorDialog open={editorOpen} title={npc.name} initialWeb={site?.web ?? ''} busy={busy} onClose={() => setEditorOpen(false)} onSave={(web) => void onRun(async () => { await adminApi.saveSite('npc', npc.id, web); setEditorOpen(false); return `Web de ${npc.name} guardada.`; })} /></section>;
}

function Network({ organization, network, kits, npcs, sites, busy, onRun }: { organization: Organization; network: OrganizationNetwork; kits: Kit[]; npcs: OrganizationNpcCatalog[]; sites: Site[]; busy: boolean; onRun: (work: () => Promise<string>) => Promise<void> }) {
  const [server, setServer] = useState<CreateOrganizationServerInput>({ name: '', kit_id: kits[0]?.id ?? 0, role: 'server', npc_type: 70, web: '' });
  const [npcId, setNpcId] = useState('');
  const [role, setRole] = useState('operator');
  const [networkName, setNetworkName] = useState(network.name);
  const [networkZone, setNetworkZone] = useState<NetworkZone>(network.zone);
  const [selectedNpcId, setSelectedNpcId] = useState<number | null>(network.npcs[0]?.id ?? null);
  const [nodeZone, setNodeZone] = useState<NetworkZone>(network.npcs[0]?.zone ?? network.zone);
  const availableNpcs = npcs.filter((npc) => npc.network_id !== network.id);
  const selectedNpc = network.npcs.find((npc) => npc.id === selectedNpcId) ?? network.npcs[0] ?? null;
  const currentWeb = sites.find((item) => item.kind === 'npc' && item.id === selectedNpc?.id)?.web ?? '';
  const [web, setWeb] = useState(currentWeb);
  const createServer = (event: FormEvent) => { event.preventDefault(); void onRun(async () => {
    const made = await adminApi.createOrganizationServer(organization.id, network.id, server);
    setServer({ ...server, name: '', web: '' });
    return `Servidor creado: ${formatIp(made.ip)} · contraseña inicial: ${made.password}. Guárdala ahora; no se vuelve a mostrar.`;
  }); };
  const assignNpc = (event: FormEvent) => { event.preventDefault(); if (!npcId) return; void onRun(async () => {
    await adminApi.assignOrganizationNpc(organization.id, network.id, { npc_id: Number(npcId), role }); setNpcId(''); return 'NPC asignado a la red.';
  }); };
  return <article className="org-network">
    <header><div><span className={`org-zone org-zone--${network.zone}`}>{ZONES.find((zone) => zone.value === network.zone)?.label}</span><h3>{network.name}</h3></div><button className="org-danger" disabled={busy} onClick={() => void onRun(async () => { await adminApi.removeOrganizationNetwork(organization.id, network.id); return `${network.name} eliminada.`; })}>Eliminar red</button></header>
    <details className="org-network-edit"><summary>Modificar esta red</summary><form onSubmit={(event) => { event.preventDefault(); void onRun(async () => { await adminApi.editOrganizationNetwork(organization.id, network.id, { name: networkName, zone: networkZone }); return `Red ${networkName} actualizada.`; }); }}><input required value={networkName} onChange={(event) => setNetworkName(event.target.value)} /><select value={networkZone} onChange={(event) => setNetworkZone(event.target.value as NetworkZone)}>{ZONES.map((zone) => <option value={zone.value} key={zone.value}>{zone.label}</option>)}</select><button disabled={busy}>Guardar red</button></form></details>
    <div className="org-network__machines">
      {network.npcs.map((npc) => <button type="button" className={`org-machine ${npc.id === selectedNpc?.id ? 'is-active' : ''}`} onClick={() => { setSelectedNpcId(npc.id); setNodeZone(npc.zone); setWeb(sites.find((item) => item.kind === 'npc' && item.id === npc.id)?.web ?? ''); }} key={npc.id}><b>{npc.name}</b><span>{formatIp(npc.ip)} · {npc.role}{npc.source_kit_id ? ` · kit #${npc.source_kit_id}` : ''}</span></button>)}
      {!network.npcs.length && <p className="org-empty">Sin equipos todavía.</p>}
    </div>
    {selectedNpc && <form className="org-web-form" onSubmit={(event) => { event.preventDefault(); void onRun(async () => { await Promise.all([adminApi.saveSite('npc', selectedNpc.id, web), adminApi.editOrganizationNode(organization.id, network.id, selectedNpc.id, { zone: nodeZone, role: selectedNpc.role, map_x: selectedNpc.map_x, map_y: selectedNpc.map_y })]); return `Web y zona de ${selectedNpc.name} guardadas.`; }); }}><div><b>Web de {selectedNpc.name}</b><small>{formatIp(selectedNpc.ip)} · HTML que verá quien visite este servidor.</small></div><select value={nodeZone} onChange={(event) => setNodeZone(event.target.value as NetworkZone)}>{ZONES.map((zone) => <option key={zone.value} value={zone.value}>{zone.label}</option>)}</select><textarea value={web} onChange={(event) => setWeb(event.target.value)} placeholder="<h1>Bienvenido</h1>" spellCheck={false} /><button className="is-primary" disabled={busy}>Guardar nodo</button></form>}
    <div className="org-network__actions">
      <form className="org-server-form" onSubmit={createServer}><b>Crear servidor desde un kit</b><input required value={server.name} onChange={(e) => setServer({ ...server, name: e.target.value })} placeholder="Gateway público" /><select required value={server.kit_id} onChange={(e) => setServer({ ...server, kit_id: Number(e.target.value) })}><option value={0} disabled>Selecciona un kit</option>{kits.map((kit) => <option value={kit.id} key={kit.id}>{kit.name}</option>)}</select><input value={server.role} onChange={(e) => setServer({ ...server, role: e.target.value })} placeholder="Rol" /><textarea value={server.web} onChange={(e) => setServer({ ...server, web: e.target.value })} placeholder="Web inicial opcional: <h1>Servicio</h1>" spellCheck={false} /><button className="is-primary" disabled={busy || !kits.length}>Crear servidor</button>{!kits.length && <small>Antes crea un kit válido en la sección Kits.</small>}</form>
      <form className="org-npc-form" onSubmit={assignNpc}><b>Asignar NPC existente</b><select value={npcId} onChange={(e) => setNpcId(e.target.value)}><option value="">Selecciona un NPC</option>{availableNpcs.map((npc) => <option value={npc.id} key={npc.id}>{npc.name} · {formatIp(npc.ip)}{npc.zone ? ` · ${npc.zone}` : ''}</option>)}</select><input value={role} onChange={(e) => setRole(e.target.value)} placeholder="Rol" /><button disabled={busy || !npcId}>Asignar</button></form>
    </div>
  </article>;
}

// Kept while its creation controls are moved into the map toolbar.
void Network;
