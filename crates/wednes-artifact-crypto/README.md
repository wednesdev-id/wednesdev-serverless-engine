# wednes-artifact-crypto

Zero-Knowledge AES-256-GCM (and ChaCha20-Poly1305) encrypted WebAssembly artifact module with in-memory execution hooks for the Wednes Serverless Engine.

## Overview

In multi-tenant serverless environments and confidential compute clusters, guest WebAssembly binaries contain sensitive proprietary business logic, algorithms, and intellectual property. Storing raw `.wasm` files on host disks or node filesystems exposes them to node operators, unauthorized processes, or compromised host storage.

`wednes-artifact-crypto` provides:
1. **Authenticated Encryption Envelope (`.wenc`)**:
   - Magic Header: `WENC` (4 bytes: `0x57, 0x45, 0x4e, 0x43`)
   - Version: `0x01` for AES-256-GCM, `0x02` for ChaCha20-Poly1305 (1 byte)
   - Cryptographically random Nonce/IV (12 bytes)
   - Polyval/Poly1305 Authentication Tag (16 bytes)
   - Ciphertext payload (`N` bytes)
2. **Zero-Knowledge In-Memory Execution Hook**:
   - Decrypts `.wenc` payload strictly in RAM.
   - Validates WebAssembly binary magic header `\0asm` (`0x00, 0x61, 0x73, 0x6d`).
   - Direct component compilation via Wasmtime `Component::from_binary`.
   - Automatic zeroization of sensitive key material and decrypted buffers using `zeroize`.
   - Never writes decrypted plaintext wasm bytes to disk or swap.

---

## Wire Format Specification

```
+----------------+---------------+-------------------+------------------+-----------------------+
|  Magic (4B)    |  Version (1B) |   Nonce/IV (12B)  |     Tag (16B)    |    Ciphertext (NB)    |
|  "WENC"        |  0x01 / 0x02  |   Random 96-bit   |   128-bit Auth   |    Encrypted Wasm     |
+----------------+---------------+-------------------+------------------+-----------------------+
0                4               5                   17                 33                      33+N
```

Header length: 33 bytes. Minimum valid `.wenc` payload size is 33 bytes.

---

## CLI Usage

The crate compiles two binary aliases: `wednes` and `wednes-artifact-crypto`.

### 1. Generate 256-bit Key
```bash
KEY=$(cargo run --bin wednes -- gen-key)
echo "Generated key: $KEY"
```

### 2. Pack and Encrypt Wasm Artifact
```bash
cargo run --bin wednes -- pack \
  --encrypt \
  --key "$KEY" \
  --input path/to/module.wasm \
  --output path/to/module.wenc \
  --cipher aes-256-gcm
```

### 3. In-Memory Runtime Verification Hook
```bash
cargo run --bin wednes -- verify \
  --key "$KEY" \
  --input path/to/module.wenc
```

Output:
```
Executing in-memory decryption hook for runtime wednesd on "path/to/module.wenc"
Success: Artifact decrypted in RAM only (15 bytes). Verified '\0asm' magic header. No plaintext written to disk.
```

---

## Rust Library API

```rust
use wednes_artifact_crypto::{
    EncryptedArtifact, CipherAlgorithm, load_encrypted_wasm_in_memory, parse_key
};

// 1. Encrypt wasm artifact
let key = [0x42u8; 32];
let wasm_bytes = b"\0asm\x01\x00\x00\x00...";
let encrypted_payload = EncryptedArtifact::encrypt(wasm_bytes, &key)?;

// 2. In-memory decryption hook for runtime wednesd
let zeroizing_wasm = load_encrypted_wasm_in_memory(&encrypted_payload, &key)?;
assert_eq!(&zeroizing_wasm[..4], b"\0asm");

// 3. Compile directly with Wasmtime (in RAM only)
// let component = wasmtime::component::Component::from_binary(&engine, &zeroizing_wasm)?;
```

---

## Running Tests

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```
