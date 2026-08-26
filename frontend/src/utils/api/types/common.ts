import type { User } from './auth';

export type UnknownRow = Record<string, unknown>;

export type Dashboard = {
  user: User;
  money: number;
  processes: number;
  unread_mail: number;
};

export type Page = {
  page: string;
  user_id: number;
  status: string;
};

export type News = {
  id: number;
  author: string;
  title: string;
  content: string;
  published_at: string;
  type: number;
};

export type QueuedProcess = {
  process_id: number;
};

export type CreatedRecord = {
  id: number;
};

