import { useEffect, useRef, useState } from 'react';
import { NavLink, useLocation } from 'react-router-dom';

import { NAV_GROUPS, type NavGroup } from '../../app/navigation';

/**
 * The categories, as icons in the top rail. Hovering one opens its pages.
 *
 * Hover alone would lock out keyboards and touch, so focus and click open the
 * same panel: `open` is one piece of state fed by three inputs rather than a
 * CSS `:hover` rule that only one of them can reach.
 */
export function NavMenu() {
  const [open, setOpen] = useState<string | null>(null);
  const location = useLocation();
  const closeTimer = useRef<number | undefined>(undefined);

  // Navigating closes whatever was open, so the panel does not hang around
  // over the page you just asked for.
  useEffect(() => {
    setOpen(null);
  }, [location.pathname]);

  useEffect(() => () => window.clearTimeout(closeTimer.current), []);

  /** A small grace period, so a diagonal trip to the panel does not close it. */
  const scheduleClose = () => {
    window.clearTimeout(closeTimer.current);
    closeTimer.current = window.setTimeout(() => setOpen(null), 140);
  };

  const cancelClose = () => window.clearTimeout(closeTimer.current);

  const isCurrent = (group: NavGroup) =>
    group.items.some((item) => location.pathname.startsWith(`/${item.path}`));

  return (
    <nav
      className="navmenu"
      aria-label="Secciones del juego"
      onKeyDown={(event) => event.key === 'Escape' && setOpen(null)}
    >
      {NAV_GROUPS.map((group) => {
        const expanded = open === group.title;

        return (
          <div
            className="navmenu__item"
            key={group.title}
            onMouseEnter={() => {
              cancelClose();
              setOpen(group.title);
            }}
            onMouseLeave={scheduleClose}
          >
            <button
              type="button"
              className={`navmenu__button${isCurrent(group) ? ' is-current' : ''}`}
              aria-expanded={expanded}
              aria-haspopup="true"
              onClick={() => setOpen(expanded ? null : group.title)}
              onFocus={() => setOpen(group.title)}
            >
              <span className="navmenu__icon" aria-hidden="true">
                {group.icon}
              </span>
              <span className="navmenu__label">{group.title}</span>
            </button>

            {expanded && (
              <div
                className={`navmenu__panel${group.pending ? ' navmenu__panel--pending' : ''}`}
                onMouseEnter={cancelClose}
                onMouseLeave={scheduleClose}
              >
                {group.pending && (
                  <p className="navmenu__note">Sin pantalla propia todavía.</p>
                )}

                <ul>
                  {group.items.map((item) => (
                    <li key={item.path}>
                      <NavLink
                        to={`/${item.path}`}
                        className={({ isActive }) =>
                          isActive ? 'navmenu__link is-active' : 'navmenu__link'
                        }
                      >
                        {item.label}
                      </NavLink>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        );
      })}
    </nav>
  );
}
