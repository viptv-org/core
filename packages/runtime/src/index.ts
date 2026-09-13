import type { HttpRequest, HttpResult } from '../../../generated/typescript/wire.js';
export type { HttpHeader, HttpRequest, HttpResponse, HttpResult } from '../../../generated/typescript/wire.js';

export type FetchTransport = (input: string, init: RequestInit) => Promise<Response>;
export interface HttpOptions {
  /** Exact origins; never wildcard hosts. HTTP is only enabled by an explicit entry. */
  allowedOrigins: readonly string[];
  fetch?: FetchTransport;
  timeoutMs?: number;
  maxResponseBytes?: number;
  credentials?: RequestCredentials;
}

export function createHttpTransport(options: HttpOptions) {
  const allowed = new Set(options.allowedOrigins.map(value => {
    const url = new URL(value);
    if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password ||
        url.pathname !== '/' || url.search || url.hash) throw new Error('Expected an HTTP origin');
    return url.origin;
  }));
  const fetchImpl = options.fetch ?? globalThis.fetch.bind(globalThis);
  const timeoutMs = options.timeoutMs ?? 30_000;
  const maxBytes = options.maxResponseBytes ?? 8 * 1024 * 1024;
  if (!Number.isSafeInteger(maxBytes) || maxBytes < 1 || !Number.isFinite(timeoutMs) || timeoutMs <= 0)
    throw new Error('Invalid HTTP transport limits');

  return async (request: HttpRequest, signal?: AbortSignal): Promise<HttpResult> => {
    let url: URL;
    try { url = new URL(request.url); } catch { return { Err: { Url: 'Invalid request URL' } }; }
    if (!allowed.has(url.origin) || url.username || url.password || url.hash)
      return { Err: { Url: 'Request URL is outside the allowed origins' } };
    const controller = new AbortController();
    let timedOut = false;
    const abort = () => controller.abort();
    signal?.addEventListener('abort', abort, { once: true });
    if (signal?.aborted) abort();
    const timeout = setTimeout(() => { timedOut = true; abort(); }, timeoutMs);
    try {
      if (controller.signal.aborted) return { Err: { Io: 'Request cancelled' } };
      const headers = new Headers();
      for (const header of request.headers) headers.append(header.name, header.value);
      if (!request.body.every(byte => Number.isInteger(byte) && byte >= 0 && byte <= 255))
        return { Err: { Io: 'Invalid request bytes' } };
      const response = await fetchImpl(url.href, {
        method: request.method, headers,
        body: request.body.length ? new Uint8Array(request.body) : undefined,
        signal: controller.signal,
        redirect: 'error',
        credentials: options.credentials ?? 'omit',
      });
      // Browser fetch rejects redirects; native adapters must disable them too.
      if (response.redirected || response.status === 0 ||
          (response.status >= 300 && response.status < 400 && response.headers.has('location'))) {
        await response.body?.cancel();
        return { Err: { Url: 'Redirects are not permitted' } };
      }
      const chunks: Uint8Array[] = [];
      let length = 0;
      const reader = response.body?.getReader();
      if (reader) {
        try {
          while (true) {
            const chunk = await reader.read();
            if (chunk.done) break;
            length += chunk.value.byteLength;
            if (length > maxBytes) {
              await reader.cancel();
              return { Err: { Io: 'HTTP response exceeds the size limit' } };
            }
            chunks.push(chunk.value);
          }
        } finally { reader.releaseLock(); }
      }
      if (controller.signal.aborted)
        return { Err: timedOut ? 'Timeout' : { Io: 'Request cancelled' } };
      const bytes = new Uint8Array(length);
      let offset = 0;
      for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.length; }
      return { Ok: {
        status: response.status,
        headers: Array.from(response.headers, ([name, value]) => ({ name, value })),
        body: Array.from(bytes),
      } };
    } catch {
      // Never surface native exceptions containing credential-bearing URLs/headers.
      return { Err: timedOut ? 'Timeout' : { Io: controller.signal.aborted ? 'Request cancelled' : 'HTTP transport failed' } };
    } finally {
      clearTimeout(timeout);
      signal?.removeEventListener('abort', abort);
    }
  };
}
