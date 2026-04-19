# ailib-wasm-test

[![License](https://img.shields.io/badge/license-MIT%20%7C%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![WASM](https://img.shields.io/badge/wasm-wasm32--unknown--unknown-yellow.svg)](https://webassembly.org/)

> A minimal web chat application that validates the usability of WASM components from the [ai-lib](https://github.com/ailib-official) ecosystem. All AI protocol logic — request building, response parsing, error classification, stream event handling — runs inside a browser WASM module compiled from `ai-lib-core`.

**Read this in**: [中文](README_CN.md)

---

## Table of Contents

- [Motivation](#motivation)
- [Architecture](#architecture)
- [Key Findings](#key-findings)
- [Project Structure](#project-structure)
- [Prerequisites](#prerequisites)
- [Build & Run](#build--run)
- [Configuration](#configuration)
- [Testing](#testing)
- [WASM API Reference](#wasm-api-reference)
- [HTTP API Reference](#http-api-reference)
- [How It Works](#how-it-works)
- [Governance](#governance)
- [License](#license)

---

## Motivation

The ai-lib ecosystem provides a Rust crate called `ai-lib-core` that implements the AI protocol specification — chat request building, response parsing, error code classification, and more. This project answers a specific question:

> **Can `ai-lib-core` compile to browser WASM and work correctly end-to-end with real AI providers?**

The answer is **yes**, with caveats documented in [Key Findings](#key-findings). This project serves as both a validation test and a reference implementation for anyone wanting to use ai-lib-core in a browser context.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│  Browser                                                      │
│                                                               │
│  ┌──────────────────┐  ┌──────────────────────────────────┐  │
│  │  index.html      │  │  ailib_wasm_bg.wasm              │  │
│  │  (Chat UI)       │──│  (ai-lib-core compiled to WASM)  │  │
│  │                  │  │                                  │  │
│  │  • User input    │  │  • build_chat_request()          │  │
│  │  • SSE display   │  │  • parse_chat_response()         │  │
│  │  • Model select  │  │  • parse_stream_event()          │  │
│  │                  │  │  • classify_error()              │  │
│  │                  │  │  • is_stream_done()              │  │
│  └────────┬─────────┘  └──────────────────────────────────┘  │
│           │                                                    │
│           │  POST /api/proxy  or  POST /api/proxy/stream      │
│           │  (JSON body built by WASM)                        │
└───────────┼────────────────────────────────────────────────────┘
            │
            ▼
┌──────────────────────────────────────────────────────────────┐
│  ailib-wasm-test-server  (Axum, port 3000)                   │
│                                                               │
│  • Resolves API keys from environment variables               │
│  • Forwards requests to AI providers via libcurl              │
│  • Strips Authorization header before sending to browser       │
│  • Returns full response (non-stream) or SSE stream           │
│                                                               │
│  Routes:                                                      │
│    GET  /              → index.html                           │
│    GET  /health        → {"status":"ok","version":"0.1.0"}    │
│    POST /api/proxy     → Non-streaming proxy                  │
│    POST /api/proxy/stream → SSE streaming proxy               │
│    *   /...            → Static file serving                  │
└──────────────────────────────────────────────────────────────┘
```

**Design principle**: The server is a dumb pipe. All AI protocol intelligence lives in the WASM module. The server exists solely to:

1. Bypass browser CORS restrictions
2. Protect API keys (never sent to the browser)
3. Handle TLS fingerprinting issues (see [Key Findings](#key-findings))

---

## Key Findings

During development, several important discoveries were made about the ai-lib ecosystem and AI provider compatibility:

### 1. ai-lib-wasm is WASI, not browser WASM

The existing `ai-lib-wasm` crate targets `wasm32-wasip1` (WASI), which is designed for server-side runtimes. It **cannot** run in browsers. To use `ai-lib-core` in a browser, we created a new crate (`ailib-wasm-browser`) that compiles `ai-lib-core` to `wasm32-unknown-unknown` with `wasm-bindgen`.

### 2. ai-lib-core compiles cleanly to browser WASM

The `drivers` module (including `OpenAiDriver`) and `error_code` module compile without modification to `wasm32-unknown-unknown`. This confirms that `ai-lib-core` is WASM-compatible out of the box.

### 3. `uuid` and `getrandom` require `js` feature

When targeting `wasm32-unknown-unknown`, the `uuid` and `getrandom` crates need their `js` features enabled:

```toml
uuid = { version = "1.6", features = ["js"] }
getrandom = { version = "0.2", features = ["js"] }
```

Without these, WASM compilation fails with a "getrandom: unavailable" panic.

### 4. `wasm-opt = false` required

The `wasm-pack` build fails when `binaryen` (wasm-opt) is not installed on the system. Disabling it in Cargo.toml metadata resolves this:

```toml
[package.metadata.wasm-pack.profile.release]
wasm-opt = false
```

### 5. Groq blocks Rust TLS fingerprints

**This is the most impactful finding.** Groq's Cloudflare proxy returns HTTP 403 for requests made with Rust's `reqwest` crate — both with `rustls` and `native-tls` backends. The blocking is based on TLS fingerprinting (JA3/JA4).

- **`reqwest` (rustls)**: 403 Forbidden
- **`reqwest` (native-tls)**: 403 Forbidden
- **`curl` binary (subprocess)**: 403 Forbidden
- **`curl` crate (libcurl FFI)**: ✅ 200 OK

The `curl` crate's `Easy` handle works because libcurl's TLS fingerprint differs from both Rust's native TLS stacks and the `curl` binary. This is likely due to HTTP/2 negotiation differences or header ordering variations in libcurl's internal implementation.

### 6. BigInt / u16 interop issues

- Rust `i64` maps to JavaScript `BigInt`, not `Number`. Fixed by changing `max_tokens` from `i64` to `f64`.
- Rust `u16` causes wasm-bindgen to return string types instead of numbers. Fixed by changing `code_val` from `u16` to `u32`.

### 7. HTTP_PROXY environment variable breaks Groq

The `curl` binary subprocess inherits the parent process's `HTTP_PROXY` environment variable, which causes Groq requests to fail. The `curl` crate does **not** auto-inherit proxy settings, so it works without intervention. When using the `curl` binary, add `--noproxy "*"`.

---

## Project Structure

```
ailib-wasm-test/
├── Cargo.toml                          # Workspace definition
├── LICENSE                             # Apache-2.0
├── LICENSE-MIT                         # MIT
├── LICENSE-APACHE                      # Apache-2.0
├── README.md                           # This file (English)
├── README_CN.md                        # Chinese version
├── crates/
│   ├── wasm-browser/                   # Browser WASM crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # wasm_bindgen wrappers
│   │       └── core_logic.rs           # Pure Rust logic + 19 unit tests
│   └── server/                         # Backend proxy server
│       ├── Cargo.toml
│       └── src/
│           └── main.rs                 # Axum server + curl proxy + 8 tests
├── static/
│   ├── index.html                      # Chat UI (single-file SPA)
│   └── wasm/                           # wasm-pack output
│       ├── ailib_wasm_bg.wasm
│       ├── ailib_wasm.js
│       └── ailib_wasm.d.ts
└── tests/
    ├── e2e.spec.js                     # Playwright end-to-end tests (7 tests)
    ├── playwright.config.js
    └── package.json
```

---

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Build server and WASM crate |
| wasm-pack | latest | Build WASM for browser target |
| Node.js | 18+ | Playwright e2e tests |
| curl (libcurl) | any | Server uses `curl` crate with `static-curl` feature |

### Install wasm-pack

```bash
cargo install wasm-pack
```

### Install Playwright

```bash
cd tests && npm install && npx playwright install
```

---

## Build & Run

### 1. Build the WASM module

```bash
wasm-pack build crates/wasm-browser --target web --out-dir ../../static/wasm --out-name ailib_wasm
```

This compiles `ai-lib-core` to `wasm32-unknown-unknown` and outputs the WASM binary + JS glue code to `static/wasm/`.

### 2. Build the server

```bash
cargo build --release
```

### 3. Set environment variables

```bash
export GROQ_API_KEY="gsk_..."
export DEEPSEEK_API_KEY="sk-..."
export NVIDIA_API_KEY="nvapi-..."
# Optional: OpenAI
export OPENAI_API_KEY="sk-..."
```

### 4. Start the server

```bash
./target/release/ailib-wasm-test-server
```

Or as a background process:

```bash
nohup ./target/release/ailib-wasm-test-server > /tmp/ailib-wasm-test.log 2>&1 & disown
```

### 5. Open in browser

Navigate to [http://localhost:3000](http://localhost:3000). You should see:

- "WASM ready" in the header (confirms WASM module loaded)
- A model selector with NVIDIA, Groq, and DeepSeek providers
- A chat input with streaming support

---

## Configuration

### Supported Providers

| Provider | Model ID | Environment Variable | Status |
|----------|----------|---------------------|--------|
| NVIDIA | `nvidia/z-ai/glm-5.1` | `NVIDIA_API_KEY` | ✅ Works |
| NVIDIA | `nvidia/z-ai/glm4.7` | `NVIDIA_API_KEY` | ✅ Works |
| Groq | `groq/llama-3.1-8b-instant` | `GROQ_API_KEY` | ✅ Works (curl crate only) |
| DeepSeek | `deepseek/deepseek-chat` | `DEEPSEEK_API_KEY` | ✅ Works |
| OpenAI | any | `OPENAI_API_KEY` | ✅ Works |

### Server Configuration

The server listens on `0.0.0.0:3000` by default. There are no command-line flags; configuration is via environment variables only.

---

## Testing

### Unit Tests (27 tests)

```bash
cargo test --release
```

This runs:
- **19 WASM logic tests** (`core_logic.rs`) — pure Rust, no WASM runtime needed
- **8 server tests** (`main.rs`) — includes health endpoint, route handling, and live Groq/DeepSeek integration tests

### End-to-End Tests (7 tests)

Requires the server to be running:

```bash
# Start server first
./target/release/ailib-wasm-test-server &

# Run e2e tests
cd tests && env -i HOME=$HOME PATH=$PATH npx playwright test --reporter=list
```

Test coverage:
1. Health endpoint returns OK
2. `index.html` loads with correct title
3. WASM module initializes successfully
4. DeepSeek streaming chat works end-to-end
5. Groq streaming chat works end-to-end
6. New chat clears history
7. All WASM functions callable from JavaScript

---

## WASM API Reference

The browser WASM module exposes 5 functions via `wasm-bindgen`:

### `build_chat_request(messages_json, model, temperature, max_tokens, stream) → BuildResult`

Builds an OpenAI-compatible chat completion request body using `ai-lib-core`'s `OpenAiDriver`.

| Parameter | Type | Description |
|-----------|------|-------------|
| `messages_json` | `string` | JSON array of `{"role":"user\|assistant\|system","content":"..."}` |
| `model` | `string` | Model identifier (e.g., `"deepseek-chat"`) |
| `temperature` | `f64` | Sampling temperature (0.0–2.0) |
| `max_tokens` | `f64` | Maximum tokens to generate (f64 to avoid JS BigInt) |
| `stream` | `bool` | Enable SSE streaming |

**Returns**: `BuildResult` with `.body()` (JSON string) and `.stream()` (boolean).

### `parse_chat_response(response_json) → ParseResult`

Parses a non-streaming chat completion response JSON.

**Returns**: `ParseResult` with `.content()`, `.finish_reason()`, `.prompt_tokens()`, `.completion_tokens()`, `.total_tokens()`.

### `parse_stream_event(data) → StreamEventResult`

Parses a single SSE stream event data payload.

**Returns**: `StreamEventResult` with:
- `.event_type()` — `"content_delta"`, `"thinking_delta"`, `"role_assign"`, `"stream_end"`, or `"unknown"`
- `.data()` — The extracted content string
- `.done()` — Whether the stream is complete

### `classify_error(status_code) → ErrorClassResult`

Classifies an HTTP error status code using `ai-lib-core`'s `StandardErrorCode`.

**Returns**: `ErrorClassResult` with `.code()`, `.name()`, `.category()`, `.retryable()`.

### `is_stream_done(data) → bool`

Checks if an SSE data payload signals stream completion (`[DONE]`).

---

## HTTP API Reference

### `GET /health`

Returns server health status.

```json
{"status": "ok", "version": "0.1.0"}
```

### `POST /api/proxy`

Non-streaming proxy. Forwards the request body to the specified AI provider URL.

**Request body**:
```json
{
  "url": "https://api.deepseek.com/chat/completions",
  "headers": {},
  "body": { "model": "deepseek-chat", "messages": [...], "stream": false },
  "stream": false
}
```

**Response**:
```json
{
  "status": 200,
  "body": { "choices": [...], "usage": {...} }
}
```

### `POST /api/proxy/stream`

SSE streaming proxy. Forwards the request and returns the response as Server-Sent Events.

**Request body**: Same as `/api/proxy` with `"stream": true`.

**Response**: SSE stream with `data:` events. Each event contains one line of the upstream provider's SSE response.

---

## How It Works

### Request Flow (Streaming)

1. User types a message and clicks "Send"
2. JavaScript calls `wasm.build_chat_request(messages, model, 0.7, 4096, true)` — WASM builds the protocol-correct request body
3. JavaScript sends `POST /api/proxy/stream` with the WASM-built body
4. Server's `proxy_stream_handler` receives the request
5. Server resolves the API key from environment variables based on the URL
6. A `spawn_blocking` thread runs the `curl` crate's `Easy` handle
7. The `write_function` callback splits incoming data by newlines and sends each line through an `mpsc` channel
8. The async side reads from `ReceiverStream`, strips the `data: ` prefix, and wraps each line in an axum SSE `Event`
9. Browser JavaScript reads the SSE stream, calls `wasm.parse_stream_event(data)` for each event
10. WASM returns the event type (`content_delta`, `stream_end`, etc.) and data
11. JavaScript appends content to the chat display

### Request Flow (Non-streaming)

Same steps 1–4, but:
- Endpoint is `POST /api/proxy`
- Server waits for the full response via `curl_proxy()`
- JavaScript calls `wasm.parse_chat_response(body)` to extract content, finish reason, and usage

### Error Handling

When the proxy returns a non-OK status, JavaScript calls `wasm.classify_error(statusCode)` which returns:
- Error code and name (e.g., `429 RATE_LIMITED`)
- Category (`"rate"`, `"client"`, `"server"`)
- Whether the error is retryable

---

## Governance

This project follows the ai-lib ecosystem governance rules:

| Rule | Description |
|------|-------------|
| **ARCH-001** | Protocol-driven design — all AI logic comes from ai-lib-core |
| **ARCH-004** | Default branch is `main` |
| **GOV-001 v2** | Public code repos must be on `ailib-official`, not personal accounts |
| **GOV-002** | Merge conflict resolution requires code-level review, not blind whole-file takeover |
| **DOC-001** | English code comments + Chinese module headers |

---

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you shall be dual-licensed as above, without any additional terms or conditions.
