import { tr, useIdioma } from '../i18n/text';
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
  useIdioma();
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
  useIdioma();
  const accept = async (id: number) => {
    try {
      await clansApi.acceptJoinRequest(id);
      status.report(tr("Dentro: ya es del clan."));
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
            {clan.description || tr("Sin nada escrito todavía.")}
          </p>
        </div>

        <div className="clan__cifras">
          <span>
            <b>
              {clan.members.length}/{clan.slots}
            </b>
            <span>{tr("plantilla")}</span>
          </span>
          <span>
            <b>{formatMoney(clan.money)}</b>
            <span>{tr("caja")}</span>
          </span>
          <span>
            <b>{clan.power}</b>
            <span>{tr("fuerza")}</span>
          </span>
        </div>
      </section>

      <h3>{tr("Plantilla")}</h3>
      <ul className="clan__plantilla">
        {clan.members.map((member) => {
          const manda = member.auth_level >= ADMIN_AUTH_LEVEL;

          return (
            <li className={manda ? 'clan__miembro es-mando' : 'clan__miembro'} key={member.id}>
              <span className="nombre">{member.login}</span>
              <span className="puesto">{manda ? tr("mando") : tr("miembro")}</span>
            </li>
          );
        })}
      </ul>

      {clan.auth_level >= ADMIN_AUTH_LEVEL && (
        <>
          <h3>{tr("Quien pide entrar")}</h3>
          {requests.length ? (
            <div className="rejilla-tarjetas">
              {requests.map((request) => (
                <article className="tarjeta" key={request.id}>
                  <span className="tarjeta__rotulo">{tr("pide entrar")}</span>
                  <div className="tarjeta__cabecera" style={{ marginTop: '0.35rem' }}>
                    <span className="tarjeta__titulo">{request.login}</span>
                  </div>
                  <p className="tarjeta__texto">{request.message}</p>
                  <div className="tarjeta__acciones">
                    <span className="micro">{tr("solicitud")}</span>
                    <button onClick={() => void accept(request.id)}>{tr("Aceptar")}</button>
                  </div>
                </article>
              ))}
            </div>
          ) : (
            <EmptyNotice>{tr("Nadie ha pedido entrar.")}</EmptyNotice>
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
  useIdioma();
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
      status.report(tr("Clan fundado."));
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
      status.report(tr("Solicitud enviada. Te contestara quien mande alli."));
    } catch (error) {
      status.reportError(error);
    }
  };

  const fundacion = (
    <form className="clan__fundar" onSubmit={create}>
      <div className="ajustes__cabecera">
        <span className="ajustes__titulo">{tr("Fundar un clan")}</span>
        <span className="micro">{tr("tres cosas y ya existe")}</span>
      </div>

      <div className="ajustes__cuerpo">
        <div className="clan__campos">
          <label>
            {tr("Nombre")} <input
              value={name}
              maxLength={MAX_NAME_LENGTH}
              onChange={(event) => setName(event.target.value)}
              required
            />
          </label>

          <label>
            {tr("Siglas")} <input
              value={nick}
              maxLength={TAG_LENGTH}
              placeholder={'\u2014'.repeat(TAG_LENGTH)}
              onChange={(event) => setNick(event.target.value)}
              required
            />
          </label>
        </div>

        <label>
          {tr("De qué va")} <textarea
            value={description}
            placeholder={tr("Lo que verá quien se lo encuentre en la lista.")}
            onChange={(event) => setDescription(event.target.value)}
          />
        </label>

        <p className="ajustes__nota">
          {tr("Las siglas son las")} {TAG_LENGTH} {tr("letras con las que se os nombra. No se cambian después.")} </p>

        <div className="correo__acciones">
          <StatusMessage text={status.message} />
          <button type="button" onClick={() => setFundando(false)}>
            {tr("Dejarlo")} </button>
          <button>{tr("Fundar")}</button>
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
          <div className="vacio__titulo">{tr("Todavía no hay ningún clan")}</div>
          <p className="vacio__texto">
            {tr("Un clan es un nombre, unas siglas con las que se os nombra y una caja común. Nadie ha fundado ninguno en esta partida, así que de momento no hay a quién pedirle entrar: el primero lo funda quien quiera.")} </p>
          <button onClick={() => setFundando(true)}>{tr("Fundar el primero")}</button>
          <StatusMessage text={status.message} as="block" />
        </div>
      )}

      {!fundando && clans.length > 0 && (
        <div className="tablon">
          <div className="tablon__barra">
            <span className="micro">
              {clans.length} {clans.length === 1 ? 'clan' : tr("clanes")}
            </span>
            <button onClick={() => setFundando(true)}>{tr("Fundar el tuyo")}</button>
          </div>

          {clans.map((summary) => (
            <div className="tablon__hilo" key={summary.id}>
              <span className="tablon__titulo">
                <span className="clan__siglas">{summary.nick}</span> {summary.name}
              </span>
              <span className="tablon__quien">
                {summary.description || tr("sin descripción")}
              </span>
              <span className="tablon__cuenta">
                <b>
                  {summary.members}/{summary.slots}
                </b>
                <span>{tr("plantilla")}</span>
              </span>

              {pidiendoA === summary.id ? (
                <div className="clan__solicitud">
                  <input
                    value={requestMessage}
                    placeholder={tr("Por qué quieres entrar.")}
                    onChange={(event) => setRequestMessage(event.target.value)}
                  />
                  <button type="button" onClick={() => setPidiendoA(undefined)}>
                    {tr("Dejarlo")} </button>
                  <button type="button" onClick={() => void join(summary.id)}>
                    {tr("Enviar")} </button>
                </div>
              ) : (
                <button
                  className="clan__pedir"
                  type="button"
                  onClick={() => setPidiendoA(summary.id)}
                >
                  {tr("Pedir entrar")} </button>
              )}
            </div>
          ))}

          <StatusMessage text={status.message} as="block" />
        </div>
      )}
    </>
  );
}
