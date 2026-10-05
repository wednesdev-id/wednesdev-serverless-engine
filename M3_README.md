# Wednes Engine M3 Implementation

## Completed Features
- **Manifest parsing & strict validation:** `memory_mb` (1-2048), `timeout_ms` (1-60000), `max_concurrency` (strictly 1). Prevents arbitrary paths/unsupported ABIs.
- **Precompile cache strategy:** Validates WASM against the WIT `wednes:function@0.1.0` ABI via `FunctionPre::new(instantiate_pre(...))` at `deploy` time. Rejects empty/invalid modules without invoking the steady-state Cranelift compiler. Uses Wasmtime's managed caching internally and avoids native deserialization of untrusted payloads on startup.
- **Atomic Registry:** Stores metadata (`registry.json`) with tempfile-based atomic swap and rollback on validation failure.
- **Resource isolation:** Propagates per-function memory and timeout configuration directly to `StoreLimits` and epoch-based `trap`.
- **CLI Commands:** Added `deploy` and `run` to natively manage registry while keeping the `wednesd --listen --wasm --id` fallback intact.
- **SDK fix:** Wrapped `bindgen!` macro in `pub mod bindings` to fix `__export_world_function_cabi` collision.

## M3 vs M4 Bound
Currently, `max_concurrency` is strictly blocked if not exactly `1`. Multi-concurrency admission and capability flags (Filesystem/Outbound HTTP/Env) remain unsupported, explicitly bailing validation during `deploy` as per V0/M3 specification until Milestone 4 expands isolation guarantees.
