# WASM state migration (hot upgrade)

Stateful data in `ai-lib-wasm` includes:

- Loaded provider manifests (`ailib_load_manifest`).
- Aggregated metrics (`WasmMetrics`).
- Scratch output buffers (`LAST_OUT` / `LAST_ERR`) — not part of snapshots.

## Snapshot schema

`WasmStateSnapshot` (JSON) is defined in `ai-lib-rust/crates/ai-lib-wasm/src/state.rs`:

- `version` — snapshot format (`SNAPSHOT_FORMAT_VERSION`).
- `abi_version` — ABI that wrote the snapshot (`AILIB_ABI_VERSION`).
- `manifests` — provider id + raw YAML (re-validated on restore).
- `active_streams` — reserved; SSE replay is host-driven (`needs_replay`).
- `metrics` — call counters and token totals.

## Exports

| Export | Role |
|--------|------|
| `ailib_snapshot_state()` | Writes snapshot JSON to `ailib_out_*`. |
| `ailib_restore_state(ptr, len)` | Atomic replace: parse failure leaves prior state. |

The `ailib_invoke` ops `snapshot_state` and `restore_state` wrap the same logic.

## Host workflow (wasmtime)

1. Call `ailib_snapshot_state` on the old instance; consume output with `ailib_out_consume` + `ailib_free`.
2. Instantiate the new module.
3. Call `ailib_restore_state` on the new instance.
4. Route new traffic to the new instance; drain the old instance.

Live HTTP streams cannot resume inside WASM after unload; the snapshot marks streams for host-side replay when that data is populated.

## Versioning rules

- Newer WASM **may** restore older snapshots (field mapping).
- Older WASM **must** reject unknown snapshot format or ABI versions with an error and leave state unchanged.
