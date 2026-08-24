import { useEffect, useState } from 'react';

import { EmptyNotice, ErrorNotice, Loading } from '../components/Notice';
import { dashboardApi, type Dashboard, type News } from '../utils/api';
import { messageFrom } from '../utils/errors';
import { formatMoney } from '../utils/format';

export function HomePage() {
  const [dashboard, setDashboard] = useState<Dashboard>();
  const [news, setNews] = useState<News[]>([]);
  const [error, setError] = useState('');

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
      <h2>Home</h2>

      <section className="cards">
        <article>
          <b>{formatMoney(dashboard.money)}</b>
          <small>Total money</small>
        </article>
        <article>
          <b>{dashboard.processes}</b>
          <small>Processes</small>
        </article>
        <article>
          <b>{dashboard.unread_mail}</b>
          <small>Unread mail</small>
        </article>
      </section>

      <h3>News</h3>
      {news.length ? (
        news.map((item) => (
          <article className="forum-post" key={item.id}>
            <b>{item.title}</b>
            <small> by {item.author}</small>
            <pre>{item.content}</pre>
          </article>
        ))
      ) : (
        <EmptyNotice>No news yet.</EmptyNotice>
      )}
    </>
  );
}
