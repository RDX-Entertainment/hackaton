import { get, put } from './client';
import type { CreatedRecord, WikiPage, WikiSummary } from './types';

export function listWikiPages(): Promise<WikiSummary[]> {
  return get<WikiSummary[]>('/wiki');
}

export function getWikiPage(slug: string): Promise<WikiPage> {
  return get<WikiPage>(`/wiki/${slug}`);
}

/** Saving always writes a new revision; there is no in-place edit. */
export function saveWikiPage(
  slug: string,
  title: string,
  content: string,
): Promise<CreatedRecord> {
  return put<CreatedRecord>(`/wiki/${slug}`, { title, content });
}
