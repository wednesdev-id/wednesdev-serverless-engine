use std::path::Path;
use zeroize::Zeroizing;

use crate::artifact::EncryptedArtifact;
use crate::error::{CryptoError, Result};

pub const WASM_MAGIC: [u8; 4] = *b"\0asm";

/// Validate that decrypted bytes begin with WebAssembly binary magic header `\0asm` (0x00, 0x61, 0x73, 0x6d).
pub fn verify_wasm_header(wasm_bytes: &[u8]) -> Result<()> {
    if wasm_bytes.len() < 4 || wasm_bytes[0..4] != WASM_MAGIC {
        return Err(CryptoError::InvalidWasmHeader);
    }
    Ok(())
}

/// In-memory decryption hook for runtime `wednesd`.
/// Decrypts `.wenc` artifact entirely in RAM and verifies `\0asm` header.
/// Plaintext bytes are NEVER written to disk and are wrapped in `Zeroizing` buffer.
pub fn load_encrypted_wasm_in_memory(
    encrypted_payload: &[u8],
    key: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>> {
    let plaintext = EncryptedArtifact::decrypt(encrypted_payload, key)?;
    verify_wasm_header(&plaintext)?;
    Ok(Zeroizing::new(plaintext))
}

/// Helper to load an encrypted file from disk, decrypt in RAM only, and verify Wasm header.
pub fn load_encrypted_wasm_file_in_memory(
    path: &Path,
    key: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>> {
    let encrypted_bytes = std::fs::read(path)?;
    load_encrypted_wasm_in_memory(&encrypted_bytes, key)
}

/// Parse a 32-byte AES/ChaCha key from either 64-character hexadecimal or raw 32-byte string.
pub fn parse_key(key_str: &str) -> Result<Zeroizing<[u8; 32]>> {
    let trimmed = key_str.trim();

    if trimmed.len() == 64 {
        let decoded =
            hex::decode(trimmed).map_err(|e| CryptoError::HexDecodeError(e.to_string()))?;
        if decoded.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 32,
                actual: decoded.len(),
            });
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&decoded);
        return Ok(Zeroizing::new(key));
    }

    if trimmed.len() == 32 {
        let mut key = [0u8; 32];
        key.copy_from_slice(trimmed.as_bytes());
        return Ok(Zeroizing::new(key));
    }

    Err(CryptoError::InvalidKeyLength {
        expected: 32,
        actual: trimmed.len(),
    })
}

/// Generate a cryptographically secure 256-bit random key.
pub fn generate_key() -> Zeroizing<[u8; 32]> {
    let mut key = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut key);
    Zeroizing::new(key)
}
