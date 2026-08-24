import { del, get, post, put } from './client';
import type {
  HackKind,
  HackResult,
  QueuedProcess,
  Remote,
  RemoteFile,
  Target,
  TransferDirection,
  Visited,
} from './types';

export function listTargets(): Promise<Target[]> {
  return get<Target[]>('/internet/targets');
}

export function navigate(ip: number): Promise<Remote> {
  return get<Remote>(`/internet/${ip}`);
}

export function login(
  ip: number,
  user: string,
  password: string,
): Promise<{ access: boolean; name: string }> {
  return post('/internet/login', { ip, user, password });
}

/** Both cracking a password and scanning ports are the same queued attack. */
export function hack(ip: number, kind: HackKind): Promise<HackResult> {
  return post<HackResult>('/internet/hack', { ip, kind });
}

export function analyze(ip: number): Promise<QueuedProcess & { analyzer_version: number }> {
  return post('/internet/analyze', { ip });
}

export function listFiles(ip: number): Promise<RemoteFile[]> {
  return get<RemoteFile[]>(`/internet/${ip}/software`);
}

export function transferFile(
  ip: number,
  softwareId: number,
  direction: TransferDirection,
): Promise<QueuedProcess> {
  return post<QueuedProcess>(`/internet/${ip}/software`, {
    software_id: softwareId,
    direction,
  });
}

export function removeFile(ip: number, softwareId: number): Promise<QueuedProcess> {
  return del<QueuedProcess>(`/internet/${ip}/software/${softwareId}`);
}

export function readLog(ip: number): Promise<{ ip: number; text: string }> {
  return get(`/internet/${ip}/log`);
}

/** Rewriting the victim's log is queued work, not an instant edit. */
export function editLog(ip: number, text: string): Promise<QueuedProcess> {
  return put<QueuedProcess>(`/internet/${ip}/log`, { text });
}

export function listHistory(): Promise<Visited[]> {
  return get<Visited[]>('/internet/history');
}

export function listBookmarks(): Promise<Visited[]> {
  return get<Visited[]>('/internet/bookmarks');
}

export function addBookmark(ip: number): Promise<{ ip: number }> {
  return post('/internet/bookmarks', { ip });
}

export function removeBookmark(ip: number): Promise<void> {
  return del<void>(`/internet/bookmarks/${ip}`);
}
