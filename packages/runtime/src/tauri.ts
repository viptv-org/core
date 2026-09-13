import { fetch as nativeFetch } from '@tauri-apps/plugin-http';
import type { HttpOptions } from './index.ts';
import { createNativeHttpTransport } from './native-fetch.ts';

/** HTTP effects from the native Rust core execute through Tauri's Rust HTTP plugin. */
export function createTauriHttpTransport(options: Omit<HttpOptions, 'fetch'>) {
  return createNativeHttpTransport(options, nativeFetch);
}
