import { useCallback, useEffect, useState } from 'react';

import { AccountSelect } from '../components/AccountSelect';
import { Icono } from '../components/iconos/Icono';
import { Loading } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import { Tag } from '../components/Tag';
import {
  certificationsApi,
  financesApi,
  type Certification,
  type Certifications,
  type UnknownRow,
} from '../utils/api';
import { formatMoney } from '../utils/format';
import { useStatusMessage } from '../utils/useStatusMessage';

/**
 * La Academia.
 *
 * Los seis niveles eran seis párrafos iguales, y para saber por cuál ibas había
 * que leerlos todos. Ahora el número manda a la izquierda, el estado se dice
 * con el vocabulario del sistema, y lo bloqueado lleva trama —no solo menos
 * brillo—, porque en monocromo bajar la opacidad se confunde con estar apagado.
 *
 * El nivel 1 es gratis y es el tutorial: por eso la tarjeta del siguiente que
 * puedes hacer es la única viva de la pantalla.
 */
export function UniversityPage() {
  const status = useStatusMessage();

  const [certificaciones, setCertificaciones] = useState<Certifications>();
  const [cuentas, setCuentas] = useState<UnknownRow[]>([]);
  const [cuenta, setCuenta] = useState('');

  const cargar = useCallback(() => {
    certificationsApi.getCertifications().then(setCertificaciones);
  }, []);

  useEffect(() => {
    cargar();
    financesApi.listAccounts().then(setCuentas).catch(() => setCuentas([]));
  }, [cargar]);

  const actuar = async (certificacion: Certification) => {
    try {
      if (certificacion.learning) {
        await certificationsApi.completeCertification(certificacion.level);
      } else {
        await certificationsApi.startCertification(
          certificacion.level,
          // Lo gratis no se le cobra a nadie.
          certificacion.price ? Number(cuenta) : undefined,
        );
      }

      status.report('Academia al día.');
      cargar();
    } catch (error) {
      status.reportError(error);
    }
  };

  if (!certificaciones) {
    return <Loading />;
  }

  const hacenFaltaCuentas = certificaciones.items.some(
    (item) => item.price > 0 && !item.completed,
  );

  return (
    <>
      <h2>Academia</h2>
      <StatusMessage text={status.message} as="block" />

      {hacenFaltaCuentas && (
        <AccountSelect
          accounts={cuentas}
          value={cuenta}
          onChange={setCuenta}
          label="Cuenta con la que pagas"
        />
      )}

      <div className="rejilla-tarjetas">
        {certificaciones.items.map((item) => {
          const enMarcha = !item.completed && item.learning;
          const abierto = !item.completed && !item.learning && item.available;
          const bloqueado = !item.completed && !item.learning && !item.available;

          const clase = item.completed
            ? 'tarjeta tarjeta--numerada tarjeta--hecha'
            : enMarcha || abierto
              ? 'tarjeta tarjeta--numerada tarjeta--viva'
              : 'tarjeta tarjeta--numerada tarjeta--bloqueada';

          return (
            <article className={clase} key={item.level}>
              <span className="tarjeta__nivel">{item.level}</span>

              <div style={{ minWidth: 0, flex: 1 }}>
                <div className="tarjeta__cabecera">
                  <span className="tarjeta__titulo">{item.name}</span>
                  <span className="micro">
                    {item.price ? (
                      <>
                        <Icono nombre="dinero" tamano={11} /> {formatMoney(item.price)}
                      </>
                    ) : (
                      'gratis'
                    )}
                  </span>
                </div>

                <p className="tarjeta__texto">{item.description}</p>

                <div className="tarjeta__acciones">
                  {item.completed && <Tag estado="memoria">hecho</Tag>}
                  {enMarcha && <Tag estado="obra">en marcha</Tag>}
                  {abierto && <Tag estado="plano">te toca</Tag>}
                  {bloqueado && <Tag estado="off">bloqueado</Tag>}

                  {enMarcha && <button onClick={() => void actuar(item)}>Terminar</button>}
                  {abierto && (
                    <button
                      disabled={item.price > 0 && !cuenta}
                      onClick={() => void actuar(item)}
                    >
                      Empezar
                    </button>
                  )}
                  {bloqueado && <span className="micro">termina el anterior</span>}
                </div>
              </div>
            </article>
          );
        })}
      </div>
    </>
  );
}
