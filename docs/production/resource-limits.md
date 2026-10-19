# Resource Limits & Isolation

**Wednes Serverless Engine** implements a **3-tier defense-in-depth isolation architecture** to prevent untrusted guest code or saturated endpoints from degrading the host system or starving adjacent functions.

---

## The 3-Tier Enforcement Model

```
 Tier 1: Linux Host & cgroups v2
 ┌─────────────────────────────────────────────────────────────┐
 │  - systemd / Docker MemoryMax (e.g. 1800MB)                 │
 │  - CPUQuota (e.g. 200%)                                     │
 │  - LimitNOFILE (e.g. 65536)                                 │
 └──────────────────────────────┬──────────────────────────────┘
                                │
 Tier 2: Gateway Admission & Schedulers
 ┌──────────────────────────────▼──────────────────────────────┐
 │  - Global Concurrency Semaphore (--global-concurrency 200)  │
 │  - Per-Function Concurrency (--default-concurrency 16)      │
 │  - Host Memory Budget Check (--memory-budget-mb 1536)       │
 └──────────────────────────────┬──────────────────────────────┘
                                │
 Tier 3: Wasmtime Sandboxed Runtime
 ┌──────────────────────────────▼──────────────────────────────┐
 │  - StoreLimits: per-invocation linear memory ceiling (32MB) │
 │  - Epoch Timer: deterministic execution deadline (250ms)    │
 │  - Closed WASI World: zero host filesystem / socket leakage │
 └─────────────────────────────────────────────────────────────┘
```

---

## 1. Gateway Admission (Tier 2)

Before any WebAssembly instance is allocated, incoming requests pass through the gateway `Scheduler`:

- **Global Semaphore:** Limits total in-flight requests across all functions. When saturated, incoming requests immediately receive `HTTP 429 Too Many Requests`.
- **Per-Function Semaphore:** Configured via `runtime.max_concurrency` in `wednes.yaml`. Prevents an aggressive webhook caller from consuming all available execution threads.
- **Memory Admission:** Verifies that allocating the function's maximum requested memory will not breach the daemon's `--memory-budget-mb`.

---

## 2. Runtime Sandboxing (Tier 3)

### Linear Memory Bounding (`StoreLimits`)
Each invocation operates in its own newly allocated `wasmtime::Store`. Linear memory allocation is governed by Wasmtime's `StoreLimits`:

- If guest code attempts to allocate past its configured `memory_mb`, the memory growth operation returns an allocation failure or triggers a trap.
- The trap is caught by the daemon; the invocation returns an HTTP error response to the client.
- **The host daemon process does not crash or leak memory.**

### Execution Deadline Trapping (Epoch Interruption)
To neutralize infinite loops, regex catastrophic backtracking, or heavy compute deadlocks:

- A background thread advances Wasmtime's epoch engine at fixed intervals (e.g., every 10 milliseconds).
- Each invocation is configured with an epoch deadline computed from `timeout_ms`.
- When the deadline expires, the Cranelift JIT engine traps out of execution immediately.
- The trapped guest instance is dropped, reclaiming resources.

---

## Failure Behavior Matrix

| Scenario | Trigger / Mechanism | HTTP Status | Impact on Daemon |
|:---|:---|:---:|:---|
| **Memory Exhaustion** | Guest allocates > `memory_mb` | `500 Internal Error` | Trapped cleanly. Zero host leakage. |
| **Infinite Loop** | Execution runs > `timeout_ms` | `500 Internal Error` | Trapped via Epoch timer. Normal operation resumes. |
| **High Traffic Spike** | Concurrent requests > semaphore | `429 Too Many Requests` | Fast rejection without invoking Wasmtime. |
| **Invalid WASM Bytecode** | Missing WIT export or malformed ABI | Rejected at `deploy` | Registry rollback; unchanged live state. |

---

## Sizing Guidelines by Node Capacity

| Server Specs | Suggested `--global-concurrency` | Suggested `--memory-budget-mb` | Per-Function Default `memory_mb` |
|:---|:---:|:---:|:---:|
| **1 vCPU / 1 GB RAM** | `50` | `650` | `16 - 32 MB` |
| **2 vCPU / 2 GB RAM** | `150` | `1400` | `32 - 64 MB` |
| **4 vCPU / 8 GB RAM** | `600` | `6000` | `64 - 128 MB` |
| **8 vCPU / 16 GB RAM** | `1500` | `12000` | `128 - 256 MB` |
