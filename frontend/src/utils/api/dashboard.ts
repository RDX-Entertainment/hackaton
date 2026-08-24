import { get } from './client';
import type { Dashboard, News, Page, UnknownRow } from './types';

export function getDashboard(): Promise<Dashboard> {
  return get<Dashboard>('/dashboard');
}

export function getNews(): Promise<News[]> {
  return get<News[]>('/news');
}

export function getServerStats(): Promise<UnknownRow> {
  return get<UnknownRow>('/stats');
}

/** The placeholder answer for a legacy page with no screen of its own yet. */
export function getLegacyPage(page: string): Promise<Page> {
  return get<Page>(`/pages/${page}`);
}

/** Reads any list endpoint as untyped rows, for the generic table screen. */
export function getResource(path: string): Promise<UnknownRow[]> {
  return get<UnknownRow[]>(path);
}
