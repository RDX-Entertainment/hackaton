import type { Visited } from '../../utils/api';
import { formatIp } from '../../utils/ip';

type StartPageProps = {
  history: Visited[];
  onVisit: (ip: number) => void;
};

/**
 * Lo que se ve cuando no hay ninguna máquina abierta. El listín ya vive en el raíl de la
 * derecha, así que aquí solo queda por dónde ibas.
 */
export function StartPage({ history, onVisit }: StartPageProps) {
  return (
    <section className="viewport viewport--start">
      <div className="start">
        <pre className="start__art" aria-hidden="true">
          {String.raw`   ___     _                     _
  |_ _|_ _| |_ ___ _ _ _ _  ___| |_
   | || ' \  _/ -_) '_| ' \/ -_)  _|
  |___|_||_\__\___|_| |_||_\___|\__|`}
        </pre>

        <p className="start__lead">
          Escribe una dirección arriba, o entra por una de las conocidas del listín.
        </p>

        {history.length > 0 && (
          <section className="start__block">
            <h4>Últimas visitas</h4>

            <div className="start__recent">
              {history.slice(0, 10).map((one) => (
                <button type="button" key={one.ip} onClick={() => onVisit(one.ip)}>
                  {one.name || formatIp(one.ip)}
                </button>
              ))}
            </div>
          </section>
        )}
      </div>
    </section>
  );
}
