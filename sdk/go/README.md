# Wednes Go SDK

Go SDK for Wednes Engine using WebAssembly.

## 1. Generate Bindings

Install `wit-bindgen` CLI:

```bash
cargo install wit-bindgen-cli
```

Generate bindings from WIT definition:

```bash
wit-bindgen go --out-dir gen ../../wit
```

## 2. Build with TinyGo

Build targeting WASI P1:

```bash
tinygo build -target=wasip1 -o function.wasm main.go
```
