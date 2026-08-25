import { useEffect, useMemo, useRef, useState, type Dispatch, type FormEvent, type MouseEvent, type PointerEvent, type SetStateAction } from 'react';

import { adminApi, type Kit, type Organization, type OrganizationNetwork, type OrganizationNetworkNpc } from '../../utils/api';
import { formatIp } from '../../utils/ip';

type Node = { network: OrganizationNetwork; npc: OrganizationNetworkNpc; x: number; y: number };
type SelectedNode = Pick<Node, 'network' | 'npc'>;

type Props = {
  organization: Organization;
  kits: Kit[];
  busy: boolean;
  onSelectNode: (node: SelectedNode) => void;
  onRun: (work: () => Promise<string>) => Promise<void>;
};

/** Interactive topology editor. It owns transient canvas state; the page owns data loading. */
export function OrganizationNodeMap({ organization, kits, busy, onSelectNode, onRun }: Props) {
  const canvas = useRef<HTMLDivElement>(null);
  const [source, setSource] = useState<SelectedNode | null>(null);
  const [cursor, setCursor] = useState<{ x: number; y: number } | null>(null);
  const [dragging, setDragging] = useState<Node | null>(null);
  const [positions, setPositions] = useState<Record<number, { x: number; y: number }>>({});
  // Keep the gateway in the visible canvas by default; it can then be placed anywhere.
  const [webPosition, setWebPosition] = useState({ x: 42, y: 110 });
  const [draggingWeb, setDraggingWeb] = useState(false);
  const [draft, setDraft] = useState({ name: '', network: organization.networks[0]?.id ?? 0, kit: kits[0]?.id ?? 0 });
  const [creating, setCreating] = useState(false);

  // Redes y kits llegan del API despues del primer render, y el inicializador de useState solo
  // corre una vez: sin esto el borrador se queda en 0 y el boton de crear no se despierta nunca.
  // Tambien reengancha la seleccion cuando la red o el kit elegidos dejan de existir.
  useEffect(() => {
    setDraft((current) => {
      const network = organization.networks.some((item) => item.id === current.network) ? current.network : organization.networks[0]?.id ?? 0;
      const kit = kits.some((item) => item.id === current.kit) ? current.kit : kits[0]?.id ?? 0;
      return network === current.network && kit === current.kit ? current : { ...current, network, kit };
    });
  }, [organization.networks, kits]);

  const nodes = useMemo<Node[]>(() => organization.networks.flatMap((network, networkIndex) => network.npcs.map((npc, nodeIndex) => ({
    network,
    npc,
    x: positions[npc.id]?.x ?? (npc.map_x || 180 + networkIndex * 240),
    y: positions[npc.id]?.y ?? (npc.map_y || 100 + nodeIndex * 105),
  }))), [organization, positions]);
  const nodeOf = (id: number) => nodes.find((node) => node.npc.id === id);
  const connectionsOf = (node: Node) => {
    const peers = organization.links.flatMap((link) => {
      const peerId = link.from_npc_id === node.npc.id ? link.to_npc_id : link.to_npc_id === node.npc.id ? link.from_npc_id : null;
      return peerId ? [nodeOf(peerId)?.npc.name ?? 'Nodo desconocido'] : [];
    });
    return node.npc.zone === 'public' ? [...peers, 'INTERNET'] : peers;
  };
  const exposedNodes = nodes.filter((node) => node.npc.zone === 'public');
  const point = (event: { clientX: number; clientY: number }) => {
    const rect = canvas.current?.getBoundingClientRect();
    return rect ? { x: event.clientX - rect.left, y: event.clientY - rect.top } : null;
  };
  const mapPosition = (event: { clientX: number; clientY: number }) => {
    const rect = canvas.current?.getBoundingClientRect();
    if (!rect) return null;
    return { x: Math.round(Math.max(20, Math.min(rect.width - 170, event.clientX - rect.left - 80))), y: Math.round(Math.max(40, Math.min(rect.height - 80, event.clientY - rect.top - 28))) };
  };
  const cancelLink = () => { setSource(null); setCursor(null); };
  const beginLink = (event: MouseEvent, node: SelectedNode) => {
    event.stopPropagation();
    setSource(node);
    setCursor(point(event));
  };
  const completeLink = (target: SelectedNode) => {
    if (!source || source.npc.id === target.npc.id || target.npc.lan_available < 1) return;
    void onRun(async () => {
      await adminApi.createOrganizationLink(organization.id, { from_npc_id: source.npc.id, to_npc_id: target.npc.id });
      cancelLink();
      return 'Conexión LAN creada.';
    });
  };
  const removeLink = (link: Organization['links'][number]) => {
    void onRun(async () => {
      await adminApi.removeOrganizationLink(organization.id, link.id);
      return 'Conexión LAN eliminada.';
    });
  };
  const saveNewNode = (event: FormEvent) => {
    event.preventDefault();
    void onRun(async () => {
      const made = await adminApi.createOrganizationServer(organization.id, draft.network, { name: draft.name, kit_id: draft.kit, role: 'server', npc_type: 70, web: '' });
      setCreating(false);
      setDraft((current) => ({ ...current, name: '' }));
      return `Nodo creado: ${formatIp(made.ip)}.`;
    });
  };
  const startDrag = (event: PointerEvent<HTMLButtonElement>, node: Node) => {
    if (source || event.button !== 0) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    setDragging(node);
  };
  const moveDrag = (event: PointerEvent<HTMLButtonElement>, node: Node) => {
    if (dragging?.npc.id !== node.npc.id) return;
    const next = mapPosition(event);
    if (next) setPositions((current) => ({ ...current, [node.npc.id]: next }));
  };
  const finishDrag = (event: PointerEvent<HTMLButtonElement>, node: Node) => {
    if (dragging?.npc.id !== node.npc.id) return;
    const next = mapPosition(event);
    setDragging(null);
    if (!next) return;
    void onRun(async () => {
      await adminApi.editOrganizationNode(organization.id, node.network.id, node.npc.id, { role: node.npc.role, map_x: next.x, map_y: next.y });
      return `${node.npc.name} movido en el mapa.`;
    });
  };
  const moveWeb = (event: PointerEvent<HTMLButtonElement>) => {
    if (!draggingWeb) return;
    const next = mapPosition(event);
    if (next) setWebPosition(next);
  };

  return <div className="org-node-map" ref={canvas} onPointerMove={(event) => source && setCursor(point(event))} onPointerLeave={() => source && setCursor(null)}>
    <span className="org-node-map__legend">Arrastra para mover · pulsa un puerto LAN para cablear</span>
    <button className="org-node-map__create" type="button" onClick={() => setCreating(true)}>+ Crear nodo</button>
    <svg className="org-node-map__links" aria-hidden="true">
      {organization.links.map((link) => { const from = nodeOf(link.from_npc_id); const to = nodeOf(link.to_npc_id); return from && to ? <line className="org-node-map__link" key={link.id} x1={from.x + 80} y1={from.y + 28} x2={to.x + 80} y2={to.y + 28} onClick={() => removeLink(link)}><title>Eliminar conexión LAN</title></line> : null; })}
      {exposedNodes.map((node) => <line key={`web-${node.npc.id}`} x1={node.x + 80} y1={node.y + 28} x2={webPosition.x + 80} y2={webPosition.y + 28} />)}
      {source && cursor && <line className="org-node-map__draft" x1={(nodeOf(source.npc.id)?.x ?? 0) + 80} y1={(nodeOf(source.npc.id)?.y ?? 0) + 28} x2={cursor.x} y2={cursor.y} />}
    </svg>
    {exposedNodes.length > 0 && <WebGateway position={webPosition} exposedNode={exposedNodes[0]} onStartDrag={(event) => { event.currentTarget.setPointerCapture(event.pointerId); setDraggingWeb(true); }} onMoveDrag={moveWeb} onFinishDrag={() => setDraggingWeb(false)} />}
    {nodes.map((node) => <MapNode key={node.npc.id} node={node} connections={connectionsOf(node)} source={source} onSelect={onSelectNode} onBeginLink={beginLink} onCompleteLink={completeLink} onStartDrag={startDrag} onMoveDrag={moveDrag} onFinishDrag={finishDrag} />)}
    {creating && <CreateNodeForm draft={draft} networks={organization.networks} kits={kits} busy={busy} onChange={setDraft} onCancel={() => setCreating(false)} onSubmit={saveNewNode} />}
  </div>;
}

