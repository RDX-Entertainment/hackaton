import { tr, useIdioma } from '../../i18n/text';
import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { useAccion } from '../../app/tutorial';
import { arsenalApi, type HackKind, type Program, type Remote } from '../../utils/api';
import type { RemoteTab } from './RemotePage';
import './intrusion.css';

const SERVICES = [
  { code: 'web_exploit', title: 'Web', protocol: 'HTTP · 80', icon: '</>', permission: 'write_web', tab: 'webedit', get detail() { return tr("Obtén acceso al sitio y modifica su HTML con un manipulador web. No abre el disco ni SSH."); } },
  { code: 'ftp_exploit', get title() { return tr("Almacenamiento"); }, protocol: 'FTP · 21', icon: '▤', permission: 'write_files', tab: 'files', get detail() { return tr("Accede a los archivos de este nodo. Descarga, sube o borra software; no concede control web."); } },
  { code: 'ssh_exploit', title: 'SSH', protocol: 'SSH · 22', icon: '>_', permission: 'modify_logs', tab: 'access', get detail() { return tr("Consigue una sesión de usuario, acceso a archivos y al registro de la máquina."); } },
] as const;

type Props = {
  remote: Remote; busy: boolean;
  onCrack: (kind: HackKind, softwareId?: number) => void;
  onIntrude: (capability: string, softwareId: number) => void;
  onAnalyze: () => void;
  onTab: (tab: RemoteTab) => void;
};

