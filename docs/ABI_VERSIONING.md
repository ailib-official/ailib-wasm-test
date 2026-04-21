# Browser + WASI WASM ABI versioning

This repository ships two WebAssembly surfaces that share protocol logic (ARCH-003):

1. **wasm-bindgen (`ailib-wasm-browser`)** — `static/wasm/`, used by the chat demo.
2. **C ABI (`ai-lib-wasm` in [ai-lib-rust](https://github.com/ailib-official/ai-lib-rust))** — WASI / wasmtime hosts.

## Browser: unified `ailib_invoke`

Legacy exports (`build_chat_request`, `parse_chat_response`, …) remain stable. New code should prefer a single JSON payload:

```js
const wasm = await import('./wasm/ailib_wasm.js');
await wasm.default('./wasm/ailib_wasm_bg.wasm');

const out = wasm.ailib_invoke(JSON.stringify({
  op: 'build_request',
  version: 1, // optional; default 1
  messages: [{ role: 'user', content: 'hi' }],
  model: 'gpt-4',
  temperature: 0.7,
  max_tokens: 1024,
  stream: false,
}));
const parsed = JSON.parse(out);
```

Supported `op` values are listed in the `capabilities` operation:

```js
JSON.parse(wasm.ailib_invoke(JSON.stringify({ op: 'capabilities' })));
```

Host `version` in the object must be `<=` the browser ABI (see `BROWSER_ABI_VERSION` in Rust). Unknown fields are ignored by design for forward compatibility.

## WASI: `ailib_invoke` + negotiation

`ai-lib-wasm` exports:

- `ailib_abi_version()` — current ABI (2).
- `ailib_capabilities_ptr` / `ailib_capabilities_len` — static JSON.
- `ailib_invoke(op, input, ctx)` — structured dispatch; v1 positional exports remain.

Snapshot format and ABI version are independent; see `WASM_STATE_MIGRATION.md`.

## Compatibility matrix

| Host \\ module | v1-only | v1+v2 |
|----------------|---------|-------|
| v1 host        | OK      | OK (use v1 exports) |
| v2 host        | OK      | OK (prefer invoke) |

Breaking changes require a major ABI bump and a migration note in this folder.
