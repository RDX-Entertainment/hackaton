import { tr, useIdioma } from '../../i18n/text';
import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { AccountsBoard } from '../../components/admin/AccountsBoard';
import { AdminCard } from '../../components/admin/AdminCard';
import { Loading } from '../../components/Notice';
import { StatusMessage } from '../../components/StatusMessage';
import { adminApi, type AdminCreateUserInput, type AdminUser, type BankAccount, type Kit } from '../../utils/api';
import { formatCount, formatDate, formatMoney } from '../../utils/format';
import { formatIp } from '../../utils/ip';
import { useStatusMessage } from '../../utils/useStatusMessage';

export function UsersPage() {
  useIdioma();
  const status = useStatusMessage();

  const [users, setUsers] = useState<AdminUser[]>();
  const [busy, setBusy] = useState(false);
  const [selectedId, setSelectedId] = useState<number | null>(null);

  const load = useCallback(async () => {
    setUsers(await adminApi.listUsers());
  }, []);

  useEffect(() => {
    load().catch((error) => status.reportError(error));
  }, [load, status]);

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

  const toggleAdmin = (user: AdminUser) =>
    void run(async () => {
      await adminApi.setAdminFlag(user.id, !user.is_admin);
      return user.is_admin ? tr("{0} deja de ser admin.", user.login) : tr("{0} ahora es admin.", user.login);
    });

  const selected = users?.find((user) => user.id === selectedId) ?? null;

  if (!users) {
    return <Loading />;
  }

  return (
    <>
      <h2>{tr("Usuarios")}</h2>
      <StatusMessage text={status.message} as="block" />
      <UserCreator onCreated={load} />

      <div className="table-wrap users-table">
        <table>
          <thead>
            <tr>
              <th>id</th>
              <th>login</th>
              <th>email</th>
              <th>ip</th>
              <th>{tr("piezas")}</th>
              <th>{tr("último acceso")}</th>
              <th>{tr("rol")}</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {users.map((user) => (
              <tr
                key={user.id}
                className={user.id === selectedId ? 'is-active' : ''}
                onClick={() => setSelectedId(user.id === selectedId ? null : user.id)}
              >
                <td>{user.id}</td>
                <td>{user.login}</td>
                <td>{user.email}</td>
                <td>{formatIp(user.game_ip)}</td>
                <td>{formatCount(user.components)}</td>
                <td>{formatDate(user.last_login)}</td>
                <td>
                  <span className={user.is_admin ? 'role role--admin' : 'role'}>
                    {user.is_admin ? 'admin' : 'jugador'}
                  </span>
                </td>
                <td>
                  <button
                    type="button"
                    disabled={busy}
                    onClick={(event) => {
                      event.stopPropagation();
                      toggleAdmin(user);
                    }}
                  >
                    {user.is_admin ? tr("quitar admin") : tr("hacer admin")}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {selected && <UserAccounts user={selected} busy={busy} onRun={run} />}
    </>
  );
}

function UserCreator({ onCreated }: { onCreated: () => Promise<void> }) {
  useIdioma();
  const status = useStatusMessage();
  const [kits, setKits] = useState<Kit[]>([]);
  const [busy, setBusy] = useState(false);
  const [draft, setDraft] = useState<AdminCreateUserInput>({ username: '', email: '', password: '', kit_id: undefined, admin: false });

  useEffect(() => { adminApi.listKits().then(setKits).catch(status.reportError); }, [status.reportError]);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    try {
      await adminApi.createUser({ ...draft, email: draft.email || undefined, kit_id: draft.kit_id || undefined });
      setDraft({ username: '', email: '', password: '', kit_id: undefined, admin: false });
      status.report(tr("Cuenta creada."));
      await onCreated();
    } catch (error) { status.reportError(error); } finally { setBusy(false); }
  };

  return (
    <form className="compose" onSubmit={submit}>
      <h3>{tr("Crear cuenta")}</h3>
      <label>{tr("Usuario")}<input required maxLength={15} value={draft.username} onChange={(event) => setDraft({ ...draft, username: event.target.value })} /></label>
      <label>{tr("Email")}<input type="email" value={draft.email} onChange={(event) => setDraft({ ...draft, email: event.target.value })} /></label>
      <label>{tr("Contraseña")}<input required minLength={8} type="password" value={draft.password} onChange={(event) => setDraft({ ...draft, password: event.target.value })} /></label>
      <label>{tr("Kit inicial")}<select value={draft.kit_id ?? ''} onChange={(event) => setDraft({ ...draft, kit_id: event.target.value ? Number(event.target.value) : undefined })}><option value="">{tr("kit por defecto")}</option>{kits.map((kit) => <option key={kit.id} value={kit.id}>{kit.name}</option>)}</select></label>
      <label><input type="checkbox" checked={draft.admin} onChange={(event) => setDraft({ ...draft, admin: event.target.checked })} /> {tr("Administrador")}</label>
      <button disabled={busy}>{tr("crear usuario")}</button>
      <StatusMessage text={status.message} />
    </form>
  );
}

/** Las cuentas del jugador elegido: abrir una, ingresarle dinero y quitárselo. */
function UserAccounts({ user, busy, onRun }: { user: AdminUser; busy: boolean; onRun: (work: () => Promise<string>) => Promise<void> }) {
  useIdioma();
  const [accounts, setAccounts] = useState<BankAccount[]>([]);

  const load = useCallback(() => {
    adminApi.listUserAccounts(user.id).then(setAccounts).catch(() => setAccounts([]));
  }, [user.id]);

  useEffect(() => {
    load();
  }, [load]);

  return (
    <AdminCard prefix="03/" brand="cuentas" note={tr("{0} · {1} abiertas", user.login, accounts.length)} accent="lime">
      <AccountsBoard
        accounts={accounts}
        busy={busy}
        emptyMessage={tr("{0} no tiene ninguna cuenta abierta.", user.login)}
        onOpen={(bankId, pin) =>
          void onRun(async () => {
            const opened = await adminApi.openUserAccount(user.id, { bank_id: bankId, password: pin });
            load();
            return tr("Cuenta {0} abierta a nombre de {1}.", opened.account, user.login);
          })
        }
        onAdjust={(account, amount) =>
          void onRun(async () => {
            const { cash } = await adminApi.adjustAccountCash(account.id, amount);
            load();
            const verbo = amount > 0 ? tr("Ingresados") : tr("Retirados");
            return tr("{0} {1} en {2}. Saldo: {3}.", verbo, formatMoney(Math.abs(amount)), account.account, formatMoney(cash));
          })
        }
      />
    </AdminCard>
  );
}
