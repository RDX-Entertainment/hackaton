import type { Visited } from '../../utils/api';
import { formatIp } from '../../utils/ip';

type BookmarkBarProps = {
  bookmarks: Visited[];
  current: number | null;
  onVisit: (ip: number) => void;
  onRemove: (ip: number) => void;
};

export function BookmarkBar({ bookmarks, current, onVisit, onRemove }: BookmarkBarProps) {
  if (bookmarks.length === 0) {
    return (
      <p className="marks marks--empty">
        Sin marcadores. Guarda una máquina desde su pestaña pública y aparecerá aquí.
      </p>
    );
  }

  return (
    <ul className="marks">
      {bookmarks.map((mark) => (
        <li key={mark.ip} className={mark.ip === current ? 'mark is-here' : 'mark'}>
          <button type="button" className="mark__go" onClick={() => onVisit(mark.ip)}>
            <span className="mark__star" aria-hidden="true">
              ★
            </span>
            {mark.name || formatIp(mark.ip)}
          </button>

          <button
            type="button"
            className="mark__drop"
            aria-label={`Quitar ${mark.name || formatIp(mark.ip)} de marcadores`}
            onClick={() => onRemove(mark.ip)}
          >
            ×
          </button>
        </li>
      ))}
    </ul>
  );
}
