# Wednes Engine

Lightweight, high-density WebAssembly function runtime powered by Rust and Wasmtime. `wednesd` provides sandboxed execution, memory limits, and admission control without Docker, Kubernetes, or database dependencies.

---

## 1. Quickstart: Scaffold & Dev Mode

### Scaffold Function Baru
Gunakan perintah `init` untuk membuat struktur awal fungsi:
```bash
./target/release/wednesd init payment-processor
cd payment-processor
```

Hasil scaffold direktori:
```text
payment-processor/
├── Cargo.toml
├── manifest.toml
└── src/
    └── lib.rs
```

Contoh `manifest.toml`:
```toml
name = "payment-processor"
abi = "wednes:function@0.1.0"
artifact = "target/payment_processor.wasm"

[runtime]
memory_mb = 64
timeout_ms = 3000
max_concurrency = 32

[capabilities]
logging = true
clock = true
outbound_http = true
filesystem = false
```

### Dev Mode (Watch & Auto-reload)
Jalankan live development server di lokal:
```bash
./target/release/wednesd dev
```

---

## 2. Contoh Implementasi

### A. Contoh Sederhana (JSON Greeting)
File: `src/lib.rs`
```rust
use wednes_sdk::{export, Guest, Request, Response};

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        let body = format!(r#"{{"status":"ok","path":"{}"}}"#, req.path);
        Response::json(200, body)
    }
}

export!(Component);
```

### B. Validasi Webhook & Parsing Body
Mengekstrak payload JSON dengan helper `wednes-sdk`:
```rust
use serde::{Deserialize, Serialize};
use wednes_sdk::{export, Guest, Request, Response};

#[derive(Deserialize)]
struct WebhookEvent {
    event_id: String,
    amount: u64,
}

#[derive(Serialize)]
struct AckResponse {
    received: bool,
    event_id: String,
}

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        match req.json::<WebhookEvent>() {
            Ok(event) => {
                let ack = AckResponse {
                    received: true,
                    event_id: event.event_id,
                };
                Response::json(200, serde_json::to_string(&ack).unwrap())
            }
            Err(_) => Response::json(400, r#"{"error":"Invalid payload"}"#),
        }
    }
}

export!(Component);
```

### C. Menangani Transaksi Skala Besar (High Concurrency & Outbound)
Menangani ribuan request transaksi per detik dengan proteksi resource:
1. **Memory Budget & Admission:** Engine menerapkan admission control berbasis semaphore dan memory budget. Request berlebih otomatis ditolak dengan `429 Too Many Requests` tanpa memicu crash host.
2. **Outbound Capability:** Fungsi dapat memvalidasi transaksi ke API pihak ketiga via capability `outbound_http`.
3. **Hard Request Limit:** Gateway membatasi ukuran request payload maksimal 2MB (`413 Payload Too Large`).

Contoh kode transaksi:
```rust
use serde::{Deserialize, Serialize};
use wednes_sdk::{export, Guest, Request, Response};

#[derive(Deserialize)]
struct TransactionRequest {
    tx_id: String,
    account_id: String,
    amount: f64,
}

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        // 1. Validasi transaksi
        let tx = match req.json::<TransactionRequest>() {
            Ok(data) => data,
            Err(_) => return Response::json(400, r#"{"error":"Invalid transaction data"}"#),
        };

        // 2. Transaksi diproses di guest sandbox terisolasi
        if tx.amount <= 0.0 {
            return Response::json(422, r#"{"error":"Amount must be positive"}"#);
        }

        // 3. Kembalikan response transaksi sukses
        let payload = format!(
            r#"{{"status":"PROCESSED","tx_id":"{}","account":"{}"}}"#,
            tx.tx_id, tx.account_id
        );
        Response::json(200, payload)
    }
}

export!(Component);
```

---

## 3. Production Deployment & Daemon Run

### 1. Build Biner & Komponen
```bash
# Build engine CLI
cargo build --release -p wednes-cli

# Build WASM & adaptasi ke Component Model
cargo build --release --target wasm32-wasip1 -p payment-processor
wasm-tools component new target/wasm32-wasip1/release/payment_processor.wasm \
  --adapt target/wasi_snapshot_preview1.reactor.wasm \
  -o target/payment_processor.wasm
```

### 2. Deploy ke Registry
```bash
./target/release/wednesd deploy \
  --manifest manifest.toml \
  --registry registry.json
```

### 3. Jalankan Gateway Daemon
```bash
./target/release/wednesd run \
  --listen 0.0.0.0:8080 \
  --registry registry.json \
  --global-concurrency 500 \
  --memory-budget-mb 1200 \
  --log-json
```

Parameter Produksi:
- `--global-concurrency`: Batas concurrent request aktif sebelum `429 Too Many Requests`.
- `--memory-budget-mb`: Batas akumulasi memori guest agar tidak melebihi RAM fisik.
- `--log-json`: Mengaktifkan log terstruktur JSON untuk observabilitas.

---

## 4. Benchmark Hasil Eksekusi Live (KVM 2 GB RAM)

Pengujian performa nyata dijalankan di KVM Ubuntu 24.04 (2 vCPU, RAM 2 GB, swap disabled):

