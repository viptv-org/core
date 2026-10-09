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
/**
* Non-serializable one-request/generation authority. Private getters are engine-adapter-only.
*/
export class NativeTorrentBridge {
  free(): void;
/**
* @returns {string}
*/
  state(): string;
/**
* @param {number} status
* @param {string} body
* @param {string} observation
* @returns {string}
*/
  accept(status: number, body: string, observation: string): string;
/**
* @returns {string}
*/
  toString(): string;
/**
* @param {string} clock
* @returns {string}
*/
  authorize(clock: string): string;
/**
* @param {string} clock
* @returns {string}
*/
  privateInfoHash(clock: string): string;
/**
* @param {string} clock
* @returns {number}
*/
  privateFileIndex(clock: string): number;
/**
* @param {string} clock
* @returns {string}
*/
  privateInputKind(clock: string): string;
/**
*/
  invalidate(): void;
/**
* @param {string} clock
* @returns {string}
*/
  privateInputValue(clock: string): string;
/**
* @returns {string | undefined}
*/
  playbackId(): string | undefined;
/**
* @param {number} status
* @param {Uint8Array} body
* @param {string} observation
* @returns {string}
*/
  acceptBytes(status: number, body: Uint8Array, observation: string): string;
/**
* @param {string} clock
* @returns {bigint | undefined}
*/
  privateExpectedFileSize(clock: string): bigint | undefined;
/**
* @param {string} facts
* @param {string} clock
* @returns {boolean}
*/
  metadataMatches(facts: string, clock: string): boolean;
/**
* @returns {bigint | undefined}
*/
  trustedWallUpperUnixMillis(): bigint | undefined;
/**
* @param {number} status
* @param {Uint8Array} body
* @param {string} observation
* @returns {string}
*/
  acceptMeasuredBytes(status: number, body: Uint8Array, observation: string): string;
/**
* @param {string} info_hash
* @param {number} file_index
* @param {number} file_count
* @param {bigint} selected_file_size
* @param {boolean} validated_v1_metadata
* @param {string} clock
* @returns {boolean}
*/
  metadataMatchesNative(info_hash: string, file_index: number, file_count: number, selected_file_size: bigint, validated_v1_metadata: boolean, clock: string): boolean;
/**
* @param {string} context
*/
  constructor(context: string);
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_nativetorrentbridge_free: (a: number) => void;
  readonly nativetorrentbridge_accept: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly nativetorrentbridge_acceptBytes: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly nativetorrentbridge_acceptMeasuredBytes: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly nativetorrentbridge_authorize: (a: number, b: number, c: number, d: number) => void;
  readonly nativetorrentbridge_invalidate: (a: number, b: number) => void;
  readonly nativetorrentbridge_metadataMatches: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly nativetorrentbridge_metadataMatchesNative: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
  readonly nativetorrentbridge_playbackId: (a: number, b: number) => void;
  readonly nativetorrentbridge_privateExpectedFileSize: (a: number, b: number, c: number, d: number) => void;
  readonly nativetorrentbridge_privateFileIndex: (a: number, b: number, c: number, d: number) => void;
  readonly nativetorrentbridge_privateInfoHash: (a: number, b: number, c: number, d: number) => void;
  readonly nativetorrentbridge_privateInputKind: (a: number, b: number, c: number, d: number) => void;
  readonly nativetorrentbridge_privateInputValue: (a: number, b: number, c: number, d: number) => void;
  readonly nativetorrentbridge_state: (a: number, b: number) => void;
  readonly nativetorrentbridge_toString: (a: number, b: number) => void;
  readonly nativetorrentbridge_trustedWallUpperUnixMillis: (a: number, b: number) => void;
  readonly nativetorrentbridge_wasm_new: (a: number, b: number, c: number) => void;
  readonly __wbg_corebridge_free: (a: number) => void;
  readonly corebridge_resolve: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly corebridge_update: (a: number, b: number, c: number, d: number) => void;
  readonly corebridge_view: (a: number, b: number) => void;
  readonly corebridge_wasm_new: () => number;
  readonly normalize: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly vizio_deviceinfo_name: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
