import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { DataTable } from '../components/DataTable';
import { EmptyNotice, Loading } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import { clansApi, type Clan, type ClanRequest, type ClanSummary } from '../utils/api';
import { useStatusMessage } from '../utils/useStatusMessage';

const MAX_NAME_LENGTH = 25;
const TAG_LENGTH = 3;

/** From this auth level up, a member may read and accept join requests. */
const ADMIN_AUTH_LEVEL = 2;

export function ClanPage() {
  const status = useStatusMessage();

  // undefined means "still loading"; null means "in no clan".
  const [clan, setClan] = useState<Clan | null | undefined>();
  const [requests, setRequests] = useState<ClanRequest[]>([]);

  const load = useCallback(() => {
    clansApi.getMyClan().then((mine) => {
      setClan(mine);

      if (mine && mine.auth_level >= ADMIN_AUTH_LEVEL) {
        clansApi.listJoinRequests().then(setRequests).catch(() => setRequests([]));
      }
    });
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  if (clan === undefined) {
    return <Loading />;
  }

  if (clan) {
    return (
      <ClanMembership clan={clan} requests={requests} status={status} onChanged={load} />
    );
  }

  return <ClanDirectory status={status} onJoined={load} />;
}

type MembershipProps = {
  clan: Clan;
  requests: ClanRequest[];
  status: ReturnType<typeof useStatusMessage>;
  onChanged: () => void;
};

/** What a member sees: the roster, and the pending requests if they may act on them. */
function ClanMembership({ clan, requests, status, onChanged }: MembershipProps) {
  const accept = async (id: number) => {
    try {
      await clansApi.acceptJoinRequest(id);
      status.report('Request accepted.');
      onChanged();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>
        [{clan.nick}] {clan.name}
      </h2>
      <p>{clan.description}</p>
      <StatusMessage text={status.message} as="block" />

      <DataTable rows={clan.members} />

      {clan.auth_level >= ADMIN_AUTH_LEVEL && (
        <>
          <h3>Join requests</h3>
          {requests.length ? (
            requests.map((request) => (
              <article className="forum-post" key={request.id}>
                <b>{request.login}</b>
                <p>{request.message}</p>
                <button onClick={() => void accept(request.id)}>Accept</button>
              </article>
            ))
          ) : (
            <EmptyNotice>No pending requests.</EmptyNotice>
          )}
        </>
      )}
    </>
  );
}

type DirectoryProps = {
  status: ReturnType<typeof useStatusMessage>;
  onJoined: () => void;
};

/** What a clanless player sees: create one, or ask to join an existing one. */
function ClanDirectory({ status, onJoined }: DirectoryProps) {
  const [clans, setClans] = useState<ClanSummary[]>([]);
  const [name, setName] = useState('');
  const [nick, setNick] = useState('');
  const [description, setDescription] = useState('');
  const [requestMessage, setRequestMessage] = useState('');

  useEffect(() => {
    clansApi.listClans().then(setClans).catch(() => setClans([]));
  }, []);

  const create = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await clansApi.createClan(name, nick, description);
      status.report('Clan created.');
      onJoined();
    } catch (error) {
      status.reportError(error);
    }
  };

  const join = async (id: number) => {
    try {
      await clansApi.requestToJoin(id, requestMessage);
      status.report('Join request sent.');
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Clan</h2>

      <form className="compose" onSubmit={create}>
        <label>
          Name
          <input
            value={name}
            maxLength={MAX_NAME_LENGTH}
            onChange={(event) => setName(event.target.value)}
            required
          />
        </label>

        <label>
          Tag ({TAG_LENGTH} characters)
          <input
            value={nick}
            maxLength={TAG_LENGTH}
            onChange={(event) => setNick(event.target.value)}
            required
          />
        </label>

        <label>
          Description
          <textarea
            value={description}
            onChange={(event) => setDescription(event.target.value)}
          />
        </label>

        <button>Create clan</button>
      </form>

      <label>
        Message for a join request
        <input
          value={requestMessage}
          onChange={(event) => setRequestMessage(event.target.value)}
        />
      </label>
      <StatusMessage text={status.message} />

      {clans.map((summary) => (
        <article className="forum-post" key={summary.id}>
          <b>
            [{summary.nick}] {summary.name}
          </b>
          <p>
            {summary.description} · {summary.members}/{summary.slots}
          </p>
          <button onClick={() => void join(summary.id)}>Request join</button>
        </article>
      ))}
    </>
  );
}
