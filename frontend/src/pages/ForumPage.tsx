import { tr, useIdioma } from '../i18n/text';
import { useCallback, useEffect, useState, type FormEvent } from 'react';
import { Link } from 'react-router-dom';

import { StatusMessage } from '../components/StatusMessage';
import { forumApi, type ForumTopic } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

export function ForumPage() {
  useIdioma();
  const [abriendo, setAbriendo] = useState(false);
  const status = useStatusMessage();

  const [topics, setTopics] = useState<ForumTopic[]>([]);
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');

  const load = useCallback(() => {
    forumApi.listTopics().then(setTopics).catch(() => setTopics([]));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const post = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await forumApi.createTopic(title, content);
      setTitle('');
      setContent('');
      status.report(tr("Hilo abierto."));
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>{tr("Foro")}</h2>

      <div className="tablon">
        <div className="tablon__barra">
          <span className="micro">
            {topics.length} {topics.length === 1 ? tr("hilo") : tr("hilos")}
          </span>
          <button onClick={() => setAbriendo((antes) => !antes)}>
            {abriendo ? tr("Dejarlo") : tr("Abrir hilo")}
          </button>
        </div>

        {abriendo && (
          <form className="compose" onSubmit={post} style={{ margin: 0, borderWidth: '0 0 1px' }}>
            <label>
              {tr("Título del hilo")} <input value={title} onChange={(event) => setTitle(event.target.value)} required />
            </label>

            <label>
              {tr("Primer mensaje")} <textarea
                value={content}
                onChange={(event) => setContent(event.target.value)}
                required
              />
            </label>

            <button>{tr("Publicar")}</button>
            <StatusMessage text={status.message} />
          </form>
        )}

        {topics.length ? (
          topics.map((topic) => (
            <div className="tablon__hilo" key={topic.id}>
              <Link className="tablon__titulo" to={`/forum/${topic.id}`}>
                {topic.title}
              </Link>
              <span className="tablon__quien">{tr("abrió")} {topic.author}</span>
              <span className="tablon__cuenta">
                <b>{topic.posts}</b>
                <span>{Number(topic.posts) === 1 ? tr("mensaje") : tr("mensajes")}</span>
              </span>
            </div>
          ))
        ) : (
          <p className="correo__nada">
            {tr("Todavía no hay ningún hilo. El foro es del juego: lo que se cuenta aquí lo leen los demás jugadores.")} </p>
        )}
      </div>

    </>
  );
}
