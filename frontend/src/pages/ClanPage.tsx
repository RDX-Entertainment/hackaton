import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { EmptyNotice, Loading } from '../components/Notice';
import { StatusMessage } from '../components/StatusMessage';
import { clansApi, type Clan, type ClanRequest, type ClanSummary } from '../utils/api';
import { formatMoney } from '../utils/format';
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
      status.report('Dentro: ya es del clan.');
      onChanged();
    } catch (error) {
      status.reportError(error);
    }
  };

  return (
    <>
      <h2>Clan</h2>
      <StatusMessage text={status.message} as="block" />

      <section className="clan__ficha">
        <span className="clan__chapa">{clan.nick}</span>

        <div style={{ minWidth: 0 }}>
          <h3 className="clan__nombre">{clan.name}</h3>
          <p className="clan__lema">
            {clan.description || 'Sin nada escrito todavía.'}
          </p>
        </div>

        <div className="clan__cifras">
          <span>
            <b>
              {clan.members.length}/{clan.slots}
            </b>
            <span>plantilla</span>
          </span>
          <span>
            <b>{formatMoney(clan.money)}</b>
            <span>caja</span>
          </span>
          <span>
            <b>{clan.power}</b>
            <span>fuerza</span>
          </span>
        </div>
      </section>

      <h3>Plantilla</h3>
      <ul className="clan__plantilla">
        {clan.members.map((member) => {
          const manda = member.auth_level >= ADMIN_AUTH_LEVEL;

          return (
            <li className={manda ? 'clan__miembro es-mando' : 'clan__miembro'} key={member.id}>
              <span className="nombre">{member.login}</span>
              <span className="puesto">{manda ? 'mando' : 'miembro'}</span>
            </li>
          );
        })}
      </ul>

      {clan.auth_level >= ADMIN_AUTH_LEVEL && (
        <>
          <h3>Quien pide entrar</h3>
          {requests.length ? (
            <div className="rejilla-tarjetas">
              {requests.map((request) => (
                <article className="tarjeta" key={request.id}>
                  <span className="tarjeta__rotulo">pide entrar</span>
                  <div className="tarjeta__cabecera" style={{ marginTop: '0.35rem' }}>
                    <span className="tarjeta__titulo">{request.login}</span>
                  </div>
                  <p className="tarjeta__texto">{request.message}</p>
                  <div className="tarjeta__acciones">
                    <span className="micro">solicitud</span>
                    <button onClick={() => void accept(request.id)}>Aceptar</button>
                  </div>
                </article>
              ))}
            </div>
          ) : (
            <EmptyNotice>Nadie ha pedido entrar.</EmptyNotice>
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

/**
 * Lo que ve quien no tiene clan.
 *
 * Antes esto abria con el formulario de fundacion ocupando media pantalla y, a
 * continuacion, **un campo suelto de «mensaje para pedir entrar» colgando fuera
 * de cualquier formulario**: un campo para pedir entrar a clanes que a lo mejor
 * no existen. Con el juego recien puesto no hay ninguno, asi que la pantalla era
 * un formulario que nadie pidio y una lista vacia.
 *
 * Ahora manda el estado en el que estas. Sin clanes, lo unico que hay es la
 * explicacion de que es un clan y el boton de fundar el primero. Con clanes, la
 * lista; y el mensaje de la solicitud sale **junto al clan al que se la pides**,
 * que es donde significa algo.
 */
function ClanDirectory({ status, onJoined }: DirectoryProps) {
  const [clans, setClans] = useState<ClanSummary[]>([]);
  const [fundando, setFundando] = useState(false);
  const [pidiendoA, setPidiendoA] = useState<number>();
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
      status.report('Clan fundado.');
      onJoined();
    } catch (error) {
      status.reportError(error);
    }
  };

  const join = async (id: number) => {
    try {
      await clansApi.requestToJoin(id, requestMessage);
      setPidiendoA(undefined);
      setRequestMessage('');
      status.report('Solicitud enviada. Te contestara quien mande alli.');
    } catch (error) {
      status.reportError(error);
    }
  };

  const fundacion = (
    <form className="clan__fundar" onSubmit={create}>
      <div className="ajustes__cabecera">
        <span className="ajustes__titulo">Fundar un clan</span>
        <span className="micro">tres cosas y ya existe</span>
      </div>

      <div className="ajustes__cuerpo">
        <div className="clan__campos">
          <label>
            Nombre
            <input
              value={name}
              maxLength={MAX_NAME_LENGTH}
              onChange={(event) => setName(event.target.value)}
              required
            />
          </label>

          <label>
            Siglas
            <input
              value={nick}
              maxLength={TAG_LENGTH}
              placeholder={'\u2014'.repeat(TAG_LENGTH)}
              onChange={(event) => setNick(event.target.value)}
              required
            />
          </label>
        </div>

        <label>
          De qué va
          <textarea
            value={description}
            placeholder="Lo que verá quien se lo encuentre en la lista."
            onChange={(event) => setDescription(event.target.value)}
          />
        </label>

        <p className="ajustes__nota">
          Las siglas son las {TAG_LENGTH} letras con las que se os nombra. No se cambian
          después.
        </p>

        <div className="correo__acciones">
          <StatusMessage text={status.message} />
          <button type="button" onClick={() => setFundando(false)}>
            Dejarlo
          </button>
          <button>Fundar</button>
        </div>
      </div>
    </form>
  );

  return (
    <>
      <h2>Clan</h2>

      {fundando && fundacion}

      {!fundando && !clans.length && (
        <div className="vacio">
          <div className="vacio__titulo">Todavía no hay ningún clan</div>
          <p className="vacio__texto">
            Un clan es un nombre, unas siglas con las que se os nombra y una caja común.
            Nadie ha fundado ninguno en esta partida, así que de momento no hay a quién
            pedirle entrar: el primero lo funda quien quiera.
          </p>
          <button onClick={() => setFundando(true)}>Fundar el primero</button>
          <StatusMessage text={status.message} as="block" />
        </div>
      )}

      {!fundando && clans.length > 0 && (
        <div className="tablon">
          <div className="tablon__barra">
            <span className="micro">
              {clans.length} {clans.length === 1 ? 'clan' : 'clanes'}
            </span>
            <button onClick={() => setFundando(true)}>Fundar el tuyo</button>
          </div>

          {clans.map((summary) => (
            <div className="tablon__hilo" key={summary.id}>
              <span className="tablon__titulo">
                <span className="clan__siglas">{summary.nick}</span> {summary.name}
              </span>
              <span className="tablon__quien">
                {summary.description || 'sin descripción'}
              </span>
              <span className="tablon__cuenta">
                <b>
                  {summary.members}/{summary.slots}
                </b>
                <span>plantilla</span>
              </span>

              {pidiendoA === summary.id ? (
                <div className="clan__solicitud">
                  <input
                    value={requestMessage}
                    placeholder="Por qué quieres entrar."
                    onChange={(event) => setRequestMessage(event.target.value)}
                  />
                  <button type="button" onClick={() => setPidiendoA(undefined)}>
                    Dejarlo
                  </button>
                  <button type="button" onClick={() => void join(summary.id)}>
                    Enviar
                  </button>
                </div>
              ) : (
                <button
                  className="clan__pedir"
                  type="button"
                  onClick={() => setPidiendoA(summary.id)}
                >
                  Pedir entrar
                </button>
              )}
            </div>
          ))}

          <StatusMessage text={status.message} as="block" />
        </div>
      )}
    </>
  );
}
