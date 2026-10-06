# Wednes Serverless Engine

Wednes Engine adalah runtime Serverless WebAssembly berbasis **Wasmtime Component Model** yang dirancang untuk performa tinggi, cold start mendekati 0ms, dan hemat resource. Engine ini mendukung arsitektur multi-fungsi modern melalui manifest `wednes.yaml`.

---

## 🚀 Live Deployment via Web IDE (`cp.wednesdev.id`)

Anda dapat langsung membuat, mengedit, men-download, dan men-deploy fungsi serverless melalui Control Plane:
1. Buka **[cp.wednesdev.id](https://cp.wednesdev.id)**.
2. Klik tombol **`+ New`** di panel Explorer.
3. Pilih bahasa yang diinginkan (**Rust**, **Go**, **TypeScript**, atau **Python**).
4. Klik **Create & Open in IDE**.
5. Tekan tombol **▶ Run / Deploy** di sudut kanan atas.
6. Function langsung live dan dapat diakses publik melalui endpoint:
   `https://cp.wednesdev.id/api/run/{project_name}-{function_name}` atau via HTTP POST/GET.

---

## 📚 Katalog Studi Kasus (Siap Copy-Paste)

Setiap contoh di bawah ini dirancang untuk struktur multi-fungsi atau single-fungsi menggunakan ABI `wednes:function@0.1.0`.

---

### 1. TypeScript / JavaScript

#### Sederhana: Basic JSON Response & Greeting
File: `src/index.ts`
```typescript
export function handle(req: any) {
  const payload = {
    status: 200,
    message: "Hello from Wednes Serverless TypeScript!",
    timestamp: new Date().toISOString()
  };

  return {
    status: 200,
    headers: [{ name: "content-type", value: "application/json" }],
    body: Array.from(new TextEncoder().encode(JSON.stringify(payload)))
  };
}
```

#### Kompleks: WhatsApp CRM Webhook Handler (Auto-Reply & Payload Parsing)
File: `src/whatsapp.ts`
```typescript
export function handle(req: any) {
  const method = req.method;
  
  // 1. WhatsApp Verification Challenge (GET)
  if (method === "GET") {
    return {
      status: 200,
      headers: [{ name: "content-type", value: "text/plain" }],
      body: Array.from(new TextEncoder().encode("CHALLENGE_ACCEPTED"))
    };
  }

  // 2. Incoming Message Processing (POST)
  if (method === "POST") {
    try {
      const bodyStr = new TextDecoder().decode(new Uint8Array(req.body));
      const data = JSON.parse(bodyStr);

      const sender = data.entry?.[0]?.changes?.[0]?.value?.messages?.[0]?.from || "unknown";
      const userText = data.entry?.[0]?.changes?.[0]?.value?.messages?.[0]?.text?.body || "";

      // Logic CRM Auto-Responder
      const reply = {
        recipient_type: "individual",
        to: sender,
        type: "text",
        text: { body: `Halo! Kami menerima pesan Anda: "${userText}". Tim CRM Wednesdev akan segera menghubungi Anda.` }
      };

      return {
        status: 200,
        headers: [{ name: "content-type", value: "application/json" }],
        body: Array.from(new TextEncoder().encode(JSON.stringify(reply)))
      };
    } catch (e: any) {
      return {
        status: 400,
        headers: [{ name: "content-type", value: "application/json" }],
        body: Array.from(new TextEncoder().encode(JSON.stringify({ error: e.message })))
      };
    }
  }

  return { status: 405, headers: [], body: [] };
}
```

---

### 2. Python

#### Sederhana: Simple Health Check & System Status
File: `app.py`
```python
from wit_world.imports.types import Request, Response, Header
import json

class WitWorld:
    def handle(self, req: Request) -> Response:
        res_data = {
            "status": "healthy",
            "runtime": "wednesd-wasm-python",
            "method": req.method,
            "path": req.path
        }
        return Response(
            status=200,
            headers=[Header(name="content-type", value="application/json")],
            body=json.dumps(res_data).encode("utf-8")
        )
```

#### Kompleks: Web Scraper & Shopify Webhook Processor
File: `app.py`
```python
from wit_world.imports.types import Request, Response, Header
import json
import urllib.request

class WitWorld:
    def handle(self, req: Request) -> Response:
        path = req.path
        method = req.method

        # Route A: Web Scraper / Product Extractor
        if path == "/api/scrape":
            try:
                # Mengambil data dari upstream API/Website
                req_obj = urllib.request.Request(
                    "https://dummyjson.com/products/1",
                    headers={"User-Agent": "WednesServerless/1.0"}
                )
                with urllib.request.urlopen(req_obj) as resp:
                    raw_data = resp.read()
                    product = json.loads(raw_data)
                    
                output = {
                    "scraped_title": product.get("title"),
                    "price": product.get("price"),
                    "source": "Wednes Scraper WASM"
                }
                return Response(
                    status=200,
                    headers=[Header(name="content-type", value="application/json")],
                    body=json.dumps(output).encode("utf-8")
                )
            except Exception as e:
                return Response(
                    status=500,
                    headers=[Header(name="content-type", value="application/json")],
                    body=json.dumps({"error": str(e)}).encode("utf-8")
                )

        # Route B: Shopify Order Created Webhook
        elif path == "/webhook/shopify" and method == "POST":
            payload = json.loads(bytes(req.body).decode("utf-8"))
            order_id = payload.get("id", "N/A")
            total = payload.get("total_price", "0.00")

            ack = {
                "acknowledged": True,
                "order_id": order_id,
                "total_processed": total,
                "system": "Wednes Engine CRM"
            }
            return Response(
                status=200,
                headers=[Header(name="content-type", value="application/json")],
                body=json.dumps(ack).encode("utf-8")
            )

        return Response(status=404, headers=[], body=b'{"error": "Route Not Found"}')
```

---

### 3. Go (Golang)

#### Sederhana: Echo Endpoint & Query Parser
File: `src/main.go`
```go
package main

import (
	"encoding/json"
	"active-function/function"
)

type MyFunction struct{}

func (m MyFunction) Handle(req function.FunctionRequest) function.FunctionResponse {
	respData, _ := json.Marshal(map[string]interface{}{
		"status":  "ok",
		"engine":  "Wednes TinyGo WASM",
		"path":    req.Path,
		"headers": len(req.Headers),
	})

	return function.FunctionResponse{
		Status: 200,
		Headers: []function.WednesFunction0_1_0_TypesHeader{
			{Name: "content-type", Value: "application/json"},
		},
		Body: respData,
	}
}

func init() {
	function.SetFunction(MyFunction{})
}

func main() {}
```

#### Kompleks: Financial Transaction Validator & Tax Calculator
File: `src/main.go`
```go
package main

import (
	"encoding/json"
	"active-function/function"
)

type TransactionRequest struct {
	Subtotal float64 `json:"subtotal"`
	TaxRate  float64 `json:"tax_rate"`
	Discount float64 `json:"discount"`
}

type TransactionResult struct {
	Subtotal float64 `json:"subtotal"`
	Tax      float64 `json:"tax"`
	Total    float64 `json:"total"`
	Status   string  `json:"status"`
}

type MyFunction struct{}

func (m MyFunction) Handle(req function.FunctionRequest) function.FunctionResponse {
	if req.Method != "POST" {
		return function.FunctionResponse{
			Status: 405,
			Body:   []byte(`{"error": "Method Not Allowed"}`),
		}
	}

	var tx TransactionRequest
	err := json.Unmarshal(req.Body, &tx)
	if err != nil {
		return function.FunctionResponse{
			Status: 400,
			Body:   []byte(`{"error": "Invalid JSON format"}`),
		}
	}

	// Hitung PPN & Total
	taxAmount := (tx.Subtotal - tx.Discount) * (tx.TaxRate / 100.0)
	finalTotal := (tx.Subtotal - tx.Discount) + taxAmount

	res := TransactionResult{
		Subtotal: tx.Subtotal,
		Tax:      taxAmount,
		Total:    finalTotal,
		Status:   "VALIDATED_BY_WEDNES",
	}

	body, _ := json.Marshal(res)

	return function.FunctionResponse{
		Status: 200,
		Headers: []function.WednesFunction0_1_0_TypesHeader{
			{Name: "content-type", Value: "application/json"},
		},
		Body: body,
	}
}

func init() {
	function.SetFunction(MyFunction{})
}

func main() {}
```

---

### 4. Rust

#### Sederhana: Static Route & Ping-Pong
File: `src/lib.rs`
```rust
wit_bindgen::generate!({
    world: "function",
    path: "wit",
});

use wednes::function::types::{Header, Request, Response};

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        let body = format!(
            r#"{{"status": "ok", "path": "{}", "engine": "Wednes Rust Wasmtime"}}"#,
            req.path
        );

        Response {
            status: 200,
            headers: vec![Header {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
            body: body.into_bytes(),
        }
    }
}

export!(Component);
```

#### Kompleks: Token Authentication & API Gateway Dispatcher
File: `src/lib.rs`
```rust
wit_bindgen::generate!({
    world: "function",
    path: "wit",
});

use wednes::function::types::{Header, Request, Response};

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        // 1. Validasi Authorization Header
        let is_authorized = req.headers.iter().any(|h| {
            h.name.to_lowercase() == "authorization" && h.value == "Bearer wednes-secret-token"
        });

        if !is_authorized {
            return Response {
                status: 401,
                headers: vec![Header {
                    name: "content-type".to_string(),
                    value: "application/json".to_string(),
                }],
                body: br#"{"error": "Unauthorized Access"}"#.to_vec(),
            };
        }

        // 2. Dispatching Path
        let res_body = match req.path.as_str() {
            "/api/v1/users" => {
                r#"{"users": [{"id": 1, "name": "Feri"}, {"id": 2, "name": "Wednes"}]}"#
            }
            "/api/v1/stats" => {
                r#"{"uptime": "99.99%", "active_sandboxes": 10, "latency_us": 240}"#
            }
            _ => {
                return Response {
                    status: 404,
                    headers: vec![],
                    body: br#"{"error": "Not Found"}"#.to_vec(),
                };
            }
        };

        Response {
            status: 200,
            headers: vec![Header {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
            body: res_body.as_bytes().to_vec(),
        }
    }
}

export!(Component);
```

---

## ⚙️ Menjalankan Multi-Fungsi (`wednes.yaml`)

Untuk menggabungkan banyak fungsi dalam satu project, buat file `wednes.yaml` di root direktori project Anda:

```yaml
version: "1.0"
name: "crm-service"

functions:
  whatsapp:
    handler: src/whatsapp.ts
    route: /webhook/whatsapp

  shopify:
    handler: src/shopify.ts
    route: /webhook/shopify

  scraper:
    handler: src/scraper.py
    route: /api/scrape
```

Saat Anda menekan tombol **Deploy**, Wednes Engine akan secara otomatis mengompilasi masing-masing fungsi ke artifact WebAssembly terpisah dan meregistrasikannya ke daemon runtime.
