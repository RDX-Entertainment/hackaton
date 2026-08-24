import { useCallback, useEffect, useState } from 'react';

import { AccountSelect } from '../components/AccountSelect';
import { Loading } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import {
  certificationsApi,
  financesApi,
  type Certification,
  type Certifications,
  type UnknownRow,
} from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

export function UniversityPage() {
  const status = useStatusMessage();

  const [certifications, setCertifications] = useState<Certifications>();
  const [accounts, setAccounts] = useState<UnknownRow[]>([]);
  const [account, setAccount] = useState('');

  const load = useCallback(() => {
    certificationsApi.getCertifications().then(setCertifications);
  }, []);

  useEffect(() => {
    load();
    financesApi.listAccounts().then(setAccounts).catch(() => setAccounts([]));
  }, [load]);

  const act = async (certification: Certification) => {
    try {
      if (certification.learning) {
        await certificationsApi.completeCertification(certification.level);
      } else {
        await certificationsApi.startCertification(
          certification.level,
          // A free certification is charged to nobody.
          certification.price ? Number(account) : undefined,
        );
      }

      status.report('Certification updated.');
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  if (!certifications) {
    return <Loading />;
  }

  const needsAccount = certifications.items.some((item) => item.price > 0 && !item.completed);

  return (
    <>
      <h2>University</h2>
      <StatusMessage text={status.message} as="block" />

      {needsAccount && (
        <AccountSelect
          accounts={accounts}
          value={account}
          onChange={setAccount}
          label="Payment account"
        />
      )}

      {certifications.items.map((item) => (
        <article className="forum-post" key={item.level}>
          <b>
            {item.level}. {item.name}
          </b>
          <p>
            {item.description} · ${item.price}
          </p>

          {item.completed && <small>Completed</small>}

          {!item.completed && item.learning && (
            <button onClick={() => void act(item)}>Complete</button>
          )}

          {!item.completed && !item.learning && item.available && (
            <button disabled={item.price > 0 && !account} onClick={() => void act(item)}>
              Start
            </button>
          )}

          {!item.completed && !item.learning && !item.available && <small>Locked</small>}
        </article>
      ))}
    </>
  );
}
