import type { User } from './store';
export type Session = { token: string; user: User };
const request = async <T>(path: string, init: RequestInit = {}): Promise<T> => {
  const session = JSON.parse(localStorage.getItem('legacy.session') || '{}') as Partial<Session>;
  const response = await fetch(`/api${path}`, { ...init, headers: { 'content-type': 'application/json', ...(session.token ? { authorization: `Bearer ${session.token}` } : {}), ...init.headers } });
  if (!response.ok) { const body = await response.json().catch(() => ({})); throw new Error(body.error || 'Request failed'); }
  return response.json() as Promise<T>;
};
export const api = { login: (username: string, password: string) => request<Session>('/auth/login', { method: 'POST', body: JSON.stringify({ username, password }) }), register: (username: string, password: string, email: string) => request<Session>('/auth/register', { method: 'POST', body: JSON.stringify({ username, password, email }) }), dashboard: () => request<Dashboard>('/dashboard'), page: (page: string) => request<Page>(`/pages/${page}`), resource: (path: string) => request<Record<string, unknown>[]>(path), sendMail: (to:string, subject:string, text:string) => request<{id:number}>('/mails', {method:'POST',body:JSON.stringify({to,subject,text})}), transfer: (from_account:number,password:string,to_account:number,amount:number) => request<{amount:number}>('/finances/transfers', {method:'POST',body:JSON.stringify({from_account,password,to_account,amount})}), wikiIndex: () => request<WikiSummary[]>('/wiki'), wiki: (slug:string) => request<WikiPage>(`/wiki/${slug}`), saveWiki: (slug:string,title:string,content:string) => request<{id:number}>(`/wiki/${slug}`, {method:'PUT',body:JSON.stringify({title,content})}), forumTopics: () => request<ForumTopic[]>('/forum/topics'), forumTopic: (id:number) => request<ForumTopicDetail>(`/forum/topics/${id}`), createTopic: (title:string,content:string) => request<{id:number}>('/forum/topics',{method:'POST',body:JSON.stringify({title,content})}), replyTopic: (id:number,content:string) => request<{id:number}>(`/forum/topics/${id}/posts`,{method:'POST',body:JSON.stringify({content})}) };
export type Dashboard = { user: User; money: number; processes: number; unread_mail: number };
export type Page = { page: string; user_id: number; status: string };
export type WikiSummary = { slug:string; title:string; updated_at:string };
export type WikiPage = WikiSummary & { content:string; author:string|null; created_at:string };
export type ForumTopic = { id:number; title:string; author:string; posts:number; updated_at:string };
export type ForumTopicDetail = { id:number; title:string; author:string; created_at:string; posts:{id:number;content:string;author:string;created_at:string}[] };
