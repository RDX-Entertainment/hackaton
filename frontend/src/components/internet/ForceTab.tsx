import type { HackKind, Remote } from '../../utils/api';

type Attack = {
  key: string;
  title: string;
  command: string;
  detail: string;
  run: () => void;
  done?: boolean;
  blocked?: string;
};

type ForceTabProps = {
  remote: Remote;
  busy: boolean;
  onCrack: (kind: HackKind) => void;
  onExploit: () => void;
  onAnalyze: () => void;
};

/**
 * Donde se prueba a entrar por la fuerza.
 *
 * Nada de aquí ocurre en el momento: todo encola un proceso que resuelve el worker
 * comparando tu software contra el suyo. Por eso cada acción dice qué encola y no
 * finge un resultado que todavía no existe.
 */
export function ForceTab({ remote, busy, onCrack, onExploit, onAnalyze }: ForceTabProps) {
  const down = remote.down ? 'La máquina no responde.' : undefined;
  const inside = remote.access ? 'Ya tienes sesión abierta aquí.' : undefined;

  const attacks: Attack[] = [
    {
      key: 'scan',
      title: 'Escaneo de puertos',
      command: 'nmap',
      detail: 'Mira qué servicios están abiertos. Es lo que decide si el exploit tiene por dónde entrar.',
      run: () => onCrack('scan'),
      done: remote.scanned,
      blocked: down,
    },
    {
      key: 'bf',
      title: 'Fuerza bruta',
      command: 'cracker',
      detail: 'Tu crackeador contra su contraseña. Lo que tarda depende de la diferencia de versiones.',
      run: () => onCrack('bf'),
      blocked: down ?? inside,
    },
    {
      key: 'exploit',
      title: 'Exploit FTP / SSH',
      command: 'exploit',
      detail: 'Entra por un servicio abierto en vez de por la puerta. Escanea antes o irás a ciegas.',
      run: onExploit,
      blocked: down ?? inside,
    },
    {
      key: 'analyze',
      title: 'Análisis',
      command: 'analyzer',
      detail: 'Estima el hardware que hay detrás. No es un ataque, pero dice contra qué te enfrentas.',
      run: onAnalyze,
      done: Boolean(remote.analysis),
      blocked: down,
    },
  ];

  return (
    <div className="page">
      <p className="force__warn">
        Todo lo de aquí queda registrado en la máquina de enfrente. Si es de un jugador, lo
        verá — y el registro se puede reescribir desde <b>acceso</b>, si llegas a entrar.
      </p>

      <ul className="force">
        {attacks.map((attack) => (
          <li
            key={attack.key}
            className={attack.blocked ? 'attack is-blocked' : 'attack'}
          >
            <div className="attack__head">
              <span className="attack__cmd">{attack.command}</span>
              <h4>{attack.title}</h4>
              {attack.done && <span className="attack__done">hecho</span>}
            </div>

            <p className="attack__detail">{attack.detail}</p>

            <div className="attack__foot">
              {attack.blocked ? (
                <span className="attack__blocked">{attack.blocked}</span>
              ) : (
                <button type="button" disabled={busy} onClick={attack.run}>
                  lanzar
                </button>
              )}
            </div>
          </li>
        ))}
      </ul>

      <p className="page__note">
        Nada de esto es inmediato: cada uno encola un proceso y el worker lo resuelve. Míralo
        en Procesos y vuelve cuando termine.
      </p>
    </div>
  );
}
