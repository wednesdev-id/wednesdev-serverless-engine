use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CryptoError {
    #[error("payload too short: expected at least {expected} bytes, got {actual}")]
    PayloadTooShort { expected: usize, actual: usize },

    #[error("invalid magic bytes: expected WENC, got {0:02x?}")]
    InvalidMagic([u8; 4]),

    #[error("unsupported version: expected 0x{expected:02x}, got 0x{actual:02x}")]
    UnsupportedVersion { expected: u8, actual: u8 },

    #[error("authentication failed: corrupted ciphertext, invalid key, or auth tag mismatch")]
    AuthenticationFailed,

    #[error("invalid wasm binary: missing '\\0asm' magic header")]
    InvalidWasmHeader,

    #[error("invalid key length: expected {expected} bytes, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("hex decoding error: {0}")]
    HexDecodeError(String),

    #[error("io error: {0}")]
    IoError(String),

    #[error("internal crypto error: {0}")]
    InternalError(String),
}

impl From<std::io::Error> for CryptoError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CryptoError>;
