# Reproducible build targets (WASM-004)
# Install just: https://github.com/casey/just

wasm_crate := "crates/wasm-browser"
wasm_out := "static/wasm"

# Build browser WASM via wasm-pack (output: static/wasm/)
build-wasm:
    wasm-pack build {{wasm_crate}} --target web --out-dir ../../{{wasm_out}} --out-name ailib_wasm

# Build release server binary (output: target/release/ailib-wasm-test-server)
build-server:
    cargo build -p ailib-wasm-test-server --release

build-all: build-wasm build-server

# Build everything, then run the server on the default port (3000)
dev: build-all
    cargo run -p ailib-wasm-test-server

clean:
    cargo clean
