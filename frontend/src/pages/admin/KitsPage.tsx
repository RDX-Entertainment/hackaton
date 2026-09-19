import { tr, useIdioma } from '../../i18n/text';
import { useCallback, useEffect, useState } from 'react';

import { KitBrowser } from '../../components/admin/KitBrowser';
import { AdminLoadError } from '../../components/admin/AdminLoadError';
import { KitForm } from '../../components/admin/KitForm';
import { Loading } from '../../components/Notice';
import { StatusMessage } from '../../components/StatusMessage';
import {
  adminApi,
  type Kit,
  type KitClan,
  type KitInput,
  type Part,
} from '../../utils/api';
import { useStatusMessage } from '../../utils/useStatusMessage';

export function KitsPage() {
  useIdioma();
  const status = useStatusMessage();

  const [kits, setKits] = useState<Kit[]>();
  const [parts, setParts] = useState<Part[]>([]);
  const [clans, setClans] = useState<KitClan[]>([]);
  const [loadError, setLoadError] = useState<unknown>(null);
  const [editing, setEditing] = useState<Kit | null>(null);
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    setLoadError(null);
    try {
      setKits(await adminApi.listKits());
      setParts(await adminApi.listParts());
      setClans(await adminApi.listClans());
    } catch (error) {
      setLoadError(error);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const run = async (work: () => Promise<string>) => {
    setBusy(true);

    try {
      status.report(await work());
      await load();
    } catch (error) {
      status.reportError(error);
    } finally {
      setBusy(false);
    }
  };

  const save = (input: KitInput) =>
    void run(async () => {
      if (editing) {
        await adminApi.editKit(editing.id, input);
        setEditing(null);
        return tr("{0} actualizado.", input.name);
      }

      await adminApi.createKit(input);
      return tr("{0} creado.", input.name);
    });

  const remove = (kit: Kit) =>
    void run(async () => {
      await adminApi.removeKit(kit.id);
      return tr("{0} borrado.", kit.name);
    });

  const makeDefault = (kit: Kit) =>
    void run(async () => {
      await adminApi.setDefaultKit(kit.id, true);
      return tr("{0} es ahora el equipo con el que nace un jugador.", kit.name);
    });

  if (loadError) return <AdminLoadError title="Kits" error={loadError} onRetry={() => void load()} />;

  if (!kits) {
    return <Loading />;
  }

  return (
    <>
      <h2>Kits</h2>
      <StatusMessage text={status.message} as="block" />

      <KitForm
        editing={editing}
        parts={parts}
        clans={clans}
        busy={busy}
        onSubmit={save}
        onCancel={() => setEditing(null)}
      />

      <KitBrowser
        kits={kits}
        busy={busy}
        onEdit={setEditing}
        onRemove={remove}
        onDefault={makeDefault}
      />
    </>
  );
}
