import { useCallback, useEffect, useState } from 'react';

import { AccountsBoard } from '../../components/admin/AccountsBoard';
import { AdminCard } from '../../components/admin/AdminCard';
import { Loading } from '../../components/Notice';
import { StatusMessage } from '../../components/StatusMessage';
import { adminApi, type AdminUser, type BankAccount } from '../../utils/api';
import { formatCount, formatDate, formatMoney } from '../../utils/format';
import { formatIp } from '../../utils/ip';
import { useStatusMessage } from '../../utils/useStatusMessage';

export function UsersPage() {
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
      return user.is_admin ? `${user.login} deja de ser admin.` : `${user.login} ahora es admin.`;
    });

  const selected = users?.find((user) => user.id === selectedId) ?? null;

  if (!users) {
    return <Loading />;
  }

  return (
    <>
      <h2>Usuarios</h2>
      <StatusMessage text={status.message} as="block" />

      <div className="table-wrap users-table">
        <table>
          <thead>
            <tr>
              <th>id</th>
              <th>login</th>
              <th>email</th>
              <th>ip</th>
              <th>piezas</th>
              <th>último acceso</th>
              <th>rol</th>
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
                    {user.is_admin ? 'quitar admin' : 'hacer admin'}
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

/** Las cuentas del jugador elegido: abrir una, ingresarle dinero y quitárselo. */
function UserAccounts({ user, busy, onRun }: { user: AdminUser; busy: boolean; onRun: (work: () => Promise<string>) => Promise<void> }) {
  const [accounts, setAccounts] = useState<BankAccount[]>([]);

  const load = useCallback(() => {
    adminApi.listUserAccounts(user.id).then(setAccounts).catch(() => setAccounts([]));
  }, [user.id]);

  useEffect(() => {
    load();
  }, [load]);

  return (
    <AdminCard prefix="03/" brand="cuentas" note={`${user.login} · ${accounts.length} abiertas`} accent="lime">
      <AccountsBoard
        accounts={accounts}
        busy={busy}
        emptyMessage={`${user.login} no tiene ninguna cuenta abierta.`}
        onOpen={(bankId, pin) =>
          void onRun(async () => {
            const opened = await adminApi.openUserAccount(user.id, { bank_id: bankId, password: pin });
            load();
            return `Cuenta ${opened.account} abierta a nombre de ${user.login}.`;
          })
        }
        onAdjust={(account, amount) =>
          void onRun(async () => {
            const { cash } = await adminApi.adjustAccountCash(account.id, amount);
            load();
            const verbo = amount > 0 ? 'Ingresados' : 'Retirados';
            return `${verbo} ${formatMoney(Math.abs(amount))} en ${account.account}. Saldo: ${formatMoney(cash)}.`;
          })
        }
      />
    </AdminCard>
  );
}
