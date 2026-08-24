import { useCallback, useEffect, useState } from 'react';

import { Loading } from '../../components/Notice';
import { StatusMessage } from '../../components/StatusMessage';
import { adminApi, type AdminUser } from '../../utils/api';
import { formatCount, formatDate } from '../../utils/format';
import { formatIp } from '../../utils/ip';
import { useStatusMessage } from '../../utils/useStatusMessage';

export function UsersPage() {
  const status = useStatusMessage();

  const [users, setUsers] = useState<AdminUser[]>();
  const [busy, setBusy] = useState(false);

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

  if (!users) {
    return <Loading />;
  }

  return (
    <>
      <h2>Usuarios</h2>
      <StatusMessage text={status.message} as="block" />

      <div className="table-wrap">
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
              <tr key={user.id}>
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
                  <button type="button" disabled={busy} onClick={() => toggleAdmin(user)}>
                    {user.is_admin ? 'quitar admin' : 'hacer admin'}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

    </>
  );
}
