use aes_gcm::{
    aead::{AeadInPlace, KeyInit},
    Aes256Gcm, Key as AesKey, Nonce as AesNonce, Tag as AesTag,
};
use chacha20poly1305::{
    ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce, Tag as ChaChaTag,
};
use zeroize::{Zeroize, Zeroizing};

use crate::error::{CryptoError, Result};

pub const MAGIC: [u8; 4] = *b"WENC";
pub const VERSION_AES_256_GCM: u8 = 0x01;
pub const VERSION_CHACHA20_POLY1305: u8 = 0x02;
pub const CURRENT_VERSION: u8 = VERSION_AES_256_GCM;
pub const HEADER_LEN: usize = 4 + 1 + 12 + 16; // 33 bytes

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CipherAlgorithm {
    Aes256Gcm = 0x01,
    ChaCha20Poly1305 = 0x02,
}

impl CipherAlgorithm {
    pub fn from_version(version: u8) -> Result<Self> {
        match version {
            VERSION_AES_256_GCM => Ok(Self::Aes256Gcm),
            VERSION_CHACHA20_POLY1305 => Ok(Self::ChaCha20Poly1305),
            other => Err(CryptoError::UnsupportedVersion {
                expected: CURRENT_VERSION,
                actual: other,
            }),
        }
    }

