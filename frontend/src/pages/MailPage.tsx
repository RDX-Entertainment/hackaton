import { useCallback, useEffect, useState } from 'react';

import { ComposeMail } from '../components/ComposeMail';
import { StatusMessage } from '../components/StatusMessage';
import { mailApi, type UnknownRow } from '../utils/api';
import { formatCell } from '../utils/format';
import { useStatusMessage } from '../utils/useStatusMessage';

export function MailPage() {
  const status = useStatusMessage();

  const [messages, setMessages] = useState<UnknownRow[]>([]);
  const [opened, setOpened] = useState<UnknownRow>();

  const load = useCallback(() => {
    mailApi.listMail().then(setMessages).catch(() => setMessages([]));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const open = async (id: number) => {
    try {
      setOpened(await mailApi.readMail(id));
      // Reading flips the unread flag, so the list behind it is now stale.
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  const remove = async (id: number) => {
    try {
      await mailApi.removeMail(id);
      setOpened(undefined);
      status.report('Mail deleted.');
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Mail</h2>

      <ComposeMail onSent={load} />
      <StatusMessage text={status.message} as="block" />

      {opened && (
        <article className="forum-post">
          <b>{formatCell(opened.subject)}</b>
          <pre>{formatCell(opened.text)}</pre>
          <button onClick={() => void remove(Number(opened.id))}>Delete</button>
        </article>
      )}

      {messages.map((message) => (
        <article className="forum-post" key={formatCell(message.id)}>
          <button onClick={() => void open(Number(message.id))}>
            {formatCell(message.subject)}
          </button>
          <small>
            {' '}
            from {formatCell(message.from)} · {message.read ? 'read' : 'unread'}
          </small>
        </article>
      ))}
    </>
  );
}
