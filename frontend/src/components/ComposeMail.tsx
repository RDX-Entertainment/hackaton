import { useState, type FormEvent } from 'react';

import { mailApi } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';
import { StatusMessage } from './StatusMessage';

const MAX_SUBJECT_LENGTH = 40;

type ComposeMailProps = {
  /** Called after a successful send so the caller can reload its list. */
  onSent: () => void;
};

export function ComposeMail({ onSent }: ComposeMailProps) {
  const status = useStatusMessage();

  const [to, setTo] = useState('');
  const [subject, setSubject] = useState('');
  const [text, setText] = useState('');

  const send = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await mailApi.sendMail(to, subject, text);
      setTo('');
      setSubject('');
      setText('');
      status.report('E-mail sent.');
      onSent();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <form className="compose" onSubmit={send}>
      <label>
        To
        <input value={to} onChange={(event) => setTo(event.target.value)} required />
      </label>

      <label>
        Subject
        <input
          value={subject}
          maxLength={MAX_SUBJECT_LENGTH}
          onChange={(event) => setSubject(event.target.value)}
          required
        />
      </label>

      <label>
        Message
        <textarea value={text} onChange={(event) => setText(event.target.value)} required />
      </label>

      <button>Send</button>
      <StatusMessage text={status.message} />
    </form>
  );
}
