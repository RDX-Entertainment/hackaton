/**
 * The sidebar. Twenty-four flat links was a wall of text with nothing to tell
 * you where you were, so they are grouped by what you go there to do.
 *
 * `path` doubles as the key the screen registry looks up, so adding a page
 * means a line here and an entry in `pageRegistry`.
 */
export type NavItem = {
  path: string;
  label: string;
};

export type NavGroup = {
  title: string;
  /**
   * Drawn in the top rail. Plain glyphs rather than an icon font or an SVG set:
   * the whole interface is monospace, and a pictogram would be the only thing
   * on screen that is not a character.
   */
  icon: string;
  items: NavItem[];
  /**
   * Marks a group whose pages have no screen of their own yet and answer with
   * a placeholder. Saying so beats letting someone click and wonder.
   */
  pending?: boolean;
};

export const NAV_GROUPS: NavGroup[] = [
  {
    title: 'Operación',
    icon: '⌁',
    items: [
      { path: 'internet', label: 'Internet' },
      { path: 'software', label: 'Software' },
      { path: 'hardware', label: 'Hardware' },
      { path: 'processes', label: 'Processes' },
    ],
  },
  {
    title: 'Progreso',
    icon: '◈',
    items: [
      { path: 'missions', label: 'Missions' },
      { path: 'research', label: 'Research' },
      { path: 'university', label: 'University' },
      { path: 'certs', label: 'Certifications' },
    ],
  },
  {
    title: 'Dinero',
    icon: '₿',
    items: [
      { path: 'finances', label: 'Finances' },
      { path: 'bitcoin', label: 'Bitcoin' },
    ],
  },
  {
    title: 'Gente',
    icon: '⚇',
    items: [
      { path: 'clan', label: 'Clan' },
      { path: 'mail', label: 'Mail' },
      { path: 'forum', label: 'Forum' },
      { path: 'wiki', label: 'Wiki' },
    ],
  },
  {
    title: 'Registro',
    icon: '▤',
    items: [
      { path: 'log', label: 'Log' },
      { path: 'list', label: 'List' },
      { path: 'ranking', label: 'Ranking' },
      { path: 'stats', label: 'Stats' },
    ],
  },
  {
    title: 'Cuenta',
    icon: '◎',
    items: [
      { path: 'profile', label: 'Profile' },
      { path: 'settings', label: 'Settings' },
    ],
  },
  {
    title: 'Sin portar',
    icon: '∅',
    pending: true,
    items: [
      { path: 'fame', label: 'Fame' },
      { path: 'doom', label: 'Doom' },
      { path: 'riddle', label: 'Riddle' },
      { path: 'war', label: 'War' },
      { path: 'webserver', label: 'Webserver' },
    ],
  },
];

/** Wiki and forum are in the sidebar but have hand-written routes of their own. */
const HAND_WRITTEN = new Set(['wiki', 'forum']);

/** The legacy pages, one generated route each. */
export const LEGACY_ITEMS: NavItem[] = NAV_GROUPS.flatMap((group) => group.items).filter(
  (item) => !HAND_WRITTEN.has(item.path),
);
