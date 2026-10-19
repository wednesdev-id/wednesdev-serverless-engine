use std::fs;
use wednes_artifact_crypto::{
    generate_key, load_encrypted_wasm_file_in_memory, load_encrypted_wasm_in_memory, parse_key,
    verify_wasm_header, CryptoError, EncryptedArtifact, WASM_MAGIC,
};

#[test]
fn test_in_memory_hook_valid_wasm() {
    let key = [0x42u8; 32];
    let valid_wasm = b"\0asm\x01\x00\x00\x00\x00dummy_module_content";

    let encrypted = EncryptedArtifact::encrypt(valid_wasm, &key).unwrap();

    // Hook decrypts in RAM and returns Zeroizing buffer
    let decrypted = load_encrypted_wasm_in_memory(&encrypted, &key).expect("hook should succeed");
    assert_eq!(&decrypted[..4], &WASM_MAGIC);
    assert_eq!(decrypted.as_slice(), valid_wasm);
}

#[test]
fn test_in_memory_hook_non_wasm_fails_header_check() {
    let key = [0x55u8; 32];
    let non_wasm = b"ELF\x02\x01\x01\x00not_a_wasm_binary";

    let encrypted = EncryptedArtifact::encrypt(non_wasm, &key).unwrap();

    // Crypto succeeds, but Wasm header verification fails
    let err = load_encrypted_wasm_in_memory(&encrypted, &key).unwrap_err();
    assert_eq!(err, CryptoError::InvalidWasmHeader);
}

#[test]
fn test_verify_wasm_header_function() {
    assert!(verify_wasm_header(b"\0asm\x01\x00\x00\x00").is_ok());
    assert!(verify_wasm_header(b"\0asm").is_ok());
    assert_eq!(
        verify_wasm_header(b"\0as").unwrap_err(),
        CryptoError::InvalidWasmHeader
    );
    assert_eq!(
        verify_wasm_header(b"wasm").unwrap_err(),
        CryptoError::InvalidWasmHeader
    );
}

#[test]
fn test_parse_key_hex_and_raw() {
    // 64 hex characters
    let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let parsed_hex = parse_key(hex_key).expect("parse 64 hex string");
    assert_eq!(parsed_hex[0], 0x01);
    assert_eq!(parsed_hex[1], 0x23);
    assert_eq!(parsed_hex[15], 0xef);

    // 32 raw bytes
    let raw_key = "12345678901234567890123456789012";
    let parsed_raw = parse_key(raw_key).expect("parse 32-byte ascii key");
    assert_eq!(parsed_raw.as_slice(), raw_key.as_bytes());

    // Invalid length
    let short_key = "abcd1234";
    let err = parse_key(short_key).unwrap_err();
    assert_eq!(
        err,
        CryptoError::InvalidKeyLength {
            expected: 32,
            actual: 8
        }
    );

    // Invalid hex characters in 64-char string
    let invalid_hex = "zzzz456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let err_hex = parse_key(invalid_hex).unwrap_err();
    assert!(matches!(err_hex, CryptoError::HexDecodeError(_)));
}

#[test]
fn test_generate_key() {
    let k1 = generate_key();
    let k2 = generate_key();
    assert_eq!(k1.len(), 32);
    assert_eq!(k2.len(), 32);
    assert_ne!(k1.as_slice(), k2.as_slice());
}

#[test]
fn test_e2e_file_load_and_in_memory_decryption() {
    let temp_dir = std::env::temp_dir().join("wednes_crypto_test");
    fs::create_dir_all(&temp_dir).unwrap();
    let wenc_path = temp_dir.join("test_module.wenc");

    let key = generate_key();
    let valid_wasm = b"\0asm\x01\x00\x00\x00\x02\x08test_wasm_bytes";

    let encrypted = EncryptedArtifact::encrypt(valid_wasm, &key).unwrap();
    fs::write(&wenc_path, &encrypted).unwrap();

    // Hook loads from file into RAM, decrypts, and verifies \0asm
    let decrypted = load_encrypted_wasm_file_in_memory(&wenc_path, &key).unwrap();
    assert_eq!(decrypted.as_slice(), valid_wasm);

    // Cleanup
    let _ = fs::remove_file(&wenc_path);
    let _ = fs::remove_dir(&temp_dir);
}
