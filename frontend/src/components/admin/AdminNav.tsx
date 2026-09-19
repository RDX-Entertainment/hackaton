import { Link, NavLink } from 'react-router-dom';

import { signedOut } from '../../app/authSlice';
import { useAppDispatch, useAppSelector } from '../../app/hooks';
import { ADMIN_SECTIONS } from '../../app/adminNavigation';
import { t } from '../../i18n';
import { LanguageSelect } from '../shell/LanguageSelect';

export function AdminNav() {
  const dispatch = useAppDispatch();
  const user = useAppSelector((state) => state.auth.user);

  return (
    <header className="topbar topbar--admin">
      <Link className="topbar__brand" to="/admin">
        <span className="topbar__mark" aria-hidden="true">
          A
        </span>
        <span className="topbar__name">
          panel<b>admin</b>
        </span>
      </Link>

      <nav className="adminnav" aria-label={t('admin.sections') ?? 'Secciones del panel'}>
        {ADMIN_SECTIONS.map((section) => (
          <NavLink
            key={section.path}
            to={section.path}
            end={section.end}
            className={({ isActive }) => (isActive ? 'adminnav__link is-active' : 'adminnav__link')}
          >
            <span className="adminnav__icon" aria-hidden="true">
              {section.icon}
            </span>
            {t(`admin.${section.path === '/admin'
              ? 'catalog'
              : section.path.split('/').filter(Boolean).at(-1)}`)
              ?? section.label}
          </NavLink>
        ))}
      </nav>

      <div className="topbar__session">
        <Link className="topbar__back" to="/">
          {t('admin.back') ?? 'volver al juego'}
        </Link>

        <LanguageSelect />

        {user && (
          <span className="topbar__user">
            <span className="topbar__user-label">adm</span>
            <span className="topbar__user-name">{user.login}</span>
          </span>
        )}

        <button className="topbar__logout" type="button" onClick={() => dispatch(signedOut())}>
          {t('shell.logout') ?? 'salir'}
        </button>
      </div>
    </header>
  );
}
