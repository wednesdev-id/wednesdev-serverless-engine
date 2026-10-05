# Ubuntu 2 GB benchmark

Revision: `deea938`. Measured on `Linux-6.8.0-139-generic-x86_64-with-glibc2.39`.
RAM: 2014860 KiB; CPUs: 2; 10 functions; 32 MiB guest memory; 250 ms timeout.

| Metric | Measured | Target |
|---|---:|---:|
| Idle RSS (10 functions) | 21.50 MiB | 64 MiB (10 functions: 150 MiB) |
| Startup to listener | 685.16 ms | <500 ms |
| Peak RSS | 52.36 MiB | measured |
| Swap used | 0 KiB | 0 |
| Guest-caused crashes | 0 | 0 |

| Case | p50 ms | p95 ms | p99 ms |
|---|---:|---:|---:|
| hello | 0.927 | 1.581 | 2.141 |
| json_api | 0.860 | 1.001 | 1.092 |
| webhook | 0.858 | 1.000 | 1.025 |
| cpu | 0.798 | 0.912 | 0.989 |
| memory | 0.784 | 0.879 | 0.959 |
| large-response | 1.007 | 1.169 | 1.353 |
| warm-start | 0.927 | 1.581 | 2.141 |
| infinite_loop | 275.634 | 276.080 | 276.080 |
| oom | 20.428 | 42.641 | 42.641 |
| trap/trap | 1.602 | 1.701 | 1.701 |
| concurrency | 6.637 | 11.408 | 11.408 |
| cold-start | 1.515 | 1.860 | 35.124 |
| startup-to-listener | 691.043 | 765.543 | 767.986 |

Cold: first request on 20 fresh daemon processes. Warm: fresh guest instance, same compiled in-memory component. Python loopback HTTP latency includes client overhead. 16 barrier-synchronized clients do not prove 16 guest executions overlap. Startup recompiles source WASM; persistent cross-process cache is NOT implemented. No results inferred for unsupported features.

Reproduce: build components using `bash benchmarks/build.sh`, copy binary, components and this harness to Ubuntu; run `python3 benchmarks/run.py --binary ./wednesd --artifacts ./artifacts --output benchmarks/report-ubuntu-2gb.json --revision <git-sha>`. JSON contains raw latency samples, artifact hashes, hardware and configuration. Timeout/OOM/trap each checked three times, followed by success on same PID. Malformed deploy must leave registry byte-identical.
