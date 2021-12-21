/* tslint:disable */
/* eslint-disable */

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: (a: number, b: number) => number;
  readonly macroquad_audio_crate_version: () => number;
  readonly quad_url_crate_version: () => number;
  readonly sapp_jsutils_crate_version: () => number;
  readonly file_loaded: (a: number) => void;
  readonly crate_version: () => number;
  readonly allocate_vec_u8: (a: number) => number;
  readonly on_clipboard_paste: (a: number, b: number) => void;
  readonly frame: () => void;
  readonly mouse_move: (a: number, b: number) => void;
  readonly raw_mouse_move: (a: number, b: number) => void;
  readonly mouse_down: (a: number, b: number, c: number) => void;
  readonly mouse_up: (a: number, b: number, c: number) => void;
  readonly mouse_wheel: (a: number, b: number) => void;
  readonly key_down: (a: number, b: number, c: number) => void;
  readonly key_press: (a: number) => void;
  readonly key_up: (a: number) => void;
  readonly resize: (a: number, b: number) => void;
  readonly touch: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_start: () => void;
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
