import type { Kit, KitScope } from '../../utils/api';
import { formatCount, formatMoney } from '../../utils/format';
import { PartIcon } from './PartIcon';

const SCOPE_LABEL: Record<KitScope, string> = {
  shop: 'tienda',
  clan: 'corporación',
  marketplace: 'mercado',
  hidden: 'oculto',
};

type KitCardProps = {
  kit: Kit;
  busy: boolean;
  onEdit: (kit: Kit) => void;
  onRemove: (kit: Kit) => void;
  onDefault: (kit: Kit) => void;
};

export function KitCard({ kit, busy, onEdit, onRemove, onDefault }: KitCardProps) {
  const board = kit.items.find((item) => item.kind === 'mobo');
  const rest = kit.items.filter((item) => item.kind !== 'mobo');

  const saving =
    kit.totals.parts_price > 0 ? Math.round((1 - kit.price / kit.totals.parts_price) * 100) : 0;

  return (
    <article className={kit.listed ? 'kititem' : 'kititem is-retired'}>
      <header className="kititem__head">
        <div className="kititem__title">
          <h4>{kit.name}</h4>
          <span className="kititem__id">#{kit.id}</span>
        </div>

        <span className={`scope scope--${kit.scope}`}>{SCOPE_LABEL[kit.scope]}</span>

        {kit.is_default && <span className="kititem__default">arranque</span>}
      </header>

      {kit.description && <p className="kititem__desc">{kit.description}</p>}

      <div className="kititem__board">
        <PartIcon kind="mobo" className="kititem__icon" />
        {board ? (
          <span>
            {board.name} <small>{board.chipset ?? 'sin chipset'} · DDR{board.generation}</small>
          </span>
        ) : (
          <span className="kititem__nofboard">sin placa</span>
        )}

        {board && <span className={`tier tier--${board.tier}`}>{board.tier}</span>}
      </div>

      <ul className="kititem__parts">
        {rest.map((item) => (
          <li key={item.part}>
            <PartIcon kind={item.kind} className="kititem__partsicon" />
            {item.amount > 1 && <b>{item.amount}×</b>}
            {item.name}
          </li>
        ))}
      </ul>

      <dl className="kititem__totals">
        <div>
          <dt>cpu</dt>
          <dd>{formatCount(kit.totals.cpu)}</dd>
        </div>
        <div>
          <dt>ram</dt>
          <dd>{formatCount(kit.totals.ram)}</dd>
        </div>
        <div>
          <dt>hdd</dt>
          <dd>{formatCount(kit.totals.hdd)}</dd>
        </div>
        <div>
          <dt>net</dt>
          <dd>{formatCount(kit.totals.net)}</dd>
        </div>
      </dl>

      {kit.clans.length > 0 && (
        <p className="kititem__clans">
          {kit.clans.map((clan) => (
            <span key={clan.id}>{clan.nick}</span>
          ))}
        </p>
      )}

      <footer className="kititem__foot">
        <span className="kititem__price">{formatMoney(kit.price)}</span>

        {saving > 0 && <span className="kititem__saving">−{saving}%</span>}

        {!kit.listed && <span className="kititem__retired">retirado</span>}

        <span className="kititem__actions">
          <button
            type="button"
            disabled={busy || kit.is_default}
            onClick={() => onDefault(kit)}
          >
            {kit.is_default ? 'es el de arranque' : 'usar al registrarse'}
          </button>
          <button type="button" disabled={busy} onClick={() => onEdit(kit)}>
            editar
          </button>
          <button type="button" disabled={busy} onClick={() => onRemove(kit)}>
            borrar
          </button>
        </span>
      </footer>
    </article>
  );
}
