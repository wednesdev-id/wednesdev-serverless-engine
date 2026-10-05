# Contributing

## Setup and checks

Use current stable Rust with rustfmt and clippy. Wasmtime dependency is 25; no minimum Rust version is currently certified. From repository root:

```bash
rustup target add wasm32-wasip1
cargo install wasm-tools --version 1.261.0 --locked
bash scripts/build-fixtures.sh
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release
```

Fixture build must precede tests: integration tests read `target/hello.wasm`, `target/infinite_loop.wasm`, and `target/oom.wasm`. Raw Rust WASM output is not a component. Build script pins and checks v25.0.0 WASI reactor adapter. CI uses same fixture script before testing.

## Changes

- Write behavior test first, observe failure, then make minimal change.
- Format with `cargo fmt --all`; submit one logical change per PR.
- Commit convention: `<type>: <description>` (feat, fix, test, docs, ci, chore).
- Include commands, actual test output, and known limits; do not infer full acceptance from smoke tests.
- Runtime changes need latency/RSS impact notes and bounded adversarial tests.
- For loop, memory, and trap tests, use client deadlines and verify successful request on same daemon PID after failure. HTTP 500 alone is not proof of isolation.
- Persistent deploy cache remains missing. Do not describe in-memory reuse as cross-process caching.

Use [issue templates](.github/ISSUE_TEMPLATE) for bugs and proposals. Follow [security reporting](docs/SECURITY.md) for suspected vulnerabilities. Contributions use Apache-2.0.
