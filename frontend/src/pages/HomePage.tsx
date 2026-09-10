import { useEffect, useState } from 'react';

import { Icono } from '../components/iconos/Icono';
import { CircuitRail } from '../components/onboarding/CircuitRail';
import { NextStep } from '../components/onboarding/NextStep';
import { useOnboarding } from '../components/onboarding/useOnboarding';
import { EmptyNotice, ErrorNotice, Loading } from '../components/Notice';
import { dashboardApi, type Dashboard, type News } from '../utils/api';
import { messageFrom } from '../utils/errors';
import { formatMoney } from '../utils/format';

/**
 * La portada.
 *
 * Antes abría con tres números y las noticias, y no había nada en toda la pantalla que
 * dijera qué hacer: para quien acababa de registrarse, un panel de estado de un estado que
 * todavía no tenía. Ahora abre por lo que toca —el paso deducido del mundo, no una marca
 * guardada— y el estado baja al segundo lugar, que es donde estaba su sitio.
 */
export function HomePage() {
  const [dashboard, setDashboard] = useState<Dashboard>();
  const [news, setNews] = useState<News[]>([]);
  const [error, setError] = useState('');
  const { onboarding } = useOnboarding();

  useEffect(() => {
    dashboardApi.getDashboard().then(setDashboard).catch((cause) => setError(messageFrom(cause)));
    dashboardApi.getNews().then(setNews).catch(() => setNews([]));
  }, []);

  if (error) {
    return <ErrorNotice>{error}</ErrorNotice>;
  }

  if (!dashboard) {
    return <Loading />;
  }

  return (
    <>
      <h2>Inicio</h2>

      {onboarding && (
        <>
          <NextStep onboarding={onboarding} />
          <CircuitRail onboarding={onboarding} />
        </>
      )}

      <section className="estado">
        <article className="tarjeta">
          <span className="tarjeta__rotulo">
            <Icono nombre="dinero" tamano={12} /> en el banco
          </span>
          <b className="tarjeta__cifra">{formatMoney(dashboard.money)}</b>
          <div className="tarjeta__regla" />
          <p className="tarjeta__pie">Lo que puedes gastar ahora mismo.</p>
        </article>

        <article className="tarjeta">
          <span className="tarjeta__rotulo">
            <Icono nombre="tiempo" tamano={12} /> en marcha
          </span>
          <b
            className={
              dashboard.processes
                ? 'tarjeta__cifra tarjeta__cifra--tiempo'
                : 'tarjeta__cifra tarjeta__cifra--quieta'
            }
          >
            {dashboard.processes}
          </b>
          <div className="tarjeta__regla" />
          <p className="tarjeta__pie">
            {dashboard.processes
              ? 'Tareas que termina el worker por su cuenta.'
              : 'Nada corriendo. Lo que empieces aparece aquí.'}
          </p>
        </article>

        <article className="tarjeta">
          <span className="tarjeta__rotulo">
            <Icono nombre="aviso" tamano={12} /> correo
          </span>
          <b
            className={
              dashboard.unread_mail
                ? 'tarjeta__cifra'
                : 'tarjeta__cifra tarjeta__cifra--quieta'
            }
          >
            {dashboard.unread_mail}
          </b>
          <div className="tarjeta__regla" />
          <p className="tarjeta__pie">Sin leer.</p>
        </article>
      </section>

      <h3>Noticias</h3>
      {news.length ? (
        news.map((item) => (
          <article className="forum-post" key={item.id}>
            <b>{item.title}</b>
            <small> por {item.author}</small>
            <pre>{item.content}</pre>
          </article>
        ))
      ) : (
        <EmptyNotice>Todavía no hay noticias.</EmptyNotice>
      )}
    </>
  );
}
