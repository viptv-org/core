import { createHttpTransport, type HttpOptions } from './index.ts';

export type NativeFetch = (url: string, init: RequestInit & { maxRedirections: number }) => Promise<Response>;

/** Tauri's plugin requires its own redirect limit; Fetch's redirect option is insufficient. */
export function createNativeHttpTransport(options: Omit<HttpOptions, 'fetch'>, fetch: NativeFetch) {
  return createHttpTransport({
    ...options,
    fetch: (url, init) => fetch(url, { ...init, maxRedirections: 0 }),
  });
}
