# Zero-Knowledge Encrypted Artifacts

In multi-tenant serverless and distributed edge computing, hosting proprietary code and algorithms on third-party cloud infrastructure exposes developers to intellectual property leakage, reverse-engineering, and unauthorized host inspection.

**Wednes Serverless Engine** adopts the **Zero-Knowledge Encrypted Artifact** paradigm: application code and compiled WebAssembly binaries remain encrypted throughout transit and storage, only being decrypted inside hardened, ephemeral memory boundaries during authorized execution.

---

## The Core Problem

Traditional cloud functions upload raw JavaScript bundles, container images, or unencrypted binaries to cloud providers:
1. **Host-Level Exposure:** Cloud hypervisor operators, compromised nodes, or adjacent containers can inspect memory dumps and filesystem layers.
2. **Reverse Engineering:** Proprietary trading logic, private API schemas, AI inference graphs, and confidential heuristics can be disassembled.
3. **Tampering:** Malicious actors or compromised CI/CD runners can inject backdoors or alter bytecode without detection.

---

## Architectural Pillars

```
+───────────────────────────+
|   Developer Workstation   |
|   (Code + Private Keys)   |
+─────────────┬─────────────+
              │
              │ 1. Compile to WASM Component
              │ 2. Sign with Ed25519 Key
              │ 3. Encrypt payload (AES-256-GCM)
              ▼
+───────────────────────────+
|   Encrypted Cloud Vault   |
| (Zero-Knowledge Storage)  |
|  - Restic AES-256 at Rest |
|  - Cloudflare R2 / S3     |
|  - Zero Egress Recovery   |
+─────────────┬─────────────+
              │
              │ 4. Pull Encrypted Artifact (Zero-Egress)
              ▼
+───────────────────────────+
|     wednesd Host Node     |
|  - Ephemeral Decryption   |
|  - In-Memory Verification |
|  - Linear Memory Sandbox  |
+───────────────────────────+
```

### 1. Cryptographic Signature & Integrity
Every artifact is signed at build time using asymmetric cryptographic keypairs (such as Ed25519). Before the `wednesd` daemon accepts an artifact during deployment:
- It checks cryptographic signatures against trusted public keys.
- It verifies cryptographic hashes (SHA-256) to ensure the artifact has not been modified.
- It validates the binary structure strictly against the `wednes:function@0.1.0` WIT contract.

### 2. Encryption at Rest & Offsite Cloud Vault
Artifacts stored in local registries or synchronized offsite use authenticated encryption:
- **AES-256-GCM / ChaCha20-Poly1305:** Artifact contents are encrypted at rest.
- **Wednes Cloud Vault:** Disaster recovery and multi-region deployment integrate with automated offsite vaults (via Restic with AES-256 encryption streaming to S3/Cloudflare R2 buckets with zero egress fees).

### 3. Ephemeral In-Memory Execution
When `wednesd` loads a zero-knowledge artifact:
- The binary is decrypted directly into memory buffers; decrypted bytecode is never written to disk in unencrypted format.
- The Cranelift JIT engine compiles the component into native machine code held exclusively in protected executable memory pages.
- Memory isolation prevents adjacent tenant processes or guest components from reading raw bytecode.

---

## Security Guarantees & Boundaries

| Guarantee | Description |
|:---|:---|
| **Confidentiality at Rest** | Stored `.wasm` binaries cannot be inspected without the decryption key. |
| **Tamper Detection** | Any modification to ciphertext causes immediate authentication tag verification failure. |
| **No Ambient Host Leakage** | Guest instances cannot access host environment variables, encryption keys, or peer memory buffers. |
| **Zero-Egress Vault Sync** | Synchronizing snapshots across regions or rebuilding failed nodes incurs $0 egress fee using zero-egress vaults. |

---

## Best Practices for Operators

1. **Keep Private Keys Offline:** Never bake deployment signing keys or master decryption keys into the guest function bytecode.
2. **Use Ephemeral Disks for Node Registries:** Configure `registry.json` and artifact directories on encrypted filesystems (e.g., LUKS / dm-crypt or RAM-backed `tmpfs`).
3. **Rotate Operator Keys Regularly:** Maintain a strict key rotation policy for CI/CD signing keys and registry tokens.
