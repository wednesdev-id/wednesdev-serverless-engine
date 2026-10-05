# Ubuntu 2 GB benchmark

Revision: `unknown`. Measured on `Linux-6.8.0-139-generic-x86_64-with-glibc2.39`.
RAM: 2014860 KiB; CPUs: 2; 10 functions; 32 MiB guest memory; 250 ms timeout.

| Metric | Measured | Target |
|---|---:|---:|
| Idle RSS (10 functions) | 15.65 MiB | 64 MiB (10 functions: 150 MiB) |
| Startup to listener | 44.52 ms | <500 ms |
| Peak RSS | 46.79 MiB | measured |
| Swap used | 0 KiB | 0 |
| Guest-caused crashes | 0 | 0 |

| Case | p50 ms | p95 ms | p99 ms |
|---|---:|---:|---:|
| hello | 0.751 | 0.998 | 1.091 |
| json_api | 0.694 | 0.827 | 0.875 |
| webhook | 0.899 | 0.984 | 1.066 |
| cpu | 0.950 | 1.039 | 1.072 |
| memory | 0.970 | 1.112 | 1.133 |
| large-response | 1.322 | 1.563 | 1.592 |
| warm-start | 0.751 | 0.998 | 1.091 |
| infinite_loop | 249.509 | 252.741 | 252.741 |
| oom | 20.572 | 70.938 | 70.938 |
| trap/trap | 1.330 | 1.416 | 1.416 |
| concurrency | 6.633 | 10.067 | 10.067 |
| cold-start | 1.790 | 5.119 | 54.002 |

Cold: first request on 20 fresh daemon processes. Warm: fresh guest instance, same compiled in-memory component. Python loopback HTTP latency includes client overhead. 16 barrier-synchronized clients do not prove 16 guest executions overlap. Startup recompiles source WASM; persistent cross-process cache is NOT implemented. No results inferred for unsupported features.

Reproduce: build components using `bash benchmarks/build.sh`, copy binary, components and this harness to Ubuntu; run `python3 benchmarks/run.py --binary ./wednesd --artifacts ./artifacts --output benchmarks/report-ubuntu-2gb.json --revision <git-sha>`. JSON contains raw latency samples, artifact hashes, hardware and configuration. Timeout/OOM/trap each checked three times, followed by success on same PID. Malformed deploy must leave registry byte-identical.