function MapNode({ node, connections, source, onSelect, onBeginLink, onCompleteLink, onStartDrag, onMoveDrag, onFinishDrag }: { node: Node; connections: string[]; source: SelectedNode | null; onSelect: (node: SelectedNode) => void; onBeginLink: (event: MouseEvent, node: SelectedNode) => void; onCompleteLink: (node: SelectedNode) => void; onStartDrag: (event: PointerEvent<HTMLButtonElement>, node: Node) => void; onMoveDrag: (event: PointerEvent<HTMLButtonElement>, node: Node) => void; onFinishDrag: (event: PointerEvent<HTMLButtonElement>, node: Node) => void }) {
  const isSource = source?.npc.id === node.npc.id;
  const isTarget = Boolean(source && !isSource && node.npc.lan_available > 0);
  return <button type="button" className={`org-node org-node--${node.npc.zone}`} style={{ left: node.x, top: node.y }} onPointerDown={(event) => onStartDrag(event, node)} onPointerMove={(event) => onMoveDrag(event, node)} onPointerUp={(event) => onFinishDrag(event, node)} onClick={() => source ? onCompleteLink(node) : onSelect(node)}>
    <small>{node.npc.zone}</small><b>{node.npc.name}</b><span>{formatIp(node.npc.ip)} <i className="org-lan">{Array.from({ length: Math.max(0, node.npc.lan_total) }).map((_, index) => { const connection = connections[index]; return <em key={index} title={connection ? `Ocupado: ${connection}` : 'Puerto LAN disponible'} className={connection ? 'is-used' : isSource ? 'is-source' : isTarget ? 'is-target' : ''} onPointerDown={(event) => event.stopPropagation()} onClick={(event) => !connection && !source && onBeginLink(event, node)} />; })}</i></span>
  </button>;
}

