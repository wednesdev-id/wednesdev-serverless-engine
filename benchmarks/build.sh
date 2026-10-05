#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
rustup target add wasm32-wasip1
command -v wasm-tools >/dev/null || cargo install wasm-tools --locked
curl -fL https://github.com/bytecodealliance/wasmtime/releases/download/v25.0.3/wasi_snapshot_preview1.reactor.wasm -o target/adapter.wasm
cargo build --release --target wasm32-wasip1 -p hello-world -p json-api -p webhook -p infinite-loop -p oom -p workload
mkdir -p target/artifacts
for name in hello_world json_api webhook infinite_loop oom workload; do
 out="$name"; test "$name" != hello_world || out=hello
 wasm-tools component new "target/wasm32-wasip1/release/$name.wasm" --adapt wasi_snapshot_preview1=target/adapter.wasm -o "target/artifacts/$out.wasm"
done
cp target/artifacts/{hello,infinite_loop,oom}.wasm target/
cargo build --release -p wednes-cli
