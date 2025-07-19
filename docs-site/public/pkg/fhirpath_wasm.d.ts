/* tslint:disable */
/* eslint-disable */
/**
 * Initialize panic hook for better error messages in the browser
 */
export function main(): void;
/**
 * Evaluate a FHIRPath expression against a FHIR resource
 *
 * # Arguments
 * * `expression` - The FHIRPath expression to evaluate
 * * `resource_json` - The FHIR resource as a JSON string
 *
 * # Returns
 * A JSON string containing the evaluation result, or an error message
 */
export function evaluate_fhirpath(expression: string, resource_json: string): string;
/**
 * Validate a FHIRPath expression syntax
 *
 * # Arguments
 * * `expression` - The FHIRPath expression to validate
 *
 * # Returns
 * A JSON string indicating whether the expression is valid
 */
export function validate_fhirpath(expression: string): string;
/**
 * Get the FHIRPath specification version
 */
export function get_fhirpath_version(): string;
/**
 * Get the AST (Abstract Syntax Tree) of a FHIRPath expression
 *
 * # Arguments
 * * `expression` - The FHIRPath expression to parse
 *
 * # Returns
 * A JSON string containing the AST representation, or an error message
 */
export function get_expression_ast(expression: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: () => void;
  readonly evaluate_fhirpath: (a: number, b: number, c: number, d: number) => [number, number];
  readonly validate_fhirpath: (a: number, b: number) => [number, number];
  readonly get_fhirpath_version: () => [number, number];
  readonly get_expression_ast: (a: number, b: number) => [number, number];
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_3: WebAssembly.Table;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
