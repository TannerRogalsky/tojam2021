/* tslint:disable */
/* eslint-disable */
/**
*/
export enum KeyEvent {
  W,
  A,
  S,
  D,
  Space,
}
/**
*/
export class Wrapper {
  free(): void;
/**
* @param {HTMLCanvasElement} canvas
* @param {Uint8Array} debug_font_data
*/
  constructor(canvas: HTMLCanvasElement, debug_font_data: Uint8Array);
/**
* @param {number} _t_ms
*/
  step(_t_ms: number): void;
/**
* @param {number} key_code
*/
  handle_key_down(key_code: number): void;
/**
* @param {number} key_code
*/
  handle_key_up(key_code: number): void;
/**
* @param {boolean} is_left_button
*/
  handle_mouse_down(is_left_button: boolean): void;
/**
* @param {boolean} is_left_button
*/
  handle_mouse_up(is_left_button: boolean): void;
/**
* @param {number} x
* @param {number} y
*/
  handle_mouse_move(x: number, y: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wrapper_free: (a: number) => void;
  readonly wrapper_new: (a: number, b: number, c: number) => number;
  readonly wrapper_step: (a: number, b: number) => void;
  readonly wrapper_handle_key_down: (a: number, b: number) => void;
  readonly wrapper_handle_key_up: (a: number, b: number) => void;
  readonly wrapper_handle_mouse_down: (a: number, b: number) => void;
  readonly wrapper_handle_mouse_up: (a: number, b: number) => void;
  readonly wrapper_handle_mouse_move: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
