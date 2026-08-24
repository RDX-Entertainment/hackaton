import { useCallback, useEffect, useMemo, useState } from 'react';

import { AdminCard } from '../../components/admin/AdminCard';
import { SiteEditor } from '../../components/internet/SiteEditor';
import { Loading } from '../../components/Notice';
import { StatusMessage } from '../../components/StatusMessage';
import { adminApi, type Site } from '../../utils/api';
import { messageFrom } from '../../utils/errors';
import { formatCount } from '../../utils/format';
import { formatIp } from '../../utils/ip';
import { useStatusMessage } from '../../utils/useStatusMessage';

type Filter = 'all' | Site['kind'];

const FILTERS: { value: Filter; label: string }[] = [
  { value: 'all', label: 'todos' },
  { value: 'npc', label: 'NPC' },
  { value: 'player', label: 'jugadores' },
];

function keyOf(site: Site): string {
  return `${site.kind}:${site.id}`;
}

export function SitesPage() {
  const status = useStatusMessage();

  const [sites, setSites] = useState<Site[]>();
  const [denied, setDenied] = useState(false);
  const [chosen, setChosen] = useState<string | null>(null);
  const [filter, setFilter] = useState<Filter>('all');
  const [text, setText] = useState('');
  const [busy, setBusy] = useState(false);

  // Aparte del mensaje de la página: un rechazo del validador tiene que leerse al lado
  // del código que lo provocó, no al principio de una pantalla de metro y medio.
  const [refused, setRefused] = useState('');

  const load = useCallback(async () => {
    try {
      setSites(await adminApi.listSites());
    } catch {
      setDenied(true);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const shown = useMemo(() => {
    if (!sites) {
      return [];
    }

    const terms = text.trim().toLowerCase().split(/\s+/).filter(Boolean);

    return sites.filter((site) => {
      if (filter !== 'all' && site.kind !== filter) {
        return false;
      }

      if (!terms.length) {
        return true;
      }

      const words = `${site.name} ${formatIp(site.ip)}`.toLowerCase();
      return terms.every((term) => words.includes(term));
    });
  }, [sites, filter, text]);

  const current = sites?.find((site) => keyOf(site) === chosen) ?? null;

  const save = async (web: string) => {
    if (!current) {
      return;
    }

    setBusy(true);
    status.clear();
    setRefused('');

    try {
      await adminApi.saveSite(current.kind, current.id, web);
      await load();
      status.report(`${current.name} publicado.`);
    } catch (error) {
      setRefused(messageFrom(error));
    } finally {
      setBusy(false);
    }
  };

  if (denied) {
    return (
      <>
        <h2>Sitios</h2>
        <p className="muted">
          Esta sección no existe para tu cuenta. El servidor responde 404 a quien no es
          administrador, igual que a una ruta inventada.
        </p>
      </>
    );
  }

  if (!sites) {
    return <Loading />;
  }

  const empty = sites.filter((site) => !site.web.trim()).length;

  return (
    <>
      <h2>Sitios</h2>
      <StatusMessage text={status.message} as="block" />

      <AdminCard
        prefix="Net"
        brand="Host"
        note={`${sites.length} páginas · ${empty} sin contenido`}
      >
        <div className="sites">
          <div className="sites__list">
            <div className="sites__tools">
              <div className="sites__search">
                <span aria-hidden="true">⌕</span>
                <input
                  value={text}
                  placeholder="nombre o dirección…"
                  onChange={(event) => setText(event.target.value)}
                />
              </div>

              <div className="chips">
                {FILTERS.map((one) => (
                  <button
                    type="button"
                    key={one.value}
                    className={filter === one.value ? 'chip is-active' : 'chip'}
                    onClick={() => setFilter(one.value)}
                  >
                    {one.label}{' '}
                    <b>
                      {one.value === 'all'
                        ? sites.length
                        : sites.filter((site) => site.kind === one.value).length}
                    </b>
                  </button>
                ))}
              </div>
            </div>

            <ul className="sites__rows">
              {shown.map((site) => (
                <li key={keyOf(site)}>
                  <button
                    type="button"
                    className={keyOf(site) === chosen ? 'srow is-active' : 'srow'}
                    onClick={() => {
                      status.clear();
                      setRefused('');
                      setChosen(keyOf(site));
                    }}
                  >
                    <span className={`srow__kind srow__kind--${site.kind}`}>
                      {site.kind === 'npc' ? 'npc' : 'jug'}
                    </span>

                    <span className="srow__text">
                      <span className="srow__name">{site.name || 'sin nombre'}</span>
                      <span className="srow__ip">{formatIp(site.ip)}</span>
                    </span>

                    <span className={site.web.trim() ? 'srow__bytes' : 'srow__bytes is-empty'}>
                      {site.web.trim() ? `${formatCount(site.web.length)} B` : 'vacío'}
                    </span>
                  </button>
                </li>
              ))}

              {shown.length === 0 && <li className="sites__none">Ninguno encaja.</li>}
            </ul>
          </div>

          <div className="sites__edit">
            {current ? (
              <SiteEditor
                ip={current.ip}
                name={current.name}
                web={current.web}
                busy={busy}
                error={refused}
                onSave={(web) => void save(web)}
              />
            ) : (
              <p className="sites__idle">
                Elige una página de la lista. Se editan igual las de un NPC y las de un
                jugador: lo que cambia es de quién es la máquina que la sirve.
              </p>
            )}
          </div>
        </div>
      </AdminCard>
    </>
  );
}