export function ServicePanel({ remote, busy, onCrack, onIntrude, onAnalyze, onTab }: Props) {
  useIdioma();
  const hackear = useAccion('red.hackear');
  const [programs, setPrograms] = useState<Program[]>([]);
  const [selected, setSelected] = useState<Record<string, number>>({});
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(true);
  useEffect(() => {
    let current = true;
    arsenalApi.listPrograms().then((items) => { if (current) setPrograms(items); })
      .catch(() => { if (current) setError(tr("No se pudo cargar el arsenal. Vuelve a abrir Servicios para reintentar.")); })
      .finally(() => { if (current) setLoading(false); });
    return () => { current = false; };
  }, []);
  const disabled = busy || remote.down || !hackear.permitida;
  const scanners = programs.filter(p => p.functions.includes('port_scan'));
  const scanner = scanners.find(p => p.software_id === selected.port_scan) ?? scanners.find(p => p.running) ?? scanners[0];
  const canScan = scanner?.running && scanner.resources && scanner.resources.cpu >= scanner.stats.cpu && scanner.resources.ram >= scanner.stats.ram && scanner.resources.net >= scanner.stats.net;
  return <div className="page intrusion">
    <div className="intrusion__recon">
      <div><h3>{tr("Elige tu punto de entrada")}</h3><p>{tr("Reconocimiento → intrusión → acciones. Cada puerta concede permisos distintos y los accesos se acumulan.")}</p></div>
      <button disabled={disabled} onClick={onAnalyze}>{tr("Analizar hardware")}</button>
    </div>
    <section className="intrusion__scanner" aria-label={tr("Escáner de puertos")}>
      <div><h4>{tr("Escáner de puertos")}</h4><p>{tr("Eficacia para descubrir HTTP, FTP y SSH; sigilo para evitar alertas. Requiere certificación 3. No concede acceso por sí solo.")}</p></div>
      {scanners.length ? <>
        <label>{tr("Programa de escaneo")}<select value={scanner?.software_id ?? ''} onChange={e => setSelected(old => ({...old, port_scan:Number(e.target.value)}))}>
          {scanners.map(p => <option key={p.software_id} value={p.software_id}>{p.name} · {p.running ? 'activo' : 'detenido'}</option>)}
        </select></label>
        {scanner && <small>{tr("Eficacia")} {scanner.stats.effectiveness} {tr("· sigilo")} {scanner.stats.stealth} {tr("· estabilidad")} {scanner.stats.stability}</small>}
        {!canScan && <small>{tr("Activa el escáner en Software y comprueba los recursos de su equipo.")}</small>}
      </> : <p>{loading ? tr("Cargando escáneres…") : tr("No tienes un escáner. Crea y compila uno en el Lab, y actívalo en Software.")} <Link to="/lab?capability=port_scan">{tr("Crear escáner de puertos")}</Link></p>}
      <button disabled={disabled || !canScan} onClick={() => onCrack('scan', scanner?.software_id)}>{remote.scanned ? tr("Volver a escanear") : tr("Escanear puertos")}</button>
    </section>
    {error && <p role="alert">{error}</p>}
    <div className="intrusion__services">
      {SERVICES.map((service) => {
        const candidates = programs.filter((p) => p.functions.includes(service.code));
        const program = candidates.find((p) => p.software_id === selected[service.code]) ?? candidates.find((p) => p.running) ?? candidates[0];
        const access = remote.permissions.includes(service.permission);
        const resources = program?.resources;
        const fits = program && resources && resources.cpu >= program.stats.cpu && resources.ram >= program.stats.ram && resources.net >= program.stats.net;
        const reason = remote.down ? tr("Máquina fuera de línea") : !remote.scanned ? tr("Escanea primero · certificación 3") : loading ? tr("Cargando arsenal…") : !program ? tr("Compila esta capacidad en el Lab") : !program.running ? tr("Activa el programa desde Software") : !fits ? tr("El equipo no tiene recursos disponibles") : undefined;
        return <section className={`intrusion__service${access ? ' is-open' : ''}`} key={service.code}>
          <header><span className="intrusion__icon" aria-hidden="true">{service.icon}</span><span><small>{service.protocol}</small><h4>{service.title}</h4></span><span className="intrusion__badge">{remote.down ? 'offline' : access ? tr("acceso") : remote.scanned ? 'disponible' : tr("sin explorar")}</span></header>
          <p>{service.detail}</p>
          {access ? <button className="is-primary" disabled={remote.down} onClick={() => onTab(service.tab)}>{tr("Abrir")} {service.title}</button> : <>
            <label>{tr("Programa de ataque ·")} {service.title}<select value={program?.software_id ?? ''} onChange={(e) => setSelected((old) => ({ ...old, [service.code]: Number(e.target.value) }))}>
              {!candidates.length && <option value="">{tr("Sin programa compatible")}</option>}
              {candidates.map((p) => <option key={p.software_id} value={p.software_id}>{p.name} · {p.running ? 'activo' : 'detenido'}</option>)}
            </select></label>
            {program && <dl className="intrusion__stats"><div><dt>{tr("Eficacia")}</dt><dd>{program.stats.effectiveness}</dd></div><div><dt>{tr("Penetración")}</dt><dd>{program.stats.penetration}</dd></div><div><dt>{tr("Sigilo")}</dt><dd>{program.stats.stealth}</dd></div></dl>}
            <button disabled={disabled || Boolean(reason)} title={hackear.motivo ?? reason} onClick={() => program && onIntrude(service.code, program.software_id)}>{tr("Atacar")} {service.title}</button>
            {reason && <small className="intrusion__hint">{reason}</small>}
          </>}
        </section>;
      })}
    </div>
    <section className="intrusion__root"><div><h4>{tr("Fuerza bruta · acceso completo")}</h4><p>{tr("Rompe la credencial de la máquina. Si triunfas, abre los tres servicios. Eficacia, penetración y sigilo se enfrentan a sus defensas.")}</p></div>
      <button data-foco="red.hackear" data-guia="red.cracker" className={hackear.destacada ? 'es-foco' : undefined} disabled={disabled || (remote.permissions.includes('write_web') && remote.permissions.includes('modify_logs') && remote.permissions.includes('write_files'))} title={hackear.motivo ?? undefined} onClick={() => onCrack('bf')}>{tr("Lanzar fuerza bruta")}</button>
    </section>
    <p className="page__note">{tr("Las operaciones tardan y pueden fallar o ser detectadas. Los módulos equipados cuentan. ¿Te falta una herramienta?")} <Link to="/lab">{tr("Ir al Lab")}</Link>.</p>
  </div>;
}
