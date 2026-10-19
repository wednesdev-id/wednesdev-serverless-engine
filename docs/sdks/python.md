# Python SDK

The **Wednes Python SDK** enables writing clean, idiomatic Python 3 functions compiled directly to WebAssembly components via [componentize-py](https://github.com/bytecodealliance/componentize-py).

---

## Handler Specification

In Python, the function logic is implemented via a `WitWorld` class with a `handle` method:

```python
"""Wednes Python SDK function handler."""
from dataclasses import dataclass
from typing import List

@dataclass
class Header:
    name: str
    value: str

@dataclass
class Request:
    method: str
    path: str
    headers: List[Header]
    body: bytes

@dataclass
class Response:
    status: int
    headers: List[Header]
    body: bytes

class WitWorld:
    def handle(self, req: Request) -> Response:
        return Response(
            status=200,
            headers=[Header(name="content-type", value="text/plain")],
            body=b"Hello from Wednes Python SDK!\n",
        )
```

---

## Examples

### 1. JSON System Status & Telemetry

```python
import json
import time
from dataclasses import dataclass
from typing import List

@dataclass
class Header:
    name: str
    value: str

@dataclass
class Request:
    method: str
    path: str
    headers: List[Header]
    body: bytes

@dataclass
class Response:
    status: int
    headers: List[Header]
    body: bytes

class WitWorld:
    def handle(self, req: Request) -> Response:
        payload = {
            "status": "healthy",
            "runtime": "Wednes Engine (Python/Wasmtime)",
            "path": req.path,
            "timestamp": int(time.time()),
        }

        body_bytes = json.dumps(payload).encode("utf-8")

        return Response(
            status=200,
            headers=[Header(name="content-type", value="application/json")],
            body=body_bytes,
        )
```

---

### 2. Payload Validation & Lead Scoring

```python
import json
from dataclasses import dataclass
from typing import List

@dataclass
class Header:
    name: str
    value: str

@dataclass
class Request:
    method: str
    path: str
    headers: List[Header]
    body: bytes

@dataclass
class Response:
    status: int
    headers: List[Header]
    body: bytes

class WitWorld:
    def handle(self, req: Request) -> Response:
        if req.method != "POST":
            return Response(
                status=405,
                headers=[Header(name="content-type", value="application/json")],
                body=b'{"error": "Method Not Allowed"}',
            )

        try:
            data = json.loads(req.body.decode("utf-8"))
            company_size = int(data.get("company_size", 1))
            budget = float(data.get("budget", 0))

            # Compute Lead Score
            score = 10
            if company_size > 50:
                score += 30
            if budget >= 10000:
                score += 50

            result = {
                "lead": data.get("email", "unknown"),
                "score": score,
                "tier": "enterprise" if score >= 80 else "standard",
            }

            return Response(
                status=200,
                headers=[Header(name="content-type", value="application/json")],
                body=json.dumps(result).encode("utf-8"),
            )
        except Exception as e:
            return Response(
                status=400,
                headers=[Header(name="content-type", value="application/json")],
                body=json.dumps({"error": str(e)}).encode("utf-8"),
            )
```

---

## Compiling Python to Wasm Component

Install `componentize-py` and compile your `app.py`:

```bash
pip install componentize-py

componentize-py \
  -w ../../wit \
  -d . \
  -m app \
  componentize function.wasm
```

The emitted `function.wasm` includes an embedded minimal CPython runtime and bytecode, executing within isolated Wasmtime sandboxes.
