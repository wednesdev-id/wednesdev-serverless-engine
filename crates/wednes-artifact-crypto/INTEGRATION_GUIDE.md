# Upstream Integration Guide: Zero-Knowledge Encrypted Wasm Artifacts

This guide provides concrete, step-by-step instructions and git patch diffs to integrate `wednes-artifact-crypto` into the upstream repository `https://github.com/wednesdev-id/wednesdev-serverless-engine`.

---

## 1. Workspace Cargo Configuration

In root `Cargo.toml`:

```toml
[workspace]
members = [
    "crates/wednes-artifact-crypto",
    "crates/wednes-core",
    "crates/wednes-telemetry",
    "crates/wednes-capabilities",
    "crates/wednes-runtime",
    "crates/wednes-runtime-wasmtime",
    "crates/wednes-gateway",
    "crates/wednes-cli",
    ...
]

[workspace.dependencies]
wednes-artifact-crypto = { path = "crates/wednes-artifact-crypto" }
```

Copy or symlink `wednesdev-serverless-crypto` to `crates/wednes-artifact-crypto` within the engine workspace.

---

## 2. Integration into `crates/wednes-core`

### 2.1 Update `crates/wednes-core/Cargo.toml`
Add dependency:
```toml
[dependencies]
wednes-artifact-crypto = { workspace = true, optional = true }
```

### 2.2 Update `crates/wednes-core/src/manifest.rs`
Allow `.wenc` extensions and optional encryption key configuration in manifest:

```diff
--- a/crates/wednes-core/src/manifest.rs
+++ b/crates/wednes-core/src/manifest.rs
@@ -14,6 +14,8 @@ pub struct Manifest {
     pub runtime: Option<RuntimeConfig>,
     #[serde(default)]
     pub capabilities: Option<Capabilities>,
+    #[serde(default)]
+    pub encryption: Option<EncryptionConfig>,
 }

+#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
+#[serde(deny_unknown_fields)]
+pub struct EncryptionConfig {
+    pub enabled: bool,
+    #[serde(default)]
+    pub key_env: Option<String>,
+}
+
 impl Manifest {
     pub fn validate(&self) -> Result<()> {
         ...
```

Artifact validation check accepts both `.wasm` and `.wenc` extensions without path traversal.

---

## 3. Integration into `crates/wednes-runtime-wasmtime`

### 3.1 Update `crates/wednes-runtime-wasmtime/Cargo.toml`
Add:
```toml
[dependencies]
wednes-artifact-crypto = { workspace = true }
```

### 3.2 Update `crates/wednes-runtime-wasmtime/src/lib.rs`
Hook in-memory decryption directly before compiling components in `load_module`, `load_manifest`, and `precompile`:

```rust
use wednes_artifact_crypto::{load_encrypted_wasm_in_memory, parse_key, MAGIC as WENC_MAGIC};

impl WasmtimeBackend {
    /// Load component either from raw wasm or encrypted .wenc payload in RAM only
    fn load_component_bytes(&self, id: &str, raw_bytes: &[u8]) -> Result<Component> {
        if raw_bytes.starts_with(&WENC_MAGIC) {
            // Retrieve key from environment variable
            let env_var = format!("WEDNES_KEY_{}", id.to_uppercase().replace('-', "_"));
            let key_str = std::env::var("WEDNES_ARTIFACT_KEY")
                .or_else(|_| std::env::var(&env_var))
                .map_err(|_| anyhow::anyhow!(
                    "Artifact for function '{}' is encrypted (WENC), but neither WEDNES_ARTIFACT_KEY nor {} is set",
                    id, env_var
                ))?;

            let key = parse_key(&key_str)?;
            // Decrypt in RAM only, verifying \0asm header; plaintext zeroizes on drop
            let decrypted_wasm = load_encrypted_wasm_in_memory(raw_bytes, &key)?;
            Ok(Component::from_binary(&self.engine, &decrypted_wasm)?)
        } else {
            Ok(Component::from_binary(&self.engine, raw_bytes)?)
        }
    }
}
```

Update `load_module`, `load_manifest`, and `precompile`:

```diff
--- a/crates/wednes-runtime-wasmtime/src/lib.rs
+++ b/crates/wednes-runtime-wasmtime/src/lib.rs
@@ -133,18 +133,20 @@ impl RuntimeBackend for WasmtimeBackend {
     fn load_module(&mut self, id: &str, wasm_path: &Path) -> Result<()> {
-        let component = Component::from_file(&self.engine, wasm_path)?;
+        let raw_bytes = std::fs::read(wasm_path)?;
+        let component = self.load_component_bytes(id, &raw_bytes)?;
         FunctionPre::new(self.linker.instantiate_pre(&component)?)?;
         self.components.insert(id.to_string(), (component, None));
         Ok(())
     }

     fn load_manifest(
         &mut self,
         manifest: &wednes_core::manifest::Manifest,
         wasm_path: &Path,
     ) -> Result<()> {
         manifest.validate()?;
-        let component = Component::from_file(&self.engine, wasm_path)?;
+        let raw_bytes = std::fs::read(wasm_path)?;
+        let component = self.load_component_bytes(&manifest.name, &raw_bytes)?;
         FunctionPre::new(self.linker.instantiate_pre(&component)?)?;
         self.components
             .insert(manifest.name.clone(), (component, Some(manifest.clone())));
         Ok(())
     }
```

---

## 4. Integration into `crates/wednes-cli`

### 4.1 Update `crates/wednes-cli/src/main.rs`
Add `Pack` subcommand to CLI:

```rust
#[derive(Subcommand, Debug)]
enum Commands {
    Pack {
        #[arg(long, default_value_t = true)]
        encrypt: bool,
        #[arg(short, long)]
        key: String,
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "aes-256-gcm")]
        cipher: String,
    },
    ...
}
```

Execution dispatch:
```rust
Some(Commands::Pack { encrypt: _, key, input, output, cipher }) => {
    let key_bytes = wednes_artifact_crypto::parse_key(&key)?;
    let wasm_bytes = std::fs::read(&input)?;
    let algo = match cipher.as_str() {
        "chacha20-poly1305" => wednes_artifact_crypto::CipherAlgorithm::ChaCha20Poly1305,
        _ => wednes_artifact_crypto::CipherAlgorithm::Aes256Gcm,
    };
    let encrypted = wednes_artifact_crypto::EncryptedArtifact::encrypt_with_cipher(
        &wasm_bytes,
        &key_bytes,
        algo,
    )?;
    std::fs::write(&output, &encrypted)?;
    tracing::info!("Packed encrypted artifact {:?}", output);
    Ok(())
}
```

---

## 5. Security & Zeroization Verification

1. **Host Disk Confidentiality**:
   Plaintext WebAssembly binaries are never written to disk, tempfiles, or swap during decryption.
2. **Buffer Zeroization**:
   Decrypted buffers wrap in `zeroize::Zeroizing` and call `plaintext.zeroize()` on authentication error, wiping residual memory before deallocation.
3. **Format Integrity**:
   Tampered ciphertexts, mismatched nonces, corrupted tags, or altered version headers reject immediately before Wasmtime compilation.
