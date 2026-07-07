# Changelog

## [0.2.0] — 2026-07-08

### Added

- **WASM-004** — `justfile` with `build-wasm`, `build-server`, `build-all`, `dev`, `clean`; Windows build notes in README ([#1](https://github.com/ailib-official/ailib-wasm-test/pull/1))
- **WASM-005** — Portable static path resolution (`--static-dir`, `AILIB_WASM_STATIC_DIR`, `CARGO_MANIFEST_DIR` fallback) ([#1](https://github.com/ailib-official/ailib-wasm-test/pull/1))
- **WASM-006** — Multi-stage `Dockerfile`, `docker-compose.yml`, `.dockerignore`, `.env.docker.example`
- **WASM-007** — README "Which WASM?" comparison (`wasm-browser` vs `ai-lib-wasm`)

### Changed

- Server `AppState` holds resolved `static_dir`; CLI accepts `--port` and `--static-dir`

## [0.1.x] — prior

- Initial browser chat demo with `wasm-browser` + Axum proxy server
