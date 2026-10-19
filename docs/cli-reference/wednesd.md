# Daemon Overview (`wednesd`)

`wednesd` is the core daemon and CLI tool for **Wednes Serverless Engine**.

It orchestrates the WebAssembly runtime, validates manifests, serves HTTP and QUIC gateways, enforces per-function isolation, and executes functions adhering to the `wednes:function@0.1.0` component ABI.

---

## Installation & Verification

Build from source or download release binaries:

```bash
# Build optimized release binary with cargo
cargo build --release --bin wednesd

# Verify binary version and build info
./target/release/wednesd --help
```

Output:
```text
Wednes Engine Function Daemon

Usage: wednesd [OPTIONS] [COMMAND]

Commands:
  init    Initialize a new serverless function project
  dev     Start local development server with file watching and live reload
  deploy  Deploy a function manifest to an atomic file registry
  run     Start the production daemon serving functions from a registry
  help    Print this message or the help of the given subcommand(s)

Options:
  -l, --listen <LISTEN>                      [default: 127.0.0.1:8080]
  -w, --wasm <WASM>                          
  -i, --id <ID>                              [default: hello]
      --global-concurrency <GLOBAL_CONCURRENCY>  [default: 100]
      --memory-budget-mb <MEMORY_BUDGET_MB>      [default: 1200]
      --default-concurrency <DEFAULT_CONCURRENCY>  [default: 16]
      --log-json                             
      --http3                                
  -h, --help                                 Print help
  -V, --version                              Print version
```

---

## Two Operating Modes

`wednesd` operates in two primary modes:

### 1. Registry Mode (Production Recommended)
In registry mode, `wednesd` reads a multi-function registry file (`registry.json`) containing precompiled function metadata, resource limits, and routing paths. This mode supports zero-downtime atomic updates, multi-function concurrency pools, and strict memory budgets.

```bash
# Run production daemon serving all functions from registry.json
wednesd run --registry ./registry.json --listen 0.0.0.0:8080
```

### 2. Standalone Mode (Development / Legacy)
In standalone mode, `wednesd` loads a single `.wasm` component directly into memory on boot. This is ideal for rapid local testing or minimal single-purpose micro-daemons.

```bash
wednesd --listen 127.0.0.1:8080 --wasm ./hello.wasm --id hello
```

---

## Logging & Observability

By default, `wednesd` outputs human-readable console logs using `tracing-subscriber`.

For cloud-native production environments (Datadog, Grafana Loki, FluentBit), enable structured JSON output with `--log-json`:

```bash
wednesd run --registry ./registry.json --log-json
```

Sample JSON log stream:
```json
{"timestamp":"2026-10-19T10:15:30.123Z","level":"INFO","fields":{"message":"Loaded function 'auth-validator'"},"target":"wednesd"}
{"timestamp":"2026-10-19T10:15:30.125Z","level":"INFO","fields":{"message":"wednesd HTTP gateway listening on 127.0.0.1:8080"},"target":"wednesd"}
```
