import { GenericResourcePage } from '../pages/GenericResourcePage';
import type { NavItem } from './navigation';
import { screenFor } from './pageRegistry';

/**
 * Resolves one sidebar entry to its screen. A path with a registered screen
 * gets it; the rest fall back to the generic table.
 *
 * Keeping the lookup in a component of its own is what lets the fallback use
 * hooks safely: the old version returned early past a `useLocation()` call.
 */
export function LegacyRoute({ path, label }: NavItem) {
  const Screen = screenFor(path);

  if (Screen) {
    return <Screen />;
  }

  return <GenericResourcePage page={path} label={label} />;
}
