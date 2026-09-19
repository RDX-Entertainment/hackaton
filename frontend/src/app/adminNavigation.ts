import { tr } from '../i18n/text';
export type AdminSection = {
  path: string;
  label: string;
  icon: string;
  /** Only the index route needs an exact match; the rest match by prefix. */
  end?: boolean;
};

export const ADMIN_SECTIONS: AdminSection[] = [
  { path: '/admin', get label() { return tr("Catálogo"); }, icon: '▤', end: true },
  { path: '/admin/kits', label: 'Kits', icon: '▣' },
  { path: '/admin/organizaciones', get label() { return tr("Organizaciones"); }, icon: '◉' },
  { path: '/admin/sitios', get label() { return tr("Sitios"); }, icon: '◈' },
  { path: '/admin/acciones', get label() { return tr("Acciones"); }, icon: '⌁' },
  { path: '/admin/usuarios', get label() { return tr("Usuarios"); }, icon: '⚇' },
  { path: '/admin/software', label: 'Software', icon: '▣' },
  { path: '/admin/tutorial', label: 'Tutorial', icon: '➤' },
];
