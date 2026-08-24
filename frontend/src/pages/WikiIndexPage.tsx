import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';

import { DataTable } from '../components/DataTable';
import { wikiApi, type WikiSummary } from '../utils/api';

export function WikiIndexPage() {
  const [pages, setPages] = useState<WikiSummary[]>([]);

  useEffect(() => {
    wikiApi.listWikiPages().then(setPages).catch(() => setPages([]));
  }, []);

  return (
    <>
      <h2>Wiki</h2>
      <p className="muted">Documentación colaborativa del juego.</p>
      <p>
        <Link to="/wiki/start">Create or edit the start page</Link>
      </p>
      <DataTable rows={pages} />
    </>
  );
}
