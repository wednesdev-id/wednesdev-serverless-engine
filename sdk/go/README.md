# Wednes Go SDK

Go SDK for Wednes Engine using WebAssembly.

## 1. Generate Bindings

Install `wit-bindgen` CLI:

```bash
cargo install wit-bindgen-cli
```

Generate bindings from WIT definition:

```bash
wit-bindgen go --out-dir bindings ../../wit
```

## 2. Build with TinyGo

Download and extract TinyGo:

```bash
wget https://github.com/tinygo-org/tinygo/releases/download/v0.31.2/tinygo0.31.2.linux-amd64.tar.gz -O /tmp/tinygo.tar.gz
tar -xzf /tmp/tinygo.tar.gz -C /tmp
```

Write `main.go` importing `bindings` and exporting `Handle`:

```go
package main

import (
	bindings "wednes-sdk-go/bindings"
)

func init() {
	bindings.SetFunction(impl{})
}

type impl struct{}

func (i impl) Handle(req bindings.FunctionRequest) bindings.FunctionResponse {
	return bindings.FunctionResponse{
		Status: 200,
		Body:   []byte("Hello from TinyGo WASM!"),
	}
}

func main() {}
```

Build targeting WASI:

```bash
/tmp/tinygo/bin/tinygo build -target=wasi -o main.wasm main.go
```

## 3. Package as Component

Create a WASM component:

```bash
wasm-tools component new main.wasm -o app.wasm
# Note: In practice, --adapt /path/to/wasi_snapshot_preview1.reactor.wasm is required.
```
