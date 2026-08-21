import type { User } from './store';
export type Session = { token: string; user: User };
const request = async <T>(path: string, init: RequestInit = {}): Promise<T> => {
  const session = JSON.parse(localStorage.getItem('legacy.session') || '{}') as Partial<Session>;
  const response = await fetch(`/api${path}`, { ...init, headers: { 'content-type': 'application/json', ...(session.token ? { authorization: `Bearer ${session.token}` } : {}), ...init.headers } });
  if (!response.ok) { const body = await response.json().catch(() => ({})); throw new Error(body.error || 'Request failed'); }
  return response.json() as Promise<T>;
};
export const api = { login: (username: string, password: string) => request<Session>('/auth/login', { method: 'POST', body: JSON.stringify({ username, password }) }), register: (username: string, password: string, email: string) => request<Session>('/auth/register', { method: 'POST', body: JSON.stringify({ username, password, email }) }), dashboard: () => request<Dashboard>('/dashboard'), page: (page: string) => request<Page>(`/pages/${page}`), resource: (path: string) => request<Record<string, unknown>[]>(path), sendMail: (to:string, subject:string, text:string) => request<{id:number}>('/mails', {method:'POST',body:JSON.stringify({to,subject,text})}), transfer: (from_account:number,password:string,to_account:number,amount:number) => request<{amount:number}>('/finances/transfers', {method:'POST',body:JSON.stringify({from_account,password,to_account,amount})}) };
export type Dashboard = { user: User; money: number; processes: number; unread_mail: number };
export type Page = { page: string; user_id: number; status: string };
