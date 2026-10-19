# Overview

**Wednes Serverless Engine** (`wednesd`) is a high-performance, edge-ready serverless runtime engineered around the WebAssembly (Wasm) Component Model and the Cranelift optimizing JIT compiler.

Designed for modern cloud workloads, microservices, and AI edge pipelines, it combines sub-millisecond execution with memory density and zero-trust sandboxing.

---

## Key Highlights

- **Near-Zero Cold Start:** Microsecond-level instance instantiation powered by pre-instantiated component templates (`instantiate_pre`).
- **Standardized ABI:** Interoperates through the official WIT interface `wednes:function@0.1.0`.
- **Multi-Protocol Gateway:** Integrated HTTP/1.1, HTTP/2, and HTTP/3 (QUIC over UDP) with automated self-signed or production TLS certificates.
- **Micro-Sandboxing & Isolation:** Hardened memory budgets (via Wasmtime `StoreLimits`), deterministic execution timeouts (via epoch-based trapping), and strict isolation preventing unauthorized host filesystem access or socket escape.
- **Atomic Registries:** Safe function updates and zero-downtime metadata swaps using transactional tempfile renames.
- **Polyglot Developer Experience:** First-class compilation pipelines for Rust, TypeScript/JavaScript, and Python.

---

## When to Use Wednes Serverless Engine

Wednes Serverless Engine excels in scenarios where traditional container-based systems (like Docker, Kubernetes Pods, or Knative) introduce excessive resource overhead and latency:

1. **High-Density Webhooks & Event Processing:** Running hundreds or thousands of distinct client endpoints on a single small node (e.g., 2 GB RAM) without running out of memory.
2. **Ultra-Low Latency APIs:** Handling authentication dispatchers, JWT verifications, edge transformers, or data filtering with sub-millisecond response guarantees.
3. **Multi-Tenant Plugin Systems:** Securely executing third-party or untrusted business logic without risking host compromise.
4. **Edge & IoT Micro-Gateways:** Deploying lightweight services to resource-constrained cloud servers or edge devices with full HTTP/3 QUIC connectivity.

---

## Design Principles

### 1. Minimal Footprint, Maximum Density
Rather than booting a Linux kernel or isolated guest OS for every workload, Wednes uses lightweight WebAssembly instances. Ten active functions consume as little as **20.33 MiB** of idle RSS memory.

### 2. Zero-Trust Sandboxing
By default, functions run in a closed world:
- No ambient environment variables.
- No direct host filesystem preopens.
- No outbound socket creation unless explicitly authorized.
- Strictly enforced CPU and memory limits per invocation.

### 3. Native Cloud-Native Standards
Wednes adheres to CNCF and bytecode alliance standards, including:
- WebAssembly System Interface (WASI).
- WebAssembly Component Model (`wit`).
- Standard Prometheus/JSON telemetry.
- Open Container and systemd packaging.

---

## Next Steps

- Jump straight into building your first function in the [Quickstart Guide](./quickstart).
- Explore runtime internals in the [Architecture Specification](./architecture).
- Inspect the [wednes.yaml Specification](../core-concepts/wednes-yaml).
