# Rust SDK

The **Wednes Rust SDK** provides zero-overhead, type-safe bindings for building serverless WebAssembly components adhering to the `wednes:function@0.1.0` ABI.

---

## Installation & Setup

Add `wednes-sdk` and `wit-bindgen` to your function's `Cargo.toml`:

```toml
[package]
name = "my-function"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wednes-sdk = { path = "../../sdk/rust" } # Or from crates.io / git
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## Core Types and Helpers

The SDK wraps WIT generated bindings into idiomatic Rust types:

### Request Methods

| Method | Return Type | Description |
|:---|:---|:---|
| `req.text()` | `Result<String, FromUtf8Error>` | Decodes request body as UTF-8 string. |
| `req.text_lossy()` | `Cow<'_, str>` | Decodes request body with replacement characters for invalid UTF-8. |
| `req.json::<T>()` | `Result<T, serde_json::Error>` | Deserializes JSON request body directly into a typed struct. |
| `req.header(name)` | `Option<&str>` | Case-insensitive lookup of a request header value. |

### Response Builders

| Builder Method | Description |
|:---|:---|
| `Response::new(status)` | Creates a response with empty headers and body. |
| `Response::text(status, body)` | Creates a plain-text response with `content-type: text/plain`. |
| `Response::json(status, json_str)` | Creates a JSON response with `content-type: application/json`. |
| `Response::json_val(status, &struct)` | Serializes a Rust value to JSON and sets headers. |
| `res.with_header(name, value)` | Fluent builder method to attach response headers. |

---

## Examples

### 1. Minimal JSON API

```rust
use wednes_sdk::{export, Guest, Request, Response};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct GreetRequest {
    name: String,
}

#[derive(Serialize)]
struct GreetResponse {
    greeting: String,
    timestamp: u64,
}

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        match req.json::<GreetRequest>() {
            Ok(body) => {
                let res = GreetResponse {
                    greeting: format!("Hello, {}!", body.name),
                    timestamp: 1729300000,
                };
                Response::json_val(200, &res).unwrap()
            }
            Err(e) => {
                Response::json(400, format!(r#"{{"error": "Invalid JSON: {}"}}"#, e))
            }
        }
    }
}

export!(Component);
```

### 2. Authorization Header Dispatcher

```rust
use wednes_sdk::{export, Guest, Request, Response};

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        // Inspect authorization header
        match req.header("authorization") {
            Some(auth) if auth == "Bearer wednes-secret-token" => {
                Response::json(200, r#"{"status": "authenticated", "access": "admin"}"#)
            }
            Some(_) => {
                Response::json(403, r#"{"error": "Forbidden: invalid token"}"#)
            }
            None => {
                Response::json(401, r#"{"error": "Unauthorized: missing header"}"#)
                    .with_header("WWW-Authenticate", "Bearer")
            }
        }
    }
}

export!(Component);
```

---

## Compiling to WASM

Compile using the standard `wasm32-wasip1` target:

```bash
cargo build --target wasm32-wasip1 --release
```

The resulting artifact `target/wasm32-wasip1/release/my_function.wasm` is ready for deployment.
