import { useCallback, useEffect, useState } from 'react';
import { useLocation } from 'react-router-dom';

import { DataTable } from '../components/DataTable';
import { dashboardApi, type UnknownRow } from '../utils/api';
import { messageFrom } from '../utils/errors';

/**
 * Legacy pages that have no screen of their own yet. Some already answer with
 * rows and get a table; the rest answer with a placeholder status, which is
 * shown as-is rather than pretending there is a screen.
 */
const RESOURCE_PATHS: Record<string, string> = {
  ranking: '/ranking/users',
  log: '/logs',
};

type GenericResourcePageProps = {
  page: string;
  label: string;
};

export function GenericResourcePage({ page, label }: GenericResourcePageProps) {
  const location = useLocation();

  const [rows, setRows] = useState<UnknownRow[] | null>(null);
  const [status, setStatus] = useState('Loading...');

  const load = useCallback(() => {
    setRows(null);
    setStatus('Loading...');

    const resourcePath = RESOURCE_PATHS[page];
    const request = resourcePath
      ? dashboardApi.getResource(resourcePath)
      : dashboardApi.getLegacyPage(page);

    request
      .then((answer) => {
        if (Array.isArray(answer)) {
          setRows(answer);
        } else {
          setStatus(answer.status);
        }
      })
      .catch((error) => setStatus(messageFrom(error)));
  }, [page]);

  useEffect(() => {
    load();
  }, [load]);

  return (
    <>
      <h2>{label}</h2>

      {rows ? (
        <DataTable rows={rows} />
      ) : (
        <p className="muted">
          {status} · {location.search || 'default view'}
        </p>
      )}
    </>
  );
}
