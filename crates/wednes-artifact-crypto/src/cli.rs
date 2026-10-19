use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::artifact::{CipherAlgorithm, EncryptedArtifact};
use crate::hook::{generate_key, load_encrypted_wasm_file_in_memory, parse_key};

#[derive(Parser, Debug)]
#[command(
    name = "wednes",
    about = "Wednes Serverless Engine - Zero-Knowledge Encrypted Artifact CLI"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Pack and encrypt Wasm artifact into .wenc envelope
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
    /// In-memory decryption hook simulating wednesd runtime: loads .wenc, decrypts in RAM, verifies \0asm
    Verify {
        #[arg(short, long)]
        key: String,

        #[arg(short, long)]
        input: PathBuf,
    },
    /// Generate a cryptographically secure 256-bit encryption key (hex)
    GenKey,
}

pub fn run_cli() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pack {
            encrypt,
            key,
            input,
            output,
            cipher,
        } => {
            if !encrypt {
                eprintln!("Error: pack command requires --encrypt");
                return ExitCode::FAILURE;
            }

            let key_bytes = match parse_key(&key) {
                Ok(k) => k,
                Err(e) => {
                    eprintln!("Error parsing key: {e}");
                    return ExitCode::FAILURE;
                }
            };

            let algo = match cipher.to_lowercase().as_str() {
                "aes-256-gcm" | "aes" => CipherAlgorithm::Aes256Gcm,
                "chacha20-poly1305" | "chacha" => CipherAlgorithm::ChaCha20Poly1305,
                other => {
                    eprintln!("Error: unsupported cipher '{other}'. Supported: aes-256-gcm, chacha20-poly1305");
                    return ExitCode::FAILURE;
                }
            };

            let wasm_bytes = match fs::read(&input) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Error reading input file {:?}: {e}", input);
                    return ExitCode::FAILURE;
                }
            };

            println!(
                "Packaging artifact: {:?} ({} bytes) with algorithm {:?}",
                input,
                wasm_bytes.len(),
                algo
            );

            let encrypted =
                match EncryptedArtifact::encrypt_with_cipher(&wasm_bytes, &key_bytes, algo) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        eprintln!("Encryption failed: {e}");
                        return ExitCode::FAILURE;
                    }
                };

            if let Some(parent) = output.parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = fs::create_dir_all(parent);
                }
            }

            if let Err(e) = fs::write(&output, &encrypted) {
                eprintln!("Error writing encrypted artifact to {:?}: {e}", output);
                return ExitCode::FAILURE;
            }

            println!(
                "Successfully generated encrypted artifact: {:?} ({} bytes, header {} bytes)",
                output,
                encrypted.len(),
                EncryptedArtifact::HEADER_LEN
            );
            ExitCode::SUCCESS
        }
        Commands::Verify { key, input } => {
            let key_bytes = match parse_key(&key) {
                Ok(k) => k,
                Err(e) => {
                    eprintln!("Error parsing key: {e}");
                    return ExitCode::FAILURE;
                }
            };

            println!(
                "Executing in-memory decryption hook for runtime wednesd on {:?}",
                input
            );

            match load_encrypted_wasm_file_in_memory(&input, &key_bytes) {
                Ok(plaintext) => {
                    println!(
                        "Success: Artifact decrypted in RAM only ({} bytes). Verified '\\0asm' magic header. No plaintext written to disk.",
                        plaintext.len()
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("In-memory verification failed: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Commands::GenKey => {
            let key = generate_key();
            let hex_key = hex::encode(*key);
            println!("{hex_key}");
            ExitCode::SUCCESS
        }
    }
}
