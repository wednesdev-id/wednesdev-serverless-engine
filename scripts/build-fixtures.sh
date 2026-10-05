#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
command -v wasm-tools >/dev/null || { printf '%s\n' 'Install wasm-tools: cargo install wasm-tools --locked' >&2; exit 1; }
mkdir -p target
curl --fail --location --retry 3 https://github.com/bytecodealliance/wasmtime/releases/download/v25.0.0/wasi_snapshot_preview1.reactor.wasm -o target/wasi_snapshot_preview1.reactor.wasm
printf '%s\n' '240c28b2561e2d32daef30112bb2aaeb6427976de731401b806d7e32a1a27326  target/wasi_snapshot_preview1.reactor.wasm' | sha256sum --check
cargo build --release --target wasm32-wasip1 -p hello-world -p infinite-loop -p oom -p json-api -p webhook
for name in hello_world infinite_loop oom json_api webhook; do
    output="$name"
    if [[ "$name" == hello_world ]]; then output=hello; fi
    wasm-tools component new "target/wasm32-wasip1/release/$name.wasm" --adapt wasi_snapshot_preview1=target/wasi_snapshot_preview1.reactor.wasm -o "target/$output.wasm"
    wasm-tools validate "target/$output.wasm"
done
