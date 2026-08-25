import { useCallback, useEffect, useState, type FormEvent } from 'react';

import { AdminCard } from '../../components/admin/AdminCard';
import { Loading } from '../../components/Notice';
import { StatusMessage } from '../../components/StatusMessage';
import { adminApi, type ActionInfo, type Organization, type SiteGrant } from '../../utils/api';
import { useStatusMessage } from '../../utils/useStatusMessage';

const SCOPES = [
  { value: 'all_sites', label: 'Todas las webs', needs: 'nada' },
  { value: 'all_organizations', label: 'Todas las corporaciones', needs: 'nada' },
  { value: 'organization', label: 'Una corporación', needs: 'organization' },
  { value: 'site', label: 'Una web', needs: 'site' },
];

const FAMILIES = [
  {
    value: 'normal',
    label: 'Normales',
    hint: 'Enlazar, navegar y cobrar. El diálogo de pago lo pinta el juego, fuera de la página.',
  },
  {
    value: 'integrated',
    label: 'Integradas',
    hint: 'La interfaz vive dentro del HTML del sitio: sus campos, su CSS, sus huecos.',
  },
];

/**
 * Quién puede usar cada orden de un sitio.
 *
 * El motor no sabe qué es un banco: sabe que hay dos familias y que cada concesión dice a
 * quién alcanza. Que hoy la banca integrada solo la tengan los bancos es una fila de esta
 * tabla, no una regla del código, y se cambia desde aquí.
 */
export function SiteActionsPage() {
  const status = useStatusMessage();

  const [actions, setActions] = useState<ActionInfo[]>();
  const [grants, setGrants] = useState<SiteGrant[]>([]);
  const [organizations, setOrganizations] = useState<Organization[]>([]);
  const [busy, setBusy] = useState(false);

  const [family, setFamily] = useState('integrated');
  const [action, setAction] = useState('*');
  const [scope, setScope] = useState('organization');
  const [target, setTarget] = useState(0);

  const load = useCallback(async () => {
    const [list, catalogue, orgs] = await Promise.all([
      adminApi.listSiteGrants(),
      adminApi.listSiteActions(),
      adminApi.listOrganizations(),
    ]);

    setGrants(list);
    setActions(catalogue);
    setOrganizations(orgs);
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

  if (!actions) {
    return <Loading />;
  }

  const needs = SCOPES.find((one) => one.value === scope)?.needs ?? 'nada';
  const nodes = organizations.flatMap((organization) =>
    organization.networks.flatMap((network) =>
      network.npcs.map((npc) => ({ npc, organization })),
    ),
  );

  const grant = (event: FormEvent) => {
    event.preventDefault();
    void run(async () => {
      await adminApi.createSiteGrant({
        action,
        family,
        scope,
        organization_id: needs === 'organization' ? target : null,
        npc_id: needs === 'site' ? target : null,
      });

      return 'Concesión creada.';
    });
  };

  const drop = (id: number) =>
    void run(async () => {
      await adminApi.removeSiteGrant(id);
      return 'Concesión retirada.';
    });

  return (
    <>
      <h2>Acciones de sitio</h2>
      <p className="muted">
        Lo que puede usar la web de una máquina. Una concesión con acción <code>*</code> abre la
        familia entera.
      </p>
      <StatusMessage text={status.message} as="block" />

      <div className="grants">
        {FAMILIES.map((one) => (
          <AdminCard
            key={one.value}
            prefix={one.value === 'normal' ? '01/' : '02/'}
            brand={one.label.toLowerCase()}
            note={`${grants.filter((row) => row.family === one.value).length} concesiones`}
            accent={one.value === 'integrated' ? 'lime' : undefined}
          >
            <p className="org-infra__intro">{one.hint}</p>

            <div className="grants__verbs">
              {actions
                .filter((row) => row.family === one.value)
                .map((row) => (
                  <span key={row.verb} title={row.help}>
                    {row.verb}
                  </span>
                ))}
            </div>

            {grants
              .filter((row) => row.family === one.value)
              .map((row) => (
                <div className="grants__row" key={row.id}>
                  <b>{row.action}</b>
                  <span>{SCOPES.find((s) => s.value === row.scope)?.label ?? row.scope}</span>
                  <em>{row.target_name ?? '—'}</em>
                  <button type="button" disabled={busy} onClick={() => drop(row.id)}>
                    quitar
                  </button>
                </div>
              ))}

            {!grants.some((row) => row.family === one.value) && (
              <p className="org-empty">Nadie puede usar estas órdenes.</p>
            )}
          </AdminCard>
        ))}
      </div>

      <AdminCard prefix="03/" brand="conceder" note="nueva">
        <form className="grants__form" onSubmit={grant}>
          <label>
            Familia
            <select
              value={family}
              onChange={(event) => {
                setFamily(event.target.value);
                setAction('*');
              }}
            >
              {FAMILIES.map((one) => (
                <option key={one.value} value={one.value}>
                  {one.label}
                </option>
              ))}
            </select>
          </label>

          <label>
            Acción
            <select value={action} onChange={(event) => setAction(event.target.value)}>
              <option value="*">todas (*)</option>
              {actions
                .filter((one) => one.family === family)
                .map((one) => (
                  <option key={one.verb} value={one.verb}>
                    {one.verb}
                  </option>
                ))}
            </select>
          </label>

          <label>
            A quién
            <select
              value={scope}
              onChange={(event) => {
                setScope(event.target.value);
                setTarget(0);
              }}
            >
              {SCOPES.map((one) => (
                <option key={one.value} value={one.value}>
                  {one.label}
                </option>
              ))}
            </select>
          </label>

          {needs === 'organization' && (
            <label>
              Corporación
              <select value={target} onChange={(event) => setTarget(Number(event.target.value))}>
                <option value={0}>Elige una</option>
                {organizations.map((one) => (
                  <option key={one.id} value={one.id}>
                    {one.name}
                  </option>
                ))}
              </select>
            </label>
          )}

          {needs === 'site' && (
            <label>
              Web
              <select value={target} onChange={(event) => setTarget(Number(event.target.value))}>
                <option value={0}>Elige una</option>
                {nodes.map(({ npc, organization }) => (
                  <option key={npc.id} value={npc.id}>
                    {npc.name} · {organization.name}
                  </option>
                ))}
              </select>
            </label>
          )}

          <button className="is-primary" disabled={busy || (needs !== 'nada' && !target)}>
            Conceder
          </button>
        </form>
      </AdminCard>
    </>
  );
}
