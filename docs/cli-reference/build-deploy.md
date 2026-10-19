# Build & Deploy Workflows

This guide covers complete build and deployment workflows from source code to production deployment with `wednesd`.

---

## Directory Organization Best Practices

To ensure reliable relative artifact resolution, maintain a clean project structure:

```text
my-serverless-project/
├── wednes.yaml
├── registry.json
├── functions/
│   ├── auth/
│   │   ├── Cargo.toml
│   │   ├── manifest.toml
│   │   ├── src/lib.rs
│   │   └── auth.wasm
│   └── webhook/
│       ├── package.json
│       ├── manifest.toml
│       ├── src/index.ts
│       └── webhook.wasm
```

::: warning Path Resolution Rule
The `wednesd deploy` command resolves the compiled `.wasm` artifact relative to the manifest file's parent directory. Keep the `.wasm` file in the same directory or alongside `manifest.toml`.
:::

---

## Step 1: Compiling from Source

### Compiling Rust
Using `cargo` and `wit-bindgen`:

```bash
cd functions/auth
cargo build --target wasm32-wasip1 --release
cp target/wasm32-wasip1/release/auth.wasm ./auth.wasm
```

### Compiling TypeScript / JavaScript
Using [`@bytecodealliance/componentize-js`](https://github.com/bytecodealliance/ComponentizeJS):

```bash
cd functions/webhook
npm install
npm run build # Emits dist/index.js

# Componentize JavaScript into WebAssembly Component
npx componentize-js dist/index.js \
  -w ../../wit \
  -o webhook.wasm
```

### Compiling Python
Using [`componentize-py`](https://github.com/bytecodealliance/componentize-py):

```bash
cd functions/etl
componentize-py \
  -w ../../wit \
  -d . \
  -m app \
  componentize etl.wasm
```

---

## Step 2: Preparing Manifests

Each function folder contains a `manifest.toml`:

```toml
# functions/auth/manifest.toml
name = "auth"
abi = "wednes:function@0.1.0"
artifact = "auth.wasm"

[runtime]
memory_mb = 32
timeout_ms = 250
max_concurrency = 1
```

---

## Step 3: Deploying into Registry

Deploy each function into the shared `registry.json`:

```bash
# Deploy auth function
wednesd deploy \
  --manifest ./functions/auth/manifest.toml \
  --registry ./registry.json

# Deploy webhook function
wednesd deploy \
  --manifest ./functions/webhook/manifest.toml \
  --registry ./registry.json
```

### Inspecting `registry.json`

The registry stores registered functions with their metadata:

```json
{
  "version": "1.0",
  "functions": {
    "auth": {
      "name": "auth",
      "abi": "wednes:function@0.1.0",
      "artifact": "auth.wasm",
      "runtime": {
        "memory_mb": 32,
        "timeout_ms": 250,
        "max_concurrency": 1
      }
    },
    "webhook": {
      "name": "webhook",
      "abi": "wednes:function@0.1.0",
      "artifact": "webhook.wasm",
      "runtime": {
        "memory_mb": 64,
        "timeout_ms": 500,
        "max_concurrency": 1
      }
    }
  }
}
```

---

## Step 4: Automating Deployments in CI/CD

Create an automated deployment script `scripts/deploy.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

REGISTRY_PATH="/var/lib/wednes/registry.json"
mkdir -p "$(dirname "$REGISTRY_PATH")"

echo "Deploying functions to $REGISTRY_PATH..."

for manifest in ./functions/*/manifest.toml; do
  echo "Deploying $(dirname "$manifest")..."
  wednesd deploy --manifest "$manifest" --registry "$REGISTRY_PATH"
done

echo "Reloading daemon..."
systemctl reload-or-restart wednesd || true
echo "Deployment successful."
```
