/* @ts-self-types="./ailib_wasm_browser.d.ts" */

/**
 * Result of `build_chat_request`: contains the serialized request body and stream flag.
 */
export class BuildResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(BuildResult.prototype);
        obj.__wbg_ptr = ptr;
        BuildResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BuildResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_buildresult_free(ptr, 0);
    }
    /**
     * The JSON request body as a string.
     * @returns {string}
     */
    body() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.buildresult_body(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Whether streaming is enabled.
     * @returns {boolean}
     */
    stream() {
        const ret = wasm.buildresult_stream(this.__wbg_ptr);
        return ret !== 0;
    }
}
if (Symbol.dispose) BuildResult.prototype[Symbol.dispose] = BuildResult.prototype.free;

/**
 * Result of `classify_error`: contains error code, name, category, retryable.
 */
export class ErrorClassResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ErrorClassResult.prototype);
        obj.__wbg_ptr = ptr;
        ErrorClassResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ErrorClassResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_errorclassresult_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    category() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.errorclassresult_category(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    code() {
        const ret = wasm.errorclassresult_code(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {string}
     */
    name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.errorclassresult_name(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {boolean}
     */
    retryable() {
        const ret = wasm.errorclassresult_retryable(this.__wbg_ptr);
        return ret !== 0;
    }
}
if (Symbol.dispose) ErrorClassResult.prototype[Symbol.dispose] = ErrorClassResult.prototype.free;

/**
 * Result of `parse_chat_response`: contains content, finish reason, and normalized usage.
 *
 * Exposes ARCH-003 extended token counters (`reasoning_tokens`,
 * `cache_read_tokens`, `cache_creation_tokens`) that parity with
 * ai-lib-ts/ai-lib-go `UnifiedResponse.usage`.
 */
export class ParseResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ParseResult.prototype);
        obj.__wbg_ptr = ptr;
        ParseResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ParseResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_parseresult_free(ptr, 0);
    }
    /**
     * @returns {bigint}
     */
    cache_creation_tokens() {
        const ret = wasm.parseresult_cache_creation_tokens(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {bigint}
     */
    cache_read_tokens() {
        const ret = wasm.parseresult_cache_read_tokens(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {bigint}
     */
    completion_tokens() {
        const ret = wasm.parseresult_completion_tokens(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    content() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.parseresult_content(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    finish_reason() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.parseresult_finish_reason(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {bigint}
     */
    prompt_tokens() {
        const ret = wasm.parseresult_prompt_tokens(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {bigint}
     */
    reasoning_tokens() {
        const ret = wasm.parseresult_reasoning_tokens(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {bigint}
     */
    total_tokens() {
        const ret = wasm.parseresult_total_tokens(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) ParseResult.prototype[Symbol.dispose] = ParseResult.prototype.free;

/**
 * Result of `parse_stream_event`: contains event type, data, and done flag.
 */
export class StreamEventResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(StreamEventResult.prototype);
        obj.__wbg_ptr = ptr;
        StreamEventResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StreamEventResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_streameventresult_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    data() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.streameventresult_data(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {boolean}
     */
    done() {
        const ret = wasm.streameventresult_done(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {string}
     */
    event_type() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.streameventresult_event_type(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) StreamEventResult.prototype[Symbol.dispose] = StreamEventResult.prototype.free;

/**
 * Unified versioned entry point (WASM-001). Pass a JSON object with `"op"` and fields
 * for that operation; returns a JSON string on success.
 *
 * Operations: `abi_version`, `capabilities`, `build_request`, `parse_response`,
 * `parse_stream_event`, `classify_error`, `is_stream_done`. Optional `"version"`
 * field defaults to `1` and must be `<= BROWSER_ABI_VERSION`.
 * @param {string} input_json
 * @returns {string}
 */
export function ailib_invoke(input_json) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(input_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ailib_invoke(ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Build a chat completion request body using ai-lib-core protocol logic.
 * @param {string} messages_json
 * @param {string} model
 * @param {number} temperature
 * @param {number} max_tokens
 * @param {boolean} stream
 * @returns {BuildResult}
 */
export function build_chat_request(messages_json, model, temperature, max_tokens, stream) {
    const ptr0 = passStringToWasm0(messages_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(model, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.build_chat_request(ptr0, len0, ptr1, len1, temperature, max_tokens, stream);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return BuildResult.__wrap(ret[0]);
}

/**
 * Classify an HTTP error status code.
 * @param {number} status_code
 * @returns {ErrorClassResult}
 */
export function classify_error(status_code) {
    const ret = wasm.classify_error(status_code);
    return ErrorClassResult.__wrap(ret);
}

/**
 * Check if an SSE data payload signals stream completion (`[DONE]`).
 * @param {string} data
 * @returns {boolean}
 */
export function is_stream_done(data) {
    const ptr0 = passStringToWasm0(data, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.is_stream_done(ptr0, len0);
    return ret !== 0;
}

/**
 * Parse a non-streaming chat completion response.
 * @param {string} response_json
 * @returns {ParseResult}
 */
export function parse_chat_response(response_json) {
    const ptr0 = passStringToWasm0(response_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.parse_chat_response(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ParseResult.__wrap(ret[0]);
}

/**
 * Parse a single SSE stream event data payload.
 * @param {string} data
 * @returns {StreamEventResult}
 */
export function parse_stream_event(data) {
    const ptr0 = passStringToWasm0(data, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.parse_stream_event(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return StreamEventResult.__wrap(ret[0]);
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_6b64449b9b9ed33c: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./ailib_wasm_bg.js": import0,
    };
}

const BuildResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_buildresult_free(ptr >>> 0, 1));
const ErrorClassResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_errorclassresult_free(ptr >>> 0, 1));
const ParseResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_parseresult_free(ptr >>> 0, 1));
const StreamEventResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_streameventresult_free(ptr >>> 0, 1));

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('ailib_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
