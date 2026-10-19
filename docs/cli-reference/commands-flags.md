# Commands & Flags Reference

Detailed reference for all `wednesd` commands, arguments, and CLI options.

---

## Global Options

These options can be passed to `wednesd` at the top level or used in standalone mode.

| Flag | Short | Type | Default | Description |
|:---|:---:|:---:|:---:|:---|
| `--listen` | `-l` | string | `127.0.0.1:8080` | Host address and port to bind TCP and UDP gateway listeners. |
| `--wasm` | `-w` | path | `None` | Path to a WebAssembly `.wasm` component file (used in standalone mode). |
| `--id` | `-i` | string | `hello` | Ingress route identifier for standalone mode (mounted at `/fn/:id`). |
| `--global-concurrency` | | integer | `100` | Maximum number of simultaneous active executions allowed across the entire daemon. |
| `--memory-budget-mb` | | integer | `1200` | Global memory ceiling in megabytes for the daemon process. |
| `--default-concurrency` | | integer | `16` | Default concurrency limit for functions that do not specify `max_concurrency`. |
| `--log-json` | | boolean | `false` | Emits structured JSON logs instead of colored text. |
| `--http3` | | boolean | `false` | Enables the HTTP/3 QUIC listener on the UDP socket matching `--listen`. |

---

## Subcommands

### 1. `wednesd init`

Initializes a new function project directory with a default template and manifest.

```bash
wednesd init <name>
```

#### Arguments
- `<name>` *(Required)*: The name of the function to generate. Must be alphanumeric with dashes or underscores.

#### Example
```bash
wednesd init webhook-service
```

---

### 2. `wednesd dev`

Starts a hot-reloading development environment. Watches the project directory for changes to source code or manifests, automatically rebuilding and restarting the runtime daemon.

```bash
wednesd dev
```

#### Behavior
- Uses filesystem notification events (`create`, `modify`, `remove`).
- Triggers compilation and reloads in-memory instances immediately without manually killing and restarting the process.

---

### 3. `wednesd deploy`

Validates, precompiles, and registers a function manifest into an atomic registry.

```bash
wednesd deploy --manifest <MANIFEST_PATH> --registry <REGISTRY_PATH>
```

#### Options
| Flag | Short | Type | Required | Description |
|:---|:---:|:---:|:---:|:---|
| `--manifest` | `-m` | path | Yes | Path to the `manifest.toml` or `wednes.yaml` specification. |
| `--registry` | `-r` | path | Yes | Path to the target `registry.json` database file. |

#### Process Flow
1. Reads and deserializes the manifest.
2. Validates `name`, `abi` (`wednes:function@0.1.0`), and runtime constraints.
3. Verifies that the `.wasm` artifact exists and conforms to the WIT world.
4. Performs Cranelift precompilation and caching.
5. Performs an atomic file swap on `<registry>` to ensure zero corrupted reads.

#### Example
```bash
wednesd deploy \
  --manifest ./functions/auth/manifest.toml \
  --registry /var/lib/wednes/registry.json
```

---

### 4. `wednesd run`

Starts the production serverless daemon, reading registered functions and configurations from the registry database.

```bash
wednesd run --registry <REGISTRY_PATH> [OPTIONS]
```

#### Options
| Flag | Short | Type | Default | Description |
|:---|:---:|:---:|:---:|:---|
| `--registry` | `-r` | path | Required | Path to the `registry.json` database file. |
| `--listen` | `-l` | string | `127.0.0.1:8080` | Address and port to bind. |
| `--global-concurrency` | | integer | `100` | Global concurrency permit ceiling. |
| `--memory-budget-mb` | | integer | `1200` | Daemon memory budget cap. |
| `--default-concurrency` | | integer | `16` | Fallback concurrency per function. |
| `--http3` | | boolean | `false` | Enable HTTP/3 QUIC on UDP. |

#### Example
```bash
wednesd run \
  --registry /var/lib/wednes/registry.json \
  --listen 0.0.0.0:8080 \
  --global-concurrency 500 \
  --memory-budget-mb 2048 \
  --http3
```
