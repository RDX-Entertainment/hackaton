import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { Loading } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import { accountApi, type Settings } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

const MIN_PASSWORD_LENGTH = 8;

export function SettingsPage() {
  const status = useStatusMessage();

  const [settings, setSettings] = useState<Settings>();
  const [email, setEmail] = useState('');
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');

  const load = useCallback(() => {
    accountApi.getSettings().then((loaded) => {
      setSettings(loaded);
      setEmail(loaded.email);
    });
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const save = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await accountApi.saveSettings(email, currentPassword, newPassword);
      setCurrentPassword('');
      setNewPassword('');
      status.report('Settings saved.');
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  if (!settings) {
    return <Loading />;
  }

  return (
    <>
      <h2>Settings · {settings.login}</h2>

      <form className="compose" onSubmit={save}>
        <label>
          Email
          <input
            type="email"
            value={email}
            onChange={(event) => setEmail(event.target.value)}
            required
          />
        </label>

        <label>
          Current password
          <input
            type="password"
            value={currentPassword}
            onChange={(event) => setCurrentPassword(event.target.value)}
            required
          />
        </label>

        <label>
          New password (optional)
          <input
            type="password"
            value={newPassword}
            minLength={MIN_PASSWORD_LENGTH}
            onChange={(event) => setNewPassword(event.target.value)}
          />
        </label>

        <button>Save settings</button>
        <StatusMessage text={status.message} />
      </form>
    </>
  );
}
