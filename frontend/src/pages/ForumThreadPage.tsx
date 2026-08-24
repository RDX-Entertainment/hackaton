import { useCallback, useEffect, useState, type FormEvent } from 'react';
import { Link, useParams } from 'react-router-dom';

import { Loading } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import { forumApi, type ForumTopicDetail } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

export function ForumThreadPage() {
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
      status.report('Reply posted.');
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
        <Link to="/forum">← Forum</Link>
      </p>

      <h2>{topic.title}</h2>

      {topic.posts.map((post) => (
        <article className="forum-post" key={post.id}>
          <b>{post.author}</b>
          <pre>{post.content}</pre>
        </article>
      ))}

      <form className="compose" onSubmit={reply}>
        <label>
          Reply
          <textarea
            value={content}
            onChange={(event) => setContent(event.target.value)}
            required
          />
        </label>

        <button>Reply</button>
        <StatusMessage text={status.message} />
      </form>
    </>
  );
}
