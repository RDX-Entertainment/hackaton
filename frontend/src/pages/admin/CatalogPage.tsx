import { tr, useIdioma } from '../../i18n/text';
import { useCallback, useEffect, useState } from 'react';

import { CatalogBrowser } from '../../components/admin/CatalogBrowser';
import { DeliveryCard } from '../../components/admin/DeliveryCard';
import { PartForm } from '../../components/admin/PartForm';
import { Loading } from '../../components/Notice';
import { StatusMessage } from '../../components/StatusMessage';
import {
  adminApi,
  type AdminUser,
  type Chipset,
  type Part,
  type PartInput,
} from '../../utils/api';
import { useStatusMessage } from '../../utils/useStatusMessage';

/** Alta de piezas, entrega directa y el catálogo, en ese orden. */
export function CatalogPage() {
  useIdioma();
  const status = useStatusMessage();

  const [parts, setParts] = useState<Part[]>();
  const [chipsets, setChipsets] = useState<Chipset[]>([]);
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [denied, setDenied] = useState(false);
  const [editing, setEditing] = useState<Part | null>(null);
  const [busy, setBusy] = useState(false);

  const loadChipsets = useCallback(async () => {
    setChipsets(await adminApi.listChipsets());
  }, []);

  const load = useCallback(async () => {
    try {
      setParts(await adminApi.listParts());
      await loadChipsets();
      setUsers(await adminApi.listUsers());
    } catch {
      setDenied(true);
    }
  }, [loadChipsets]);

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

  const save = (input: PartInput) =>
    void run(async () => {
      if (editing) {
        await adminApi.editPart(editing.id, input);
        setEditing(null);
        return tr("{0} actualizado.", input.name);
      }

      await adminApi.createPart(input);
      return tr("{0} creado.", input.name);
    });

  const remove = (part: Part) =>
    void run(async () => {
      await adminApi.removePart(part.id);
      return tr("{0} borrado.", part.name);
    });

  const deliver = (part: number, user: number, amount: number) =>
    void run(async () => {
      const made = await adminApi.grantComponents(part, user, amount);
      const who = users.find((option) => option.id === user)?.login ?? user;

      return tr("{0} piezas entregadas a {1}.", made.components.length, who);
    });

  if (denied) {
    return (
      <>
        <h2>{tr("Catálogo")}</h2>
        <p className="muted">
          {tr("Esta sección no existe para tu cuenta. El servidor responde 404 a quien no es administrador, igual que a una ruta inventada.")} </p>
      </>
    );
  }

  if (!parts) {
    return <Loading />;
  }

  return (
    <>
      <h2>{tr("Catálogo")}</h2>
      <StatusMessage text={status.message} as="block" />

      <PartForm
        editing={editing}
        chipsets={chipsets}
        busy={busy}
        onSubmit={save}
        onCancel={() => setEditing(null)}
        onChipsetCreated={loadChipsets}
      />

      <DeliveryCard parts={parts} users={users} busy={busy} onDeliver={deliver} />

      <CatalogBrowser parts={parts} busy={busy} onEdit={setEditing} onRemove={remove} />
    </>
  );
}
