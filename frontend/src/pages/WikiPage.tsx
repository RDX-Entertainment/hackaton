import { tr, useIdioma } from '../i18n/text';
import { useCallback, useEffect, useState, type FormEvent } from 'react';
import { Link, useParams } from 'react-router-dom';

import { StatusMessage } from '../components/StatusMessage';
import { wikiApi, type WikiPage as WikiPageData } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

const DEFAULT_SLUG = 'start';

export function WikiPage() {
  useIdioma();
  const { slug = DEFAULT_SLUG } = useParams();
  const status = useStatusMessage();

  const [page, setPage] = useState<WikiPageData>();
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');

  const load = useCallback(() => {
    wikiApi
      .getWikiPage(slug)
      .then((loaded) => {
        setPage(loaded);
        setTitle(loaded.title);
        setContent(loaded.content);
      })
      .catch(() => {
        // A slug with no page yet is not an error: it is a page to be written.
        setPage(undefined);
        setTitle(slug);
        setContent('');
      });
  }, [slug]);

  useEffect(() => {
    load();
  }, [load]);

  const save = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await wikiApi.saveWikiPage(slug, title, content);
      status.report(tr("Saved."));
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <p>
        <Link to="/wiki">{tr("← Wiki index")}</Link>
      </p>

      <h2>{page?.title || tr("New page: {0}", slug)}</h2>

      {page && (
        <article className="wiki-content">
          <pre>{page.content}</pre>
          <small>{tr("Last author:")} {page.author || 'unknown'}</small>
        </article>
      )}

      <h3>{tr("Edit")}</h3>
      <form className="compose" onSubmit={save}>
        <label>
          {tr("Title")} <input value={title} onChange={(event) => setTitle(event.target.value)} required />
        </label>

        <label>
          {tr("Content")} <textarea
            value={content}
            onChange={(event) => setContent(event.target.value)}
            required
          />
        </label>

        <button>{tr("Save revision")}</button>
        <StatusMessage text={status.message} />
      </form>
    </>
  );
}
