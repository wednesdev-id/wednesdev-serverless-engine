# Wednes Engine

Rust + WebAssembly function runtime. `wednesd` serves typed WIT functions using Wasmtime 25, without Docker, Kubernetes, or a database. Apache-2.0; alpha, not a production security boundary guarantee.

## Quickstart

Requires current stable Rust, native Rust build prerequisites, Bash, curl, and sha256sum. Run from repository root:

```bash
rustup target add wasm32-wasip1
cargo install wasm-tools --version 1.261.0 --locked
cargo build --release
bash scripts/build-fixtures.sh
./target/release/wednesd --listen 127.0.0.1:8080 --wasm target/hello.wasm --id hello
```

Daemon stays in foreground. In another terminal:

```bash
curl --fail-with-body --max-time 5 -X POST http://127.0.0.1:8080/fn/hello -d '{}'
```

Expected HTTP 200 with JSON `message` equal to `hello from wednes engine`. Stop daemon with Ctrl-C.

**A raw `wasm32-wasip1` module cannot be loaded directly.** Build script downloads and verifies Wasmtime **v25.0.0 reactor adapter**, builds all examples, then wraps each module with `wasm-tools component new`. For hello, conversion is:

```bash
wasm-tools component new target/wasm32-wasip1/release/hello_world.wasm --adapt wasi_snapshot_preview1=target/wasi_snapshot_preview1.reactor.wasm -o target/hello.wasm
wasm-tools validate target/hello.wasm
```

Routes are `/fn/<id>` and `/fn/<id>/<path>`, not `/<id>`.

## Examples

Build script produces these runnable components:

| Source | Component | Behavior |
| --- | --- | --- |
| `examples/hello-world` | `target/hello.wasm` | JSON greeting |
| `examples/json-api` | `target/json_api.wasm` | JSON echo; malformed JSON returns 400 |
| `examples/webhook` | `target/webhook.wasm` | Acknowledges method and payload byte count; no signature verification |
| `examples/infinite-loop` | `target/infinite_loop.wasm` | Timeout test fixture |
| `examples/oom` | `target/oom.wasm` | Memory-limit test fixture |

To try JSON API or webhook, restart daemon with corresponding component and ID, then POST JSON to `/fn/<id>`. Webhook example is not a secure webhook receiver.

## Registry and architecture

[Architecture and registry walkthrough](docs/ARCHITECTURE.md) describes deploy/run commands, ABI, admission, and storage. `wednes init` and `wednes dev` are planned, not implemented commands.

Compiled components are reused **in memory during a daemon process**. Deployment validates and compiles but does **not** persist compiled code for a later `run` process. All V0 acceptance criteria are not yet satisfied.

## Benchmarks and contributing

See [benchmark report](benchmarks/REPORT.md) for measured results and reproduction. Targets are not guarantees. Build fixtures before tests: see [CONTRIBUTING.md](CONTRIBUTING.md). Read [security boundaries and reporting](docs/SECURITY.md) before exposing any listener beyond localhost.
