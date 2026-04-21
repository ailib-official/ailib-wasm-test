# WASM memory model (browser + WASI)

## Problem

WASM linear memory is a single growable arena. Without an explicit release contract, host reads of `ailib_out_ptr` / length pin buffers that never return to the allocator.

## WASI (`ai-lib-wasm`)

1. **Borrow** — After a successful call, read output via `ailib_out_ptr()` and `ailib_out_len()` while the next call has not overwritten `LAST_OUT`.

2. **Transfer** — Prefer `ailib_out_consume(out_len)` to take ownership of the output buffer. The pointer is valid until the host calls `ailib_free(ptr, len)` with the same length returned from consume.

3. **Bulk reset** — `ailib_arena_reset()` drops scratch buffers for `LAST_OUT` and `LAST_ERR` without a full instance reload.

`ailib_free` expects buffers produced by `ailib_out_consume` (boxed slices). Passing arbitrary pointers is undefined behavior.

## Browser (wasm-bindgen)

Exported structs (`BuildResult`, `ParseResult`, …) have wasm-bindgen-generated `.free()`. After copying strings/values to JavaScript, call `.free()` to release WASM-side allocations. The demo and Playwright tests illustrate this pattern:

```js
const br = wasm.build_chat_request(msgJson, model, 0.7, 1024, true);
const body = br.body();
br.free();
```

The unified `ailib_invoke` API returns a plain JSON **string**, avoiding an extra exported struct for the happy path.

## Future work

A slab allocator / arena inside `ai-lib-wasm` may be added for high-throughput gateways; the external ptr+len+free contract stays stable.
