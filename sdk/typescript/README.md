# Wednes Engine TypeScript SDK

TypeScript SDK for creating Wednes Engine components.

## Prerequisites

- Node.js (v18+)
- wasm-tools

## Build Process

1. Install dependencies:
   ```bash
   npm install
   ```

2. Compile TypeScript to JavaScript:
   ```bash
   npx tsc
   ```

3. Componentize the JavaScript file using `jco componentize`:
   ```bash
   npx jco componentize -w ../../wit dist/index.js -o app.wasm
   ```

4. Validate the WebAssembly component:
   ```bash
   wasm-tools validate app.wasm
   ```

The resulting `app.wasm` can be run in the Wednes Engine.
