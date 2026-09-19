import { tr, useIdioma } from '../i18n/text';
import { useCallback, useEffect, useState, type FormEvent } from 'react';
import { Link, useParams } from 'react-router-dom';

import { Loading } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import { forumApi, type ForumTopicDetail } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

export function ForumThreadPage() {
  useIdioma();
  const { id = '' } = useParams();
  const status = useStatusMessage();

  const [topic, setTopic] = useState<ForumTopicDetail>();
  const [content, setContent] = useState('');

  const load = useCallback(() => {
    forumApi
      .getTopic(Number(id))
      .then(setTopic)
      .catch(() => setTopic(undefined));
  }, [id]);

  useEffect(() => {
    load();
  }, [load]);

  const reply = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await forumApi.replyToTopic(Number(id), content);
      setContent('');
      status.report(tr("Respuesta publicada."));
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  if (!topic) {
    return <Loading />;
  }

  return (
    <>
      <p>
        <Link to="/forum">{tr("← Volver al foro")}</Link>
      </p>

      <h2>{topic.title}</h2>

      {topic.posts.map((post, indice) => (
        <article className="hilo__mensaje" key={post.id}>
          <div className="hilo__firma">
            <span className="hilo__autor">{post.author}</span>
            <span className="micro">{indice === 0 ? tr("abrió el hilo") : `#${indice + 1}`}</span>
          </div>
          <pre className="hilo__texto">{post.content}</pre>
        </article>
      ))}

      <form className="hilo__mensaje" onSubmit={reply}>
        <div className="hilo__firma">
          <span className="hilo__autor">{tr("tú")}</span>
          <span className="micro">{tr("responder")}</span>
        </div>
        <textarea
          className="correo__cuerpo-editable"
          value={content}
          placeholder={tr("Lo que quieras decir.")}
          onChange={(event) => setContent(event.target.value)}
          required
        />
        <div className="correo__acciones">
          <StatusMessage text={status.message} />
          <button>{tr("Publicar")}</button>
        </div>
      </form>
    </>
  );
}
