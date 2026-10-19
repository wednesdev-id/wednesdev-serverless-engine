# `wednes.yaml` Specification

The `wednes.yaml` file (and its TOML equivalent `manifest.toml`) serves as the declarative configuration contract for functions and services running on **Wednes Serverless Engine**.

It specifies routing paths, runtime resource bounds, compilation handlers, and capability permissions.

---

## Minimal Specification Example

```yaml
version: "1.0"
name: "payment-gateway"

functions:
  charge:
    handler: src/charge.ts
    route: /api/v1/charge
    runtime:
      memory_mb: 64
      timeout_ms: 250
      max_concurrency: 1

  webhook:
    handler: src/webhook.py
    route: /api/v1/webhook
    runtime:
      memory_mb: 32
      timeout_ms: 1000
      max_concurrency: 1
```

---

## Schema Reference

### Top-Level Attributes

| Field | Type | Required | Description |
|:---|:---:|:---:|:---|
| `version` | string | Yes | Manifest schema version. Currently `"1.0"`. |
| `name` | string | Yes | Unique project identifier. Must contain only alphanumeric characters, dashes (`-`), and underscores (`_`). |
| `functions` | map | Yes | Dictionary of function configurations keyed by function name. |

---

### Function Definition Attributes

Each function defined under `functions.<key>` supports the following attributes:

| Field | Type | Default | Description |
|:---|:---:|:---:|:---|
| `handler` | string | Required | Source entry file (e.g., `src/index.ts`, `src/lib.rs`, `app.py`) or compiled artifact (e.g., `dist/function.wasm`). |
| `route` | string | Required | Ingress HTTP route path mounted under the function gateway. |
| `runtime` | object | Optional | Per-function execution limits and runtime sandbox constraints. |
| `capabilities` | object | Optional | Sandbox permission declarations (see Capabilities section below). |

---

### Runtime Constraints (`runtime`)

Runtime constraints are passed directly into Wasmtime's resource limits and the epoch scheduler.

```yaml
runtime:
  memory_mb: 64       # Allowed range: 1 to 2048 MB
  timeout_ms: 500     # Allowed range: 1 to 60000 ms (up to 60s)
  max_concurrency: 1  # Concurrent invocations per instance permit
```

| Parameter | Type | Validation Rules | Default | Description |
|:---|:---:|:---|:---:|:---|
| `memory_mb` | integer | `1 <= memory_mb <= 2048` | `32` | Maximum linear memory allocation for the guest instance. Invocations attempting to grow beyond this limit trigger an immediate `MemoryOutOfBounds` / OOM trap. |
| `timeout_ms` | integer | `1 <= timeout_ms <= 60000` | `250` | Maximum wall-clock execution time. Handled by background epoch ticks. Exceeding this triggers an epoch trap. |
| `max_concurrency` | integer | `>= 1` | `1` | Semaphore slots allocated for concurrent execution of this function. |

---

### Capabilities Declaration (`capabilities`)

Wednes enforces a zero-trust capability model. Functions run without host privileges by default:

```yaml
capabilities:
  filesystem: false
  network_outbound: false
  inherited_env: false
```

::: warning Security Constraint
In the current M3 specification, outbound HTTP, ambient host environment inheritance, and arbitrary host filesystem access are strictly disabled. Any manifest specifying unsupported capabilities will fail deployment validation to prevent insecure host leaks.
:::

---

## Single-Function TOML Manifest (`manifest.toml`)

When deploying standalone precompiled components directly via the CLI, you can use the single-function TOML format:

```toml
name = "auth-validator"
abi = "wednes:function@0.1.0"
artifact = "auth_validator.wasm"

[runtime]
memory_mb = 64
timeout_ms = 300
max_concurrency = 1
```

### Validation Rules for Artifacts

To prevent directory traversal attacks:
- The `artifact` field must refer to a direct basename (e.g., `auth_validator.wasm`).
- Paths containing `/`, `\`, or `..` are strictly rejected by the validator.
- The artifact must reside in the same directory as the manifest at deployment time.

---

## Multi-Language Example

A full real-world `wednes.yaml` demonstrating a multi-service architecture using Rust, TypeScript, and Python:

```yaml
version: "1.0"
name: "ecommerce-suite"

functions:
  # Fast token authentication in Rust
  authenticator:
    handler: crates/auth/src/lib.rs
    route: /api/auth
    runtime:
      memory_mb: 16
      timeout_ms: 50

  # High-throughput webhook handler in TypeScript
  stripe-webhook:
    handler: services/stripe/index.ts
    route: /webhooks/stripe
    runtime:
      memory_mb: 64
      timeout_ms: 500

  # Async data processing in Python
  analytics-etl:
    handler: services/analytics/app.py
    route: /internal/etl
    runtime:
      memory_mb: 128
      timeout_ms: 5000
```
