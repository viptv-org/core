/* tslint:disable */
/* eslint-disable */
/**
* The display name from a SmartCast deviceinfo response, or `null` when the
* answering host is not a Vizio television. Direct-probe discovery uses
* this after connecting to each candidate from `vizio_discovery_candidates`.
* @param {string} body
* @returns {string | undefined}
*/
export function vizio_deviceinfo_name(body: string): string | undefined;
/**
* @param {string} kind
* @param {string} input
* @param {string} origin
* @returns {string}
*/
export function normalize(kind: string, input: string, origin: string): string;
/**
*/
export class CoreBridge {
  free(): void;
/**
* @param {string} event
* @returns {string}
*/
  update(event: string): string;
/**
* @param {number} id
* @param {string} result
* @returns {string}
*/
  resolve(id: number, result: string): string;
/**
*/
  constructor();
/**
* @returns {string}
*/
  view(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_corebridge_free: (a: number) => void;
  readonly corebridge_resolve: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly corebridge_update: (a: number, b: number, c: number, d: number) => void;
  readonly corebridge_view: (a: number, b: number) => void;
  readonly corebridge_wasm_new: () => number;
  readonly normalize: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly vizio_deviceinfo_name: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
