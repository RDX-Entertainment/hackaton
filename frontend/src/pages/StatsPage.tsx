import { useEffect, useState } from 'react';

import { DataTable } from '../components/DataTable';
import { dashboardApi, type UnknownRow } from '../utils/api';

export function StatsPage() {
  const [stats, setStats] = useState<UnknownRow>();

  useEffect(() => {
    dashboardApi.getServerStats().then(setStats).catch(() => setStats(undefined));
  }, []);

  // The snapshot only exists once the worker has run at least once.
  if (!stats) {
    return <p>Waiting for the game worker…</p>;
  }

  return (
    <>
      <h2>Server statistics</h2>
      <DataTable rows={[stats]} />
    </>
  );
}
