pub mod artifact;
pub mod cli;
pub mod error;
pub mod hook;

pub use artifact::{
    CipherAlgorithm, EncryptedArtifact, CURRENT_VERSION, HEADER_LEN, MAGIC, VERSION_AES_256_GCM,
    VERSION_CHACHA20_POLY1305,
};
pub use cli::{run_cli, Cli, Commands};
pub use error::{CryptoError, Result};
pub use hook::{
    generate_key, load_encrypted_wasm_file_in_memory, load_encrypted_wasm_in_memory, parse_key,
    verify_wasm_header, WASM_MAGIC,
};
