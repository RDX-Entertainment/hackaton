import { tr, useIdioma } from '../../i18n/text';
import { useAccion } from '../../app/tutorial';
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
  useIdioma();
  const hackear = useAccion('red.hackear');

  const down = remote.down ? tr("La máquina no responde.") : undefined;
  const inside = remote.access ? tr("Ya tienes sesión abierta aquí.") : undefined;

  const attacks: Attack[] = [
    {
      key: 'scan',
      title: tr("Escaneo de puertos"),
      command: 'nmap',
      detail: tr("Mira qué servicios están abiertos. Es lo que decide si el exploit tiene por dónde entrar."),
      run: () => onCrack('scan'),
      done: remote.scanned,
      blocked: down,
    },
    {
      key: 'bf',
      title: tr("Fuerza bruta"),
      command: 'cracker',
      detail: tr("Eficacia contra resistencia, penetración contra blindaje y sigilo contra detección. Los atributos y el hardware deciden las probabilidades y el tiempo; la versión no decide el resultado."),
      run: () => onCrack('bf'),
      blocked: down ?? inside,
    },
    {
      key: 'exploit',
      title: 'Exploit FTP / SSH',
      command: 'exploit',
      detail: tr("Entra por un servicio abierto en vez de por la puerta. Escanea antes o irás a ciegas."),
      run: onExploit,
      blocked: down ?? inside,
    },
    {
      key: 'analyze',
      title: tr("Análisis"),
      command: 'analyzer',
      detail: tr("Estima el hardware que hay detrás. No es un ataque, pero dice contra qué te enfrentas."),
      run: onAnalyze,
      done: Boolean(remote.analysis),
      blocked: down,
    },
  ];

  return (
    <div className="page">
      <p className="force__warn">
        {tr("Todo lo de aquí queda registrado en la máquina de enfrente. Si es de un jugador, lo verá — y el registro se puede reescribir desde")} <b>{tr("acceso")}</b>{tr(", si llegas a entrar.")} </p>

      <ul className="force">
        {attacks.map((attack) => (
          <li
            key={attack.key}
            className={attack.blocked ? 'attack is-blocked' : 'attack'}
          >
            <div className="attack__head">
              <span className="attack__cmd">{attack.command}</span>
              <h4>{attack.title}</h4>
              {attack.done && <span className="attack__done">{tr("hecho")}</span>}
            </div>

            <p className="attack__detail">{attack.detail}</p>

            <div className="attack__foot">
              {attack.blocked ? (
                <span className="attack__blocked">{attack.blocked}</span>
              ) : (
                <button
                  type="button"
                  data-foco="red.hackear"
                  data-guia={attack.key === 'bf' ? 'red.cracker' : undefined}
                  className={hackear.destacada ? 'es-foco' : undefined}
                  disabled={busy || !hackear.permitida}
                  title={hackear.motivo ?? undefined}
                  onClick={attack.run}
                >
                  {tr("lanzar")} </button>
              )}
            </div>
          </li>
        ))}
      </ul>

      <p className="page__note">
        {tr("Nada de esto es inmediato: cada uno encola un proceso y el worker lo resuelve. Míralo en Procesos y vuelve cuando termine.")} </p>
    </div>
  );
}
