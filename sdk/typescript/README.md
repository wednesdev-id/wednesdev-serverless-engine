# Wednes Engine TypeScript SDK

TypeScript SDK for creating Wednes Engine components.

## Build Process

1. Install dependencies:
   ```bash
   npm install
   ```

2. Compile TypeScript to JavaScript:
   ```bash
   npx tsc src/index.ts
   ```

3. Componentize the JavaScript file using `jco componentize`:
   ```bash
   npx jco componentize src/index.js --wit ../../wit/function.wit -n function -o hello.wasm
   ```

The resulting `hello.wasm` can be run in the Wednes Engine.
