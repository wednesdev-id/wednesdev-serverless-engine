# Ubuntu 2 GB benchmark

Revision: `deea9384cae5009c4efab5ceac5fc392fb761427`. Measured on `Linux-6.8.0-139-generic-x86_64-with-glibc2.39`.
RAM: 2014860 KiB; CPUs: 2; 10 functions; 32 MiB guest memory; 250 ms timeout.

| Metric | Measured | Target |
|---|---:|---:|
| Idle RSS (10 functions) | 20.33 MiB | 64 MiB (10 functions: 150 MiB) |
| Startup to listener | 725.26 ms | <500 ms |
| Peak RSS | 51.29 MiB | measured |
| Swap used | 0 KiB | 0 |
| Guest-caused crashes | 0 | 0 |

| Case | p50 ms | p95 ms | p99 ms |
|---|---:|---:|---:|
| hello | 0.845 | 0.989 | 1.070 |
| json_api | 0.837 | 1.007 | 1.129 |
| webhook | 0.818 | 0.913 | 0.929 |
| cpu | 0.814 | 0.893 | 0.994 |
| memory | 0.826 | 0.982 | 1.091 |
| large-response | 1.075 | 1.341 | 1.437 |
| warm-start | 0.845 | 0.989 | 1.070 |
| infinite_loop | 276.045 | 276.229 | 276.229 |
| oom | 23.449 | 26.271 | 26.271 |
| trap/trap | 1.120 | 1.387 | 1.387 |
| concurrency | 8.117 | 12.066 | 12.066 |
| cold-start | 1.472 | 5.665 | 34.221 |
| startup-to-listener | 686.926 | 714.128 | 725.256 |

Cold: first request on 20 fresh daemon processes. Warm: fresh guest instance, same compiled in-memory component. Python loopback HTTP latency includes client overhead. 16 barrier-synchronized clients do not prove 16 guest executions overlap. Startup recompiles source WASM; persistent cross-process cache is NOT implemented. No results inferred for unsupported features.

Reproduce: build components using `bash benchmarks/build.sh`, copy binary, components and this harness to Ubuntu; run `python3 benchmarks/run.py --binary ./wednesd --artifacts ./artifacts --output benchmarks/report-ubuntu-2gb.json --revision <git-sha>`. JSON contains raw latency samples, artifact hashes, hardware and configuration. Timeout/OOM/trap each checked three times, followed by success on same PID. Malformed deploy must leave registry byte-identical.
