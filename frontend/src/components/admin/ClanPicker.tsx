import type { KitClan } from '../../utils/api';

type ClanPickerProps = {
  clans: KitClan[];
  chosen: number[];
  onChange: (chosen: number[]) => void;
};

export function ClanPicker({ clans, chosen, onChange }: ClanPickerProps) {
  const toggle = (id: number) =>
    onChange(chosen.includes(id) ? chosen.filter((one) => one !== id) : [...chosen, id]);

  if (clans.length === 0) {
    return (
      <p className="clanpick__empty">
        No hay ninguna corporación creada todavía, así que un kit de corporación no lo vería
        nadie.
      </p>
    );
  }

  return (
    <div className="clanpick">
      {clans.map((clan) => (
        <button
          type="button"
          key={clan.id}
          className={chosen.includes(clan.id) ? 'clanpick__item is-active' : 'clanpick__item'}
          aria-pressed={chosen.includes(clan.id)}
          onClick={() => toggle(clan.id)}
        >
          <span className="clanpick__nick">{clan.nick}</span>
          {clan.name}
        </button>
      ))}
    </div>
  );
}
