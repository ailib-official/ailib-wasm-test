# syntax=docker/dockerfile:1
# Multi-stage build: WASM (wasm-pack) + server (release), minimal runtime.
# Build from repo root: docker compose build
# Or: docker build -f Dockerfile -t ailib-wasm-test ..

FROM rust:1.83-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install wasm-pack --locked

WORKDIR /build

# ai-lib-core path dependency (not in this repo)
ARG AI_LIB_RUST_REPO=https://github.com/ailib-official/ai-lib-rust.git
ARG AI_LIB_RUST_REF=main
RUN git clone --depth 1 --branch "${AI_LIB_RUST_REF}" "${AI_LIB_RUST_REPO}" ai-lib-rust

COPY . ailib-wasm-test

WORKDIR /build/ailib-wasm-test

RUN wasm-pack build crates/wasm-browser \
    --target web \
    --out-dir ../../static/wasm \
    --out-name ailib_wasm

RUN cargo build -p ailib-wasm-test-server --release

# --- runtime ---
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /build/ailib-wasm-test/target/release/ailib-wasm-test-server /app/ailib-wasm-test-server
COPY --from=builder /build/ailib-wasm-test/static /app/static

ENV AILIB_WASM_STATIC_DIR=/app/static

EXPOSE 3000

ENTRYPOINT ["/app/ailib-wasm-test-server"]
CMD ["--port", "3000"]
