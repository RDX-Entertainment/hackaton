export type AdminSection = {
  path: string;
  label: string;
  icon: string;
  /** Only the index route needs an exact match; the rest match by prefix. */
  end?: boolean;
};

export const ADMIN_SECTIONS: AdminSection[] = [
  { path: '/admin', label: 'Catálogo', icon: '▤', end: true },
  { path: '/admin/kits', label: 'Kits', icon: '▣' },
  { path: '/admin/sitios', label: 'Sitios', icon: '◈' },
  { path: '/admin/usuarios', label: 'Usuarios', icon: '⚇' },
];
