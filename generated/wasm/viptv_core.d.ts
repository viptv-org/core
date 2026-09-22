/* tslint:disable */
/* eslint-disable */
/**
* @param {string} base
* @param {string} parts
* @returns {string}
*/
export function addonEndpoint(base: string, parts: string): string;
/**
* @param {string} catalog
* @returns {string}
*/
export function addonCatalogExtras(catalog: string): string;
/**
* @param {string} manifest
* @param {string} resource
* @param {string} kind
* @param {string} id
* @returns {boolean}
*/
export function addonSupports(manifest: string, resource: string, kind: string, id: string): boolean;
/**
* @param {string} entries
* @param {string} request
* @returns {string}
*/
export function discoverPlan(entries: string, request: string): string;
/**
* @param {string} responses
* @param {string} plan
* @param {bigint} skip
* @returns {string}
*/
export function discoverAggregate(responses: string, plan: string, skip: bigint): string;
/**
* @param {bigint} provider_id
* @param {string} kind
* @param {string} row
* @returns {string}
*/
export function providerCandidate(provider_id: bigint, kind: string, row: string): string;
/**
* @param {string} kind
* @param {string} request
* @param {string} candidates
* @returns {string}
*/
export function providerSelectCandidates(kind: string, request: string, candidates: string): string;
/**
* @param {string} provider
* @param {string} kind
* @param {string} id
* @param {string} ext
* @returns {string}
*/
export function providerMediaUrl(provider: string, kind: string, id: string, ext: string): string;
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
  readonly addonCatalogExtras: (a: number, b: number, c: number) => void;
  readonly addonEndpoint: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly addonSupports: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
  readonly discoverAggregate: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly discoverPlan: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly providerCandidate: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly providerMediaUrl: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
  readonly providerSelectCandidates: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly __wbg_corebridge_free: (a: number) => void;
  readonly corebridge_resolve: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly corebridge_update: (a: number, b: number, c: number, d: number) => void;
  readonly corebridge_view: (a: number, b: number) => void;
  readonly corebridge_wasm_new: () => number;
  readonly normalize: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
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
