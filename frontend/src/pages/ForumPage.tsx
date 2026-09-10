import { useCallback, useEffect, useState, type FormEvent } from 'react';
import { Link } from 'react-router-dom';

import { StatusMessage } from '../components/StatusMessage';
import { forumApi, type ForumTopic } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

export function ForumPage() {
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
      status.report('Hilo abierto.');
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Foro</h2>

      <form className="compose" onSubmit={post}>
        <label>
          Título del hilo
          <input value={title} onChange={(event) => setTitle(event.target.value)} required />
        </label>

        <label>
          Primer mensaje
          <textarea
            value={content}
            onChange={(event) => setContent(event.target.value)}
            required
          />
        </label>

        <button>Abrir hilo</button>
        <StatusMessage text={status.message} />
      </form>

      {topics.length ? (
        <div className="rejilla-tarjetas">
          {topics.map((topic) => (
            <article className="tarjeta" key={topic.id}>
              <span className="tarjeta__rotulo">abrió {topic.author}</span>

              <div className="tarjeta__cabecera" style={{ marginTop: '0.35rem' }}>
                <Link className="tarjeta__titulo" to={`/forum/${topic.id}`}>
                  {topic.title}
                </Link>
              </div>

              <div className="tarjeta__acciones">
                <span className="micro">
                  {topic.posts} {Number(topic.posts) === 1 ? 'mensaje' : 'mensajes'}
                </span>
                <Link to={`/forum/${topic.id}`}>leer</Link>
              </div>
            </article>
          ))}
        </div>
      ) : (
        <div className="vacio">
          <div className="vacio__titulo">Todavía no hay ningún hilo</div>
          <p className="vacio__texto">
            El foro es del juego: lo que se cuenta aquí lo leen los demás jugadores.
          </p>
        </div>
      )}
    </>
  );
}
