# Wednes Serverless Engine

Wednes Engine is an ultra-fast, WebAssembly Component Model-based serverless engine designed for Web, AI, and Microservices workloads.

## Features
- **Cold Start < 5ms**: Execution via Wasmtime.
- **Polyglot**: Write in Rust, Go, TypeScript/JS, or Python.
- **Framework Compatibility**: You can use familiar Web Frameworks (Axum, Fiber, Express, FastAPI) by bridging requests.

## How It Works
The engine uses a custom ABI (`wednes:function@0.1.0`):
```wit
interface types {
  record header { name: string, value: string }
  record request { method: string, path: string, headers: list<header>, body: list<u8> }
  record response { status: u16, headers: list<header>, body: list<u8> }
}

world function {
  use types.{request, response};
  export handle: func(req: request) -> response;
}
```

---

## Web Framework Integration (Microservices & Chat CRM)

To minimize the technical gap, you can wrap your existing Web Framework inside the `handle` export. Below are examples of using familiar frameworks to build practical applications like WhatsApp CRM integration or Shopify Webhooks.

### 1. Python (FastAPI Adapter)
Using `componentize-py`. You can use the popular `asgi-tools` or write a lightweight WSGI/ASGI wrapper.

**Scraping / Shopify Integration Example:**
```python
from wit_world.imports.types import Request, Response, Header
import json
import urllib.request # Native requests

class WitWorld:
    def handle(self, req: Request) -> Response:
        path = req.path
        method = req.method
        
        # Simple Framework Routing Logic
        if path == "/webhook/shopify" and method == "POST":
            # Process Shopify Webhook
            payload = json.loads(bytes(req.body).decode('utf-8'))
            return Response(status=200, headers=[], body=b'{"status": "received"}')
            
        elif path == "/api/scrape":
            # Scraping logic
            res = urllib.request.urlopen("https://dummyjson.com/products/1")
            data = res.read()
            return Response(status=200, headers=[], body=data)

        return Response(status=404, headers=[], body=b'Not Found')
```

### 2. TypeScript / Express-like (Componentize-JS)

**WhatsApp CRM Webhook Example:**
```typescript
export function handle(req: any) {
  const path = req.path;
  const method = req.method;
  
  if (path === "/webhook/whatsapp" && method === "POST") {
    const bodyStr = new TextDecoder().decode(new Uint8Array(req.body));
    const payload = JSON.parse(bodyStr);
    
    // CRM Logic here: auto-reply to customer
    const reply = {
      messaging_product: "whatsapp",
      to: payload.entry[0].changes[0].value.messages[0].from,
      text: { body: "Hello from Wednes CRM Serverless!" }
    };
    
    return {
      status: 200,
      headers: [{ name: "content-type", value: "application/json" }],
      body: Array.from(new TextEncoder().encode(JSON.stringify(reply)))
    };
  }

  return { status: 404, headers: [], body: [] };
}
```

### 3. Rust (Axum-like routing macro)
For Rust, the SDK (`wednes_sdk`) exports the structures. You can map the incoming `req: Request` to an `http::Request<Vec<u8>>` if you want to use the `axum` or `router` ecosystem inside the WASM sandbox.

```rust
use wednes_sdk::{export, Request, Response, Header};

struct Component;

impl wednes_sdk::function::Guest for Component {
    fn handle(req: Request) -> Response {
        match (req.method.as_str(), req.path.as_str()) {
            ("POST", "/crm/message") => {
                let msg = std::str::from_utf8(&req.body).unwrap();
                Response::json(200, format!("{\"reply\": \"Received: {}\"}", msg))
            },
            _ => Response::json(404, "{\"error\": \"Not Found\"}")
        }
    }
}
export!(Component);
```

## Outbound HTTP (Scraping)
Outbound HTTP fetching requires the `wasi:http/outgoing-handler` capability. Currently, Wednes Engine executes components with restricted capabilities to prevent unauthorized network access. Scraping can be achieved natively via standard library tools (like `urllib` in Python or `fetch` in JS) once the Outbound HTTP capability is enabled in your function manifest.

