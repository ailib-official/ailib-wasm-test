# ailib-wasm-test

[![License](https://img.shields.io/badge/license-MIT%20%7C%20Apache--2.0-blue.svg)](LICENSE) [![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/) [![WASM](https://img.shields.io/badge/wasm-wasm32--unknown--unknown-yellow.svg)](https://webassembly.org/)

> A minimal web chat app that proves **ai-lib-core runs in the browser**. All AI protocol logic — request building, response parsing, error classification, stream handling — executes inside a WASM module compiled from `ai-lib-core`.

**Read this in**: [中文](README_CN.md)

---

## What Is This?

A proof-of-concept chat application demonstrating that the [ai-lib](https://github.com/ailib-official) protocol stack works end-to-end in a browser via WebAssembly.

The browser loads a WASM module compiled from `ai-lib-core` (Rust). When you send a message, the WASM module — not the server — builds the AI protocol request, parses the response, classifies errors, and handles streaming events. The server is just a thin proxy for CORS and key safety.

### One-line summary

> **Protocol intelligence in the browser, compiled from Rust.**

---

## Quick Demo

```bash
# Build WASM
wasm-pack build crates/wasm-browser --target web --out-dir ../../static/wasm --out-name ailib_wasm

# Build server
cargo build --release

# Set your keys
export DEEPSEEK_API_KEY="sk-..."
export NVIDIA_API_KEY="nvapi-..."

# Run
./target/release/ailib-wasm-test-server
```

Open **http://localhost:3000** — chat with AI models, powered entirely by WASM-based protocol logic.

---

## How It Works

```
┌──────────────────────────────────────────────┐
│  Browser                                      │
│                                               │
│  ┌─────────┐     ┌────────────────────────┐  │
│  │ Chat UI │────▶│ ailib_wasm (WASM)      │  │
│  │         │     │                        │  │
│  │ Display │◀────│ • Build requests       │  │
│  │ Streams │     │ • Parse responses      │  │
│  │         │     │ • Classify errors      │  │
│  └─────────┘     └────────────────────────┘  │
│        │                      │               │
└────────┼──────────────────────┼───────────────┘
         │   WASM-built body    │
         ▼                      │
┌──────────────────────────────────────────────┐
│  Thin Proxy (Axum)                           │
│                                              │
│  • Hides API keys from the browser           │
│  • Bypasses CORS                             │
│  • Forwards requests to AI providers         │
└──────────────────────────────────────────────┘
```

**Design principle**: The server is a dumb pipe. All AI protocol intelligence lives in the WASM module.

---

## Supported Providers

| Provider | Models | Status |
|----------|--------|--------|
| **DeepSeek** | deepseek-chat | ✅ |
| **NVIDIA** | glm-5.1, glm4.7 | ✅ |
| **Groq** | llama-3.1-8b-instant | ✅ |
| **OpenAI** | any | ✅ |

Set the corresponding environment variable (`DEEPSEEK_API_KEY`, `NVIDIA_API_KEY`, `GROQ_API_KEY`, `OPENAI_API_KEY`) and select the model in the UI.

---

## WASM API

The WASM module exposes 5 functions via `wasm-bindgen`:

| Function | Purpose |
|----------|---------|
| `build_chat_request()` | Build an OpenAI-compatible request body |
| `parse_chat_response()` | Parse a non-streaming response |
| `parse_stream_event()` | Parse a single SSE stream event |
| `classify_error()` | Classify an HTTP error by standard error codes |
| `is_stream_done()` | Check if a stream event signals completion |

All functions are implemented in `ai-lib-core` and compiled to WASM — zero JavaScript protocol logic.

---

## Build & Run

### Prerequisites

- **Rust** 1.75+ — [rustup.rs](https://rustup.rs)
- **wasm-pack** — `cargo install wasm-pack`

### Steps

```bash
# 1. Build WASM module
wasm-pack build crates/wasm-browser --target web --out-dir ../../static/wasm --out-name ailib_wasm

# 2. Build server
cargo build --release

# 3. Set API keys
export DEEPSEEK_API_KEY="sk-..."
export NVIDIA_API_KEY="nvapi-..."

# 4. Start
./target/release/ailib-wasm-test-server
```

### Test

```bash
# Unit tests
cargo test --release

# E2E tests (requires running server + Playwright)
cd tests && npm install && npx playwright install
npx playwright test
```

---

## Project Structure

```
ailib-wasm-test/
├── crates/
│   ├── wasm-browser/    # WASM crate (wasm-bindgen + ai-lib-core)
│   └── server/          # Thin Axum proxy
├── static/
│   ├── index.html       # Single-file chat UI
│   └── wasm/            # Compiled WASM output
└── tests/
    └── e2e.spec.js      # Playwright E2E tests
```

---

## Why This Matters

This project validates a key property of the ai-lib ecosystem:

> **The same Rust code that runs on the server can run in the browser.**

No protocol reimplementation in JavaScript. No drift between server-side and client-side AI logic. One protocol stack, multiple runtimes.

This is foundational for the ai-lib vision: an open AI protocol where any runtime — server, edge, browser, embedded — can speak the same language.

---

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.
