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
      status.report('Topic created.');
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Forum</h2>

      <form className="compose" onSubmit={post}>
        <label>
          Topic title
          <input value={title} onChange={(event) => setTitle(event.target.value)} required />
        </label>

        <label>
          First post
          <textarea
            value={content}
            onChange={(event) => setContent(event.target.value)}
            required
          />
        </label>

        <button>Post topic</button>
        <StatusMessage text={status.message} />
      </form>

      <div className="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Topic</th>
              <th>Author</th>
              <th>Posts</th>
            </tr>
          </thead>
          <tbody>
            {topics.map((topic) => (
              <tr key={topic.id}>
                <td>
                  <Link to={`/forum/${topic.id}`}>{topic.title}</Link>
                </td>
                <td>{topic.author}</td>
                <td>{topic.posts}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}
