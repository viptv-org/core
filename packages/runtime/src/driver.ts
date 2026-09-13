import type { Event, Request, StorageOperation, StorageResult, ViewModel } from '../../../generated/typescript/wire.js';
import type { HttpRequest, HttpResult } from './index.ts';

type Awaitable<T> = T | Promise<T>;
export interface CorePort {
  update(event: string): Awaitable<string>;
  resolve(id: number, result: string): Awaitable<string>;
  view(): Awaitable<string>;
}
export interface StoragePort {
  load(): Awaitable<string | null>;
  save(tokens: string): Awaitable<void>;
  clear(): Awaitable<void>;
}
export interface DriverOptions {
  core: CorePort;
  storage: StoragePort;
  http(request: HttpRequest, signal: AbortSignal): Promise<HttpResult>;
  render(view: ViewModel): void;
  onError(message: string): void;
  maxPendingEffects?: number;
}

/** One effect dispatcher for browser WASM and Tauri's native core commands. */
export function createCoreDriver(options: DriverOptions) {
  const maxPending = options.maxPendingEffects ?? 64;
  if (!Number.isSafeInteger(maxPending) || maxPending < 1) throw new Error('Invalid pending effect limit');
  let disposed = false;
  let queue: Promise<void> = Promise.resolve();
  let storageQueue: Promise<unknown> = Promise.resolve();
  const pending = new Set<Promise<void>>();
  const controllers = new Set<AbortController>();

  function enqueue(work: () => Promise<void>): Promise<void> {
    const task = queue.then(async () => { if (!disposed) await work(); });
    queue = task.catch(() => { if (!disposed) options.onError('Could not process core effects'); });
    return task;
  }

  async function storage(operation: StorageOperation): Promise<StorageResult> {
    try {
      if (operation === 'Load') return { Ok: await options.storage.load() };
      if (operation === 'Clear') await options.storage.clear();
      else await options.storage.save(operation.Save);
      return { Ok: null };
    } catch { return { Err: 'Secure storage operation failed' }; }
  }

  function track(id: number, output: Promise<HttpResult | StorageResult>) {
    const task = output.then(result => enqueue(async () => {
      await drain(await options.core.resolve(id, JSON.stringify(result)));
    })).catch(() => { if (!disposed) options.onError('Could not resolve core effect'); });
    pending.add(task);
    void task.finally(() => pending.delete(task));
  }

  async function drain(serialized: string) {
    const requests = JSON.parse(serialized) as Request[];
    if (!Array.isArray(requests)) throw new Error('Invalid effect batch');
    for (const { id, effect } of requests) {
      if (disposed) return;
      if ('Render' in effect) {
        options.render(JSON.parse(await options.core.view()) as ViewModel);
      } else if (pending.size >= maxPending) {
        // A broken/hostile producer must not grow unbounded transport/storage queues.
        throw new Error('Too many pending effects');
      } else if ('Http' in effect) {
        const controller = new AbortController();
        controllers.add(controller);
        const output = Promise.resolve().then(() => options.http(effect.Http, controller.signal))
          .catch((): HttpResult => ({ Err: { Io: 'HTTP transport failed' } }))
          .finally(() => controllers.delete(controller));
        track(id, output);
      } else if ('Storage' in effect) {
        // Preserve Save/Clear order even when platform storage is asynchronous.
        const output = storageQueue.then(() => disposed
          ? { Err: 'Shell disposed' } as StorageResult
          : storage(effect.Storage));
        storageQueue = output;
        track(id, output);
      } else throw new Error('Unknown core effect');
    }
  }

  return {
    dispatch(event: Event): Promise<void> {
      if (disposed) return Promise.reject(new Error('Core driver is disposed'));
      return enqueue(async () => { await drain(await options.core.update(JSON.stringify(event))); });
    },
    /** Cancel old network work when the host changes account/server scope. */
    cancelHttp() { for (const controller of controllers) controller.abort(); },
    async idle(): Promise<void> {
      do { await queue; await Promise.all([...pending]); } while (pending.size);
      await queue;
    },
    dispose() {
      disposed = true;
      for (const controller of controllers) controller.abort();
    },
  };
}

/** Pass Tauri's invoke function; no Tauri dependency enters the browser bundle. */
export function tauriCorePort(invoke: <T>(command: string, args?: Record<string, unknown>) => Promise<T>): CorePort {
  return {
    update: event => invoke<string>('core_update', { event }),
    resolve: (id, result) => invoke<string>('core_resolve', { id, result }),
    view: () => invoke<string>('core_view'),
  };
}