function WebGateway({ position, exposedNode, onStartDrag, onMoveDrag, onFinishDrag }: { position: { x: number; y: number }; exposedNode: Node; onStartDrag: (event: PointerEvent<HTMLButtonElement>) => void; onMoveDrag: (event: PointerEvent<HTMLButtonElement>) => void; onFinishDrag: () => void }) {
  return <button type="button" className="org-web-gateway" style={{ left: position.x, top: position.y }} onPointerDown={onStartDrag} onPointerMove={onMoveDrag} onPointerUp={onFinishDrag} aria-label="Nodo Internet">
    <span className="org-web-gateway__icon" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M4 7h16M4 12h16M4 17h16M7 4v16M12 4v16M17 4v16" /></svg></span><b>INTERNET</b><small>IP expuesta · {formatIp(exposedNode.npc.ip)}</small>
  </button>;
}

function CreateNodeForm({ draft, networks, kits, busy, onChange, onCancel, onSubmit }: { draft: { name: string; network: number; kit: number }; networks: OrganizationNetwork[]; kits: Kit[]; busy: boolean; onChange: Dispatch<SetStateAction<{ name: string; network: number; kit: number }>>; onCancel: () => void; onSubmit: (event: FormEvent) => void }) {
  // Un select sin opciones no se despliega, asi que el motivo va escrito: callarlo deja un
  // formulario que parece roto.
  const blocked = !networks.length
    ? 'Esta organización no tiene ninguna red todavía. Créala abajo, en Infraestructura.'
    : !kits.length
      ? 'No hay ningún kit. Crea uno en la sección Kits.'
      : '';
  return <form className="org-node-create" onSubmit={onSubmit}><b>Crear nodo</b><input required value={draft.name} placeholder="Nombre" onChange={(event) => onChange((current) => ({ ...current, name: event.target.value }))} /><select value={draft.network} disabled={!networks.length} onChange={(event) => onChange((current) => ({ ...current, network: Number(event.target.value) }))}>{networks.length ? networks.map((network) => <option key={network.id} value={network.id}>{network.name}</option>) : <option value={0}>Sin redes</option>}</select><select value={draft.kit} disabled={!kits.length} onChange={(event) => onChange((current) => ({ ...current, kit: Number(event.target.value) }))}>{kits.length ? kits.map((kit) => <option key={kit.id} value={kit.id}>{kit.name}</option>) : <option value={0}>Sin kits</option>}</select>{blocked && <small className="org-node-create__blocked">{blocked}</small>}<span><button type="button" onClick={onCancel}>Cancelar</button><button className="is-primary" disabled={busy || Boolean(blocked) || !draft.network || !draft.kit}>Crear</button></span></form>;
}
