# Quickstart

This guide walks you through building, deploying, and serving your first WebAssembly serverless function with `wednesd`.

---

## Prerequisites

Before starting, ensure you have the following installed on your machine:

- **Linux / macOS** (x86_64 or aarch64)
- **Rust toolchain:** `rustc` and `cargo` (1.78+ recommended)
- **Wasm target:** `rustup target add wasm32-wasip1`
- **Node.js (optional, for TypeScript):** Node.js 18+ or 22+
- **Python (optional, for Python):** Python 3.10+
- **curl or httpie:** For testing HTTP endpoints

---

## 1. Install or Build `wednesd`

Clone the repository and build the daemon binary using Cargo:

```bash
git clone https://github.com/wednesdev-id/wednesdev-serverless-engine.git
cd wednesdev-serverless-engine

# Build optimized release binary
cargo build --release --bin wednesd

# Verify binary
./target/release/wednesd --help
```

For quick access, you can move `./target/release/wednesd` to `/usr/local/bin` or add target directory to your `$PATH`.

---

## 2. Create Your First Function (Rust)

Create a minimal Rust component that implements the `wednes:function@0.1.0` ABI.

### Initialize Project

```bash
cargo new --lib hello-world
cd hello-world
```

### Configure `Cargo.toml`

```toml
[package]
name = "hello-world"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = { version = "0.24", default-features = false, features = ["macros"] }
```

### Implement Handler in `src/lib.rs`

```rust
wit_bindgen::generate!({
    world: "function",
    inline: r#"
        package wednes:function@0.1.0;

        interface types {
            record header {
                name: string,
                value: string,
            }
            record request {
                method: string,
                path: string,
                headers: list<header>,
                body: list<u8>,
            }
            record response {
                status: u16,
                headers: list<header>,
                body: list<u8>,
            }
        }

        world function {
            use types.{request, response};
            export handle: func(req: request) -> response;
        }
    "#
});

use wednes::function::types::{Header, Request, Response};

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        let msg = format!(
            r#"{{"message": "Hello from Wednes Engine!", "method": "{}", "path": "{}"}}"#,
            req.method, req.path
        );

        Response {
            status: 200,
            headers: vec![Header {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
            body: msg.into_bytes(),
        }
    }
}

export!(Component);
```

### Build Wasm Component

```bash
cargo build --target wasm32-wasip1 --release
```

---

## 3. Run Directly (Legacy Single-Function Mode)

For quick development, you can start `wednesd` pointing directly to your compiled `.wasm` component:

```bash
wednesd --listen 127.0.0.1:8080 \
        --wasm target/wasm32-wasip1/release/hello_world.wasm \
        --id hello
```

In a second terminal, invoke the function via HTTP:

```bash
curl -X POST http://127.0.0.1:8080/fn/hello \
     -H "Content-Type: application/json" \
     -d '{"name": "Alice"}'
```

Output:
```json
{"message": "Hello from Wednes Engine!", "method": "POST", "path": "/fn/hello"}
```

---

## 4. Production Multi-Function Deployment via Registry

For production environments, Wednes Engine uses declarative manifests and an atomic registry.

### Step 4.1: Write Manifest (`manifest.toml` or `wednes.yaml`)

Create `manifest.toml`:

```toml
name = "api-gateway"
abi = "wednes:function@0.1.0"
artifact = "hello_world.wasm"

[runtime]
memory_mb = 64
timeout_ms = 500
max_concurrency = 1
```

::: tip
Artifact path in `manifest.toml` must be in the same directory as the manifest, or a relative basename.
:::

### Step 4.2: Deploy Function to Registry

```bash
wednesd deploy \
  --manifest ./manifest.toml \
  --registry ./registry.json
```

Output:
```text
2026-10-19T10:00:00Z  INFO wednesd: Precompiling artifact: hello_world.wasm
2026-10-19T10:00:00Z  INFO wednesd: Deployed function 'api-gateway' to registry "registry.json"
```

### Step 4.3: Start Daemon with Registry

```bash
wednesd run \
  --registry ./registry.json \
  --listen 0.0.0.0:8080 \
  --global-concurrency 500 \
  --memory-budget-mb 1024
```

### Step 4.4: Invoke Endpoint

Test your deployed endpoint:

```bash
curl http://127.0.0.1:8080/fn/api-gateway/users
```

---

## 5. Enable HTTP/3 (QUIC)

To serve traffic over ultra-low latency HTTP/3 QUIC on UDP:

```bash
wednesd run \
  --registry ./registry.json \
  --listen 0.0.0.0:8443 \
  --http3
```

`wednesd` will automatically generate self-signed TLS certificates and bind a QUIC listener alongside standard HTTP.
