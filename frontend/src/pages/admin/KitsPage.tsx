import { useCallback, useEffect, useState } from 'react';

import { KitBrowser } from '../../components/admin/KitBrowser';
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
  const status = useStatusMessage();

  const [kits, setKits] = useState<Kit[]>();
  const [parts, setParts] = useState<Part[]>([]);
  const [clans, setClans] = useState<KitClan[]>([]);
  const [denied, setDenied] = useState(false);
  const [editing, setEditing] = useState<Kit | null>(null);
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    try {
      setKits(await adminApi.listKits());
      setParts(await adminApi.listParts());
      setClans(await adminApi.listClans());
    } catch {
      setDenied(true);
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
        return `${input.name} actualizado.`;
      }

      await adminApi.createKit(input);
      return `${input.name} creado.`;
    });

  const remove = (kit: Kit) =>
    void run(async () => {
      await adminApi.removeKit(kit.id);
      return `${kit.name} borrado.`;
    });

  const makeDefault = (kit: Kit) =>
    void run(async () => {
      await adminApi.setDefaultKit(kit.id, true);
      return `${kit.name} es ahora el equipo con el que nace un jugador.`;
    });

  if (denied) {
    return (
      <>
        <h2>Kits</h2>
        <p className="muted">
          Esta sección no existe para tu cuenta. El servidor responde 404 a quien no es
          administrador, igual que a una ruta inventada.
        </p>
      </>
    );
  }

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
