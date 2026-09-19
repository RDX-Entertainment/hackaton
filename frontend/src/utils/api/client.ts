/**
 * The one place that knows how to talk to the backend: base path, bearer token,
 * JSON encoding and what an error response means. Endpoint modules build on it
 * and never touch `fetch` themselves.
 */
import { readToken } from '../session';

const API_BASE = '/api';

const NO_CONTENT = 204;

type RequestOptions = {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE';
  /** Sent as a JSON body. Left out entirely when undefined. */
  body?: unknown;
};

function buildHeaders(hasBody: boolean): HeadersInit {
  const headers: Record<string, string> = {};

  if (hasBody) {
    headers['content-type'] = 'application/json';
  }

  const token = readToken();
  if (token) {
    headers.authorization = `Bearer ${token}`;
  }

  return headers;
}

/**
 * A failed request keeps its status code. Some screens answer differently to a 404
 * than to any other failure, and a bare Error throws that away.
 */
export class HttpError extends Error {
  readonly status: number;

  /**
   * El codigo estable del fallo: `NO_CERTIFICATION`, `DISK_FULL`, `NOT_FOUND`. Es lo que se
   * traduce. El `message` que lo acompaña es la frase que mando el servidor, y sirve de
   * respaldo mientras un codigo no tenga entrada en ningun catalogo.
   */
  readonly code: string | null;

  constructor(message: string, status: number, code: string | null = null) {
    super(message);
    this.name = 'HttpError';
    this.status = status;
    this.code = code;
  }
}

async function errorFrom(response: Response): Promise<HttpError> {
  const body = (await response.json().catch(() => ({}))) as { error?: string; code?: string };
  return new HttpError(
    body.error || `Request failed (${response.status})`,
    response.status,
    body.code ?? null,
  );
}

export async function request<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const { method = 'GET', body } = options;
  const hasBody = body !== undefined;

  const response = await fetch(`${API_BASE}${path}`, {
    method,
    headers: buildHeaders(hasBody),
    body: hasBody ? JSON.stringify(body) : undefined,
  });

  if (!response.ok) {
    throw await errorFrom(response);
  }

  if (method !== 'GET') window.dispatchEvent(new Event('game:changed'));

  // DELETE and some POSTs answer with no body at all.
  if (response.status === NO_CONTENT) {
    return undefined as T;
  }

  return (await response.json()) as T;
}

export const get = <T>(path: string): Promise<T> => request<T>(path);

export const post = <T>(path: string, body?: unknown): Promise<T> =>
  request<T>(path, { method: 'POST', body });

export const put = <T>(path: string, body?: unknown): Promise<T> =>
  request<T>(path, { method: 'PUT', body });

export const del = <T>(path: string): Promise<T> => request<T>(path, { method: 'DELETE' });