| Metrik | Hasil Pengukuran | Target PRD V0 | Status |
| :--- | :--- | :--- | :--- |
| **Idle RSS wednesd** (10 fungsi aktif) | **20.33 MiB** | <= 150 MiB | **Memenuhi Target** |
| **Peak RSS** (saat stress test) | **51.29 MiB** | <= 500 MiB | **Memenuhi Target** |
| **Engine Startup Time** (p95) | **714.1 ms** | < 1000 ms | **Memenuhi Target** |
| **Cold Start Invocation** (p95) | **5.66 ms** | < 10 ms | **Memenuhi Target** |
| **Warm Start Invocation** (p95) | **1.47 ms** | < 2 ms | **Memenuhi Target** |
| **Concurrent Requests Handled** | **16+ concurrent** | >= 16 | **Stabil** |
| **Crash Count Host** | **0 crash** | 0 crash | **100% Terisolasi** |
| **Swap Usage** | **0 KiB** | 0 KiB | **Sesuai Target** |

### Keamanan & Ketahanan Guest:
- **Infinite Loop:** Eksekusi diputus paksa oleh Wasmtime epoch timer setelah batas timeout tercapai (`HTTP 500`). Daemon tetap hidup.
- **Memory OOM:** Alokasi liar di sandbox WASM diblokir oleh `StoreLimits` (`HTTP 500`). Host aman.
- **WASM Rusak / Malformed:** Ditolak pada tahap validasi registri, tidak mengganggu fungsi yang sedang berjalan.

---

## 5. Observabilitas & Endpoint Metrics

Daemon mengekspos metrik runtime berformat Prometheus di `/metrics`:
```bash
curl http://127.0.0.1:8080/metrics
```

Contoh output:
```text
# HELP wednes_invocations_total Total number of function invocations
# TYPE wednes_invocations_total counter
wednes_invocations_total 12480

# HELP wednes_active_invocations Number of currently running invocations
# TYPE wednes_active_invocations gauge
wednes_active_invocations 4

# HELP wednes_invocation_traps_total Total guest traps/panics
# TYPE wednes_invocation_traps_total counter
wednes_invocation_traps_total 2

# HELP wednes_invocation_rejected_total Invocations rejected due to admission limits
# TYPE wednes_invocation_rejected_total counter
wednes_invocation_rejected_total 14
```

---

## 6. Multi-Language CLI Builder (AWS Lambda / Azure Functions Style)

CLI menyediakan layer abstraksi build untuk mengompilasi kode sumber tingkat tinggi (Python, TypeScript, Go) langsung menjadi biner WebAssembly Component Model tanpa konfigurasi manual Wasmtime.

### Alur Kerja Kompilasi Otomatis
```text
User Code (.py, .ts, .go) ──> wednesd CLI Builder ──> Wasm Component (.wasm) ──> wednesd Runtime
```

### A. Kasus 1: Python Function (Serverless Data Processing)
Fungsi Python menerima event request, memproses JSON, dan membaca environment variable.

1. **File Sumber (`app.py`):**
```python
import json
import os

def handle(request):
    env_mode = os.environ.get("ENV", "production")
    body = json.loads(request.get("body", "{}"))
    user = body.get("user", "Anonymous")
    
    return {
        "status": 200,
        "headers": [["content-type", "application/json"]],
        "body": json.dumps({"message": f"Hello {user}!", "env": env_mode})
    }
```

2. **Kompilasi via CLI:**
```bash
# Otomatis menggunakan componentize-py di latar belakang
wednesd build --entry app.py --lang python -o target/app.wasm
```

---

### B. Kasus 2: TypeScript Webhook Handler
Menangani verifikasi payload dan webhook eksternal.

1. **File Sumber (`index.ts`):**
```typescript
interface HttpRequest {
  method: string;
  path: string;
  headers: [string, string][];
  body: Uint8Array;
}

export function handle(req: HttpRequest) {
  const decoder = new TextDecoder();
  const payload = JSON.parse(decoder.decode(req.body));

  if (!payload.event) {
    return { status: 400, headers: [], body: new TextEncoder().encode("Missing event") };
  }

  return {
    status: 200,
    headers: [["content-type", "application/json"]],
    body: new TextEncoder().encode(JSON.stringify({ received: true, id: payload.id })),
  };
}
```

2. **Kompilasi via CLI:**
```bash
# Otomatis menggunakan componentize-js & jco
wednesd build --entry index.ts --lang ts -o target/webhook.wasm
```

---

### C. Kasus 3: Go High-Performance Microservice
Layanan pemrosesan transaksi cepat menggunakan TinyGo.

1. **File Sumber (`main.go`):**
```go
package main

import (
	"encoding/json"
)

type Request struct {
	Body []byte `json:"body"`
}

type Response struct {
	Status  uint16            `json:"status"`
	Headers map[string]string `json:"headers"`
	Body    string            `json:"body"`
}

func Handle(req Request) Response {
	var data map[string]interface{}
	json.Unmarshal(req.Body, &data)

	res := map[string]string{"status": "success", "engine": "wednes"}
	out, _ := json.Marshal(res)

	return Response{
		Status:  200,
		Headers: map[string]string{"content-type": "application/json"},
		Body:    string(out),
	}
}

func main() {}
```

2. **Kompilasi via CLI:**
```bash
# Otomatis memanggil tinygo dengan target wasip1
wednesd build --entry main.go --lang go -o target/service.wasm
```

---

### Meneruskan Argumen & Environment Variables
Konfigurasi diteruskan ke fungsi melalui `manifest.toml` (serupa konfigurasi AWS Lambda console):

```toml
name = "my-service"
abi = "wednes:function@0.1.0"
artifact = "target/service.wasm"

[runtime]
memory_mb = 64
timeout_ms = 5000

# Meneruskan environment variables ke guest sandbox
[environment]
DB_HOST = "10.0.0.5"
API_KEY = "secret-token"

[capabilities]
logging = true
clock = true
outbound_http = true
```

Deploy langsung ke runtime lokal atau server:
```bash
wednesd deploy --manifest manifest.toml --registry /tmp/registry.json
```

