/* tslint:disable */
/* eslint-disable */

/**
 * Result of `build_chat_request`: contains the serialized request body and stream flag.
 */
export class BuildResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * The JSON request body as a string.
     */
    body(): string;
    /**
     * Whether streaming is enabled.
     */
    stream(): boolean;
}

/**
 * Result of `classify_error`: contains error code, name, category, retryable.
 */
export class ErrorClassResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    category(): string;
    code(): number;
    name(): string;
    retryable(): boolean;
}

/**
 * Result of `parse_chat_response`: contains content, finish reason, and normalized usage.
 *
 * Exposes ARCH-003 extended token counters (`reasoning_tokens`,
 * `cache_read_tokens`, `cache_creation_tokens`) that parity with
 * ai-lib-ts/ai-lib-go `UnifiedResponse.usage`.
 */
export class ParseResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    cache_creation_tokens(): bigint;
    cache_read_tokens(): bigint;
    completion_tokens(): bigint;
    content(): string;
    finish_reason(): string;
    prompt_tokens(): bigint;
    reasoning_tokens(): bigint;
    total_tokens(): bigint;
}

/**
 * Result of `parse_stream_event`: contains event type, data, and done flag.
 */
export class StreamEventResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    data(): string;
    done(): boolean;
    event_type(): string;
}

/**
 * Unified versioned entry point (WASM-001). Pass a JSON object with `"op"` and fields
 * for that operation; returns a JSON string on success.
 *
 * Operations: `abi_version`, `capabilities`, `build_request`, `parse_response`,
 * `parse_stream_event`, `classify_error`, `is_stream_done`. Optional `"version"`
 * field defaults to `1` and must be `<= BROWSER_ABI_VERSION`.
 */
export function ailib_invoke(input_json: string): string;

/**
 * Build a chat completion request body using ai-lib-core protocol logic.
 */
export function build_chat_request(messages_json: string, model: string, temperature: number, max_tokens: number, stream: boolean): BuildResult;

/**
 * Classify an HTTP error status code.
 */
export function classify_error(status_code: number): ErrorClassResult;

/**
 * Check if an SSE data payload signals stream completion (`[DONE]`).
 */
export function is_stream_done(data: string): boolean;

/**
 * Parse a non-streaming chat completion response.
 */
export function parse_chat_response(response_json: string): ParseResult;

/**
 * Parse a single SSE stream event data payload.
 */
export function parse_stream_event(data: string): StreamEventResult;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_buildresult_free: (a: number, b: number) => void;
    readonly __wbg_errorclassresult_free: (a: number, b: number) => void;
    readonly __wbg_parseresult_free: (a: number, b: number) => void;
    readonly __wbg_streameventresult_free: (a: number, b: number) => void;
    readonly ailib_invoke: (a: number, b: number) => [number, number, number, number];
    readonly build_chat_request: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly buildresult_body: (a: number) => [number, number];
    readonly buildresult_stream: (a: number) => number;
    readonly classify_error: (a: number) => number;
    readonly errorclassresult_category: (a: number) => [number, number];
    readonly errorclassresult_code: (a: number) => number;
    readonly errorclassresult_name: (a: number) => [number, number];
    readonly errorclassresult_retryable: (a: number) => number;
    readonly is_stream_done: (a: number, b: number) => number;
    readonly parse_chat_response: (a: number, b: number) => [number, number, number];
    readonly parse_stream_event: (a: number, b: number) => [number, number, number];
    readonly parseresult_cache_creation_tokens: (a: number) => bigint;
    readonly parseresult_cache_read_tokens: (a: number) => bigint;
    readonly parseresult_completion_tokens: (a: number) => bigint;
    readonly parseresult_content: (a: number) => [number, number];
    readonly parseresult_finish_reason: (a: number) => [number, number];
    readonly parseresult_prompt_tokens: (a: number) => bigint;
    readonly parseresult_reasoning_tokens: (a: number) => bigint;
    readonly parseresult_total_tokens: (a: number) => bigint;
    readonly streameventresult_data: (a: number) => [number, number];
    readonly streameventresult_done: (a: number) => number;
    readonly streameventresult_event_type: (a: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