    pub fn version_byte(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedArtifact {
    pub version: u8,
    pub nonce: [u8; 12],
    pub tag: [u8; 16],
    pub ciphertext: Vec<u8>,
}

impl EncryptedArtifact {
    pub const MAGIC: [u8; 4] = MAGIC;
    pub const CURRENT_VERSION: u8 = CURRENT_VERSION;
    pub const HEADER_LEN: usize = HEADER_LEN;

    /// Parse and validate encrypted artifact binary format.
    /// Validates length >= 33 bytes, magic header b"WENC", and supported version byte.
    pub fn parse(payload: &[u8]) -> Result<Self> {
        if payload.len() < Self::HEADER_LEN {
            return Err(CryptoError::PayloadTooShort {
                expected: Self::HEADER_LEN,
                actual: payload.len(),
            });
        }

        let mut magic = [0u8; 4];
        magic.copy_from_slice(&payload[0..4]);
        if magic != Self::MAGIC {
            return Err(CryptoError::InvalidMagic(magic));
        }

        let version = payload[4];
        if version != VERSION_AES_256_GCM && version != VERSION_CHACHA20_POLY1305 {
            return Err(CryptoError::UnsupportedVersion {
                expected: CURRENT_VERSION,
                actual: version,
            });
        }

        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&payload[5..17]);

        let mut tag = [0u8; 16];
        tag.copy_from_slice(&payload[17..33]);

        let ciphertext = payload[33..].to_vec();

        Ok(Self {
            version,
            nonce,
            tag,
            ciphertext,
        })
    }

    /// Serialize encrypted artifact into binary wire format:
    /// [WENC (4)] [Version (1)] [Nonce (12)] [Tag (16)] [Ciphertext (N)]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::HEADER_LEN + self.ciphertext.len());
        out.extend_from_slice(&Self::MAGIC);
        out.push(self.version);
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&self.tag);
        out.extend_from_slice(&self.ciphertext);
        out
    }

    /// Encrypt plaintext Wasm bytes using AES-256-GCM (Version 0x01).
    pub fn encrypt(wasm_bytes: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
        Self::encrypt_with_cipher(wasm_bytes, key, CipherAlgorithm::Aes256Gcm)
    }

    /// Encrypt plaintext bytes using chosen cipher algorithm.
    pub fn encrypt_with_cipher(
        wasm_bytes: &[u8],
        key: &[u8; 32],
        algorithm: CipherAlgorithm,
    ) -> Result<Vec<u8>> {
        let zeroized_key = Zeroizing::new(*key);

        let mut nonce = [0u8; 12];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce);

        let mut ciphertext = wasm_bytes.to_vec();

        let tag_bytes = match algorithm {
            CipherAlgorithm::Aes256Gcm => {
                let cipher = Aes256Gcm::new(AesKey::<Aes256Gcm>::from_slice(zeroized_key.as_ref()));
                let tag = cipher
                    .encrypt_in_place_detached(AesNonce::from_slice(&nonce), &[], &mut ciphertext)
                    .map_err(|e| CryptoError::InternalError(e.to_string()))?;
                let mut tag_arr = [0u8; 16];
                tag_arr.copy_from_slice(tag.as_slice());
                tag_arr
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(zeroized_key.as_ref()));
                let tag = cipher
                    .encrypt_in_place_detached(
                        ChaChaNonce::from_slice(&nonce),
                        &[],
                        &mut ciphertext,
                    )
                    .map_err(|e| CryptoError::InternalError(e.to_string()))?;
                let mut tag_arr = [0u8; 16];
                tag_arr.copy_from_slice(tag.as_slice());
                tag_arr
            }
        };

        let artifact = Self {
            version: algorithm.version_byte(),
            nonce,
            tag: tag_bytes,
            ciphertext,
        };

        Ok(artifact.to_bytes())
    }

    /// Decrypt an encrypted WENC payload using the 32-byte key.
    /// Inspects version header:
    /// - 0x01: AES-256-GCM
    /// - 0x02: ChaCha20-Poly1305
    ///
    /// Zeroizes intermediate decrypted buffers on authentication failure.
    pub fn decrypt(encrypted_payload: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
        let artifact = Self::parse(encrypted_payload)?;
        artifact.decrypt_payload(key)
    }

    /// Decrypt the payload stored within this EncryptedArtifact instance.
    pub fn decrypt_payload(&self, key: &[u8; 32]) -> Result<Vec<u8>> {
        let zeroized_key = Zeroizing::new(*key);
        let algo = CipherAlgorithm::from_version(self.version)?;

        let mut plaintext = self.ciphertext.clone();

        let decrypt_res = match algo {
            CipherAlgorithm::Aes256Gcm => {
                let cipher = Aes256Gcm::new(AesKey::<Aes256Gcm>::from_slice(zeroized_key.as_ref()));
                let nonce = AesNonce::from_slice(&self.nonce);
                let tag = AesTag::from_slice(&self.tag);
                cipher.decrypt_in_place_detached(nonce, &[], &mut plaintext, tag)
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(zeroized_key.as_ref()));
                let nonce = ChaChaNonce::from_slice(&self.nonce);
                let tag = ChaChaTag::from_slice(&self.tag);
                cipher.decrypt_in_place_detached(nonce, &[], &mut plaintext, tag)
            }
        };

        match decrypt_res {
            Ok(()) => Ok(plaintext),
            Err(_) => {
                plaintext.zeroize();
                Err(CryptoError::AuthenticationFailed)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cipher_algorithm_version_mapping() {
        assert_eq!(
            CipherAlgorithm::from_version(0x01).unwrap(),
            CipherAlgorithm::Aes256Gcm
        );
        assert_eq!(
            CipherAlgorithm::from_version(0x02).unwrap(),
            CipherAlgorithm::ChaCha20Poly1305
        );
        assert!(matches!(
            CipherAlgorithm::from_version(0x03),
            Err(CryptoError::UnsupportedVersion { .. })
        ));
    }

    #[test]
    fn test_artifact_direct_methods() {
        let key = [0x1au8; 32];
        let wasm = b"\0asm\x01\x00\x00\x00unit_test";

        let enc = EncryptedArtifact::encrypt(wasm, &key).unwrap();
        let artifact = EncryptedArtifact::parse(&enc).unwrap();
        assert_eq!(artifact.version, VERSION_AES_256_GCM);

        let dec = artifact.decrypt_payload(&key).unwrap();
        assert_eq!(dec, wasm);
    }
}
