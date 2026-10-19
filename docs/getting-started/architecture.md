# Architecture

This document details the internal design and architectural components of **Wednes Serverless Engine**.

---

## High-Level Request Pipeline

Every incoming HTTP or QUIC request flows through an admission and sandboxed execution pipeline:

```
[ HTTP/1.1 / HTTP/2 Client ]      [ HTTP/3 QUIC Client ]
             │                               │
             ▼ (TCP)                         ▼ (UDP)
     +───────────────────────────────────────────────+
     |                 axum Router                   |
     |         (Route: /fn/:id and /fn/:id/*path)    |
     +───────────────────────────────────────────────+
                             │
                             ▼
     +───────────────────────────────────────────────+
     |                   Scheduler                   |
     |  - Global Concurrency Semaphore               |
     |  - Memory Budget Admission                    |
     |  - Per-Function Concurrency Semaphore         |
     +───────────────────────────────────────────────+
                             │ (Permit Acquired)
                             ▼
     +───────────────────────────────────────────────+
     |             WasmtimeBackend Engine            |
     |  - Compiled Component Cache (in-memory)       |
     |  - Fresh wasmtime::Store creation             |
     |  - StoreLimits (memory_mb bound)              |
     |  - Epoch Interruption (timeout_ms bound)      |
     |  - WASI P1/P2 Context Sandbox                 |
     +───────────────────────────────────────────────+
                             │
                             ▼
     +───────────────────────────────────────────────+
     |            Guest Component Execution          |
     |           ABI: wednes:function@0.1.0          |
     |             handle(request) -> response       |
     +───────────────────────────────────────────────+
```

---

## Crate Modularization

Wednes Serverless Engine is architected as an idiomatic Rust workspace composed of focused crates:

| Crate | Responsibilities |
|:---|:---|
| **`wednes-cli`** | The `wednesd` daemon entrypoint, argument parsing via `clap`, subcommands (`init`, `dev`, `deploy`, `run`), and legacy single-component loader. |
| **`wednes-gateway`** | Network ingress, `axum` routing for `/fn/:id` and `/fn/:id/*path`, HTTP/3 QUIC server setup via `quinn` and `rustls`, and scheduling semaphores. |
| **`wednes-core`** | Domain types (`Request`, `Response`, `Header`), strict manifest validation (`wednes.yaml` / `manifest.toml`), and atomic file registry (`registry.json`). |
| **`wednes-runtime`** | Abstract `RuntimeBackend` trait defining execution interfaces, lifecycle hooks, and common errors. |
| **`wednes-runtime-wasmtime`** | Concrete Wasmtime implementation: JIT compilation, pre-instance caching (`instantiate_pre`), typed export checking, epoch background timer, and WASI builder. |
| **`wednes-capabilities`** | Security boundaries and capability verification (filesystem access, network policies, outbound HTTP admission). |
| **`wednes-telemetry`** | Structured JSON logging (`--log-json`), tracing spans, and runtime latency tracking. |

---

## Sandboxed Execution Lifecycle

### 1. Ingress & Admission Control
When an HTTP request arrives at `axum`:
1. The function identifier `:id` is extracted from the URL.
2. The `Scheduler` checks whether the function is registered. If missing, it returns `404 Not Found`.
3. The scheduler requests two semaphore permits:
   - **Global Concurrency Permit:** Bounded by `--global-concurrency` (default: 100).
   - **Per-Function Concurrency Permit:** Configured per-function in manifest (default: 16).
4. If either semaphore is saturated, the gateway immediately rejects the request with `429 Too Many Requests`. This prevents cascading memory exhaustion or unbounded event queues.

### 2. Precompiled Component Cache
Compilation of WebAssembly bytecode to native machine code via Cranelift is computationally intensive. To achieve sub-millisecond execution:
- At deploy/load time, the component is parsed, validated against the WIT contract `wednes:function@0.1.0`, and pre-instantiated.
- The compiled component bytecode resides in memory as an immutable template across the lifetime of the daemon process.

### 3. Per-Invocation Store Isolation
For each request:
- A new, clean `wasmtime::Store` is created.
- A fresh linear memory instance is mapped.
- State is strictly isolated: no shared global memory, static mutation, or dirty state can leak between requests or across different callers.
- After the request finishes, the `Store` is dropped and linear memory is immediately reclaimed by the OS.

### 4. Epoch Trapping & Memory Limits
To prevent runaway scripts:
- **Memory Enforcement:** `StoreLimits` bounds linear memory growth according to the function's `memory_mb` limit (1 MB to 2048 MB).
- **CPU Timeouts:** A background epoch incrementer ticks continuously. If a function exceeds its `timeout_ms` budget (e.g., in an infinite loop), Wasmtime triggers an epoch trap, aborting execution and returning an internal execution error without crashing `wednesd`.

---

## Atomic Registry Architecture

Wednes uses an atomic, transactional local registry model (`registry.json`):

1. **Deployment Phase:**
   - The user runs `wednesd deploy --manifest manifest.toml --registry registry.json`.
   - The manifest is parsed and checked against strict validation constraints.
   - The `.wasm` binary is checked for exports conforming to `wednes:function@0.1.0`.
2. **Atomic Swap:**
   - The new registry is written to a temporary sibling file (e.g., `registry.json.tmp.12345`).
   - The temporary file is atomically renamed (`std::fs::rename`) over `registry.json`.
   - On POSIX filesystems, this rename is guaranteed to be atomic. Readers never observe corrupted or half-written registry states.
3. **Rollback Guarantee:**
   - If validation or precompilation fails at any step, the temporary file is purged and the existing registry remains untouched.

---

## Security Boundaries & Operator Model

- **Untrusted Guests:** Guest WASM code is strictly untrusted.
- **Trusted Operator:** Host OS, daemon binary, file registry, and build pipeline belong to the trusted boundary.
- **Closed-by-Default WASI:** Guests have no access to host directories, no environment variables, and cannot spawn host child processes.
- **Reverse Proxy Recommended:** When exposing `wednesd` to the public internet, place it behind an authenticated reverse proxy (such as Traefik, Nginx, or Cloudflare Tunnel) to handle rate limiting and public TLS termination.
