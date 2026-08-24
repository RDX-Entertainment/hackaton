import { useCallback, useEffect, useState } from 'react';

import { StatusMessage } from '../components/StatusMessage';
import { missionsApi, type MissionCommand, type UnknownRow } from '../utils/api';
import { formatCell } from '../utils/format';
import { useStatusMessage } from '../utils/useStatusMessage';

/** The legacy status codes: 1 is on offer, 2 is accepted. */
const STATUS_OFFERED = 1;
const STATUS_ACCEPTED = 2;

export function MissionsPage() {
  const status = useStatusMessage();
  const [missions, setMissions] = useState<UnknownRow[]>([]);

  const load = useCallback(() => {
    missionsApi.listMissions().then(setMissions).catch(() => setMissions([]));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const command = async (id: number, action: MissionCommand) => {
    try {
      await missionsApi.commandMission(id, action);
      status.report(`Mission ${action}ed.`);
      load();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Missions</h2>
      <StatusMessage text={status.message} as="block" />

      {missions.map((mission) => {
        const id = Number(mission.id);
        const state = Number(mission.status);

        return (
          <article className="forum-post" key={id}>
            <b>
              Mission #{id} · level {formatCell(mission.level)}
            </b>
            <p>
              Prize: ${formatCell(mission.prize)} · status {formatCell(mission.status)}
            </p>

            {state === STATUS_OFFERED && (
              <button onClick={() => void command(id, 'accept')}>Accept</button>
            )}
            {state === STATUS_ACCEPTED && (
              <button onClick={() => void command(id, 'abort')}>Abort</button>
            )}
          </article>
        );
      })}
    </>
  );
}
