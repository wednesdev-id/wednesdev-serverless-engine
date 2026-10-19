# Benchmarks & Performance

Comprehensive performance benchmarks conducted on standard cloud hardware running **Wednes Serverless Engine**.

---

## Test Environment

- **Host OS:** Linux 6.8.0-139-generic x86_64 (glibc 2.39)
- **CPU:** 2 vCPU
- **RAM:** 2,014,860 KiB (~2 GB)
- **Workload:** 10 active serverless functions loaded concurrently
- **Configuration:** 32 MiB guest memory ceiling per invocation, 250 ms epoch timeout

---

## Latency Percentiles (p50, p95, p99)

The following metrics reflect end-to-end loopback HTTP request-response timings across various synthetic and real-world workloads:

| Test Case | Description | p50 (ms) | p95 (ms) | p99 (ms) |
|:---|:---|:---:|:---:|:---:|
| **`hello`** | Minimal text/plain greeting | **0.845** | 0.989 | 1.070 |
| **`json_api`** | JSON deserialization, transformation & serialization | **0.837** | 1.007 | 1.129 |
| **`webhook`** | Webhook header validation & signature check | **0.818** | 0.913 | 0.929 |
| **`cpu`** | Compute intensive loop (fibonacci / hashing) | **0.814** | 0.893 | 0.994 |
| **`memory`** | Dynamic array allocations within sandbox bounds | **0.826** | 0.982 | 1.091 |
| **`large-response`** | Streaming multi-kilobyte JSON payload | **1.075** | 1.341 | 1.437 |
| **`warm-start`** | Execution with precompiled component in cache | **0.845** | 0.989 | 1.070 |
| **`cold-start`** | First invocation across fresh daemon instances | **1.472** | 5.665 | 34.221 |
| **`concurrency`** | 16 barrier-synchronized concurrent clients | **8.117** | 12.066 | 12.066 |

---

## Sandbox Error Trapping & Safety Latency

Testing resilience when untrusted guest functions misbehave:

| Test Case | Behavior | Latency (ms) | Daemon Status |
|:---|:---|:---:|:---|
| **`infinite_loop`** | Guest enters `while (true) {}` loop | 276.04 ms | Trapped cleanly via epoch timer; host surviving |
| **`oom`** | Guest attempts allocating beyond `memory_mb` | 23.45 ms | Trapped cleanly; zero host memory leaked |
| **`trap/trap`** | Guest triggers explicit `unreachable` opcode | 1.12 ms | Trapped cleanly; HTTP 500 returned to caller |

::: tip Same-PID Recovery
In all failure cases (`infinite_loop`, `oom`, `trap`), the daemon PID survived without restart, and subsequent valid requests immediately succeeded in < 1 ms.
:::

---

## Resource Consumption

| Metric | Measured Value | Target SLA |
|:---|:---:|:---:|
| **Idle RSS Memory (10 functions loaded)** | **20.33 MiB** | < 64.00 MiB |
| **Peak RSS Memory (Under full load)** | **51.29 MiB** | < 150.00 MiB |
| **Startup to Active Listener** | **725.26 ms** | < 1000.00 ms |
| **Swap Used** | **0 KiB** | 0 KiB |
| **Guest-Caused Daemon Crashes** | **0** | 0 |

---

## Reproducing the Benchmarks

You can reproduce these results on any Linux host:

```bash
# 1. Build test fixtures and components
bash scripts/build-fixtures.sh

# 2. Build release daemon binary
cargo build --release --bin wednesd

# 3. Execute automated benchmark harness
python3 benchmarks/run.py \
  --binary ./target/release/wednesd \
  --artifacts ./artifacts \
  --output benchmarks/my-report.json \
  --revision $(git rev-parse HEAD)
```
