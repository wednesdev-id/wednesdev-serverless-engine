use wednes_artifact_crypto::{
    CipherAlgorithm, EncryptedArtifact, CURRENT_VERSION, HEADER_LEN, MAGIC, VERSION_AES_256_GCM,
    VERSION_CHACHA20_POLY1305,
};

#[test]
fn test_roundtrip_aes_256_gcm() {
    let key = [0x5au8; 32];
    let wasm_payload = b"\0asm\x01\x00\x00\x00\x01\x04\x01\x60\x00\x00";

    let encrypted =
        EncryptedArtifact::encrypt(wasm_payload, &key).expect("encryption should succeed");
    assert!(encrypted.len() >= HEADER_LEN + wasm_payload.len());
    assert_eq!(&encrypted[0..4], &MAGIC);
    assert_eq!(encrypted[4], VERSION_AES_256_GCM);

    let decrypted =
        EncryptedArtifact::decrypt(&encrypted, &key).expect("decryption should succeed");
    assert_eq!(decrypted, wasm_payload);
}

#[test]
fn test_roundtrip_chacha20_poly1305() {
    let key = [0x7bu8; 32];
    let wasm_payload = b"\0asm\x01\x00\x00\x00\x01\x04\x01\x60\x00\x00";

    let encrypted = EncryptedArtifact::encrypt_with_cipher(
        wasm_payload,
        &key,
        CipherAlgorithm::ChaCha20Poly1305,
    )
    .expect("ChaCha encryption should succeed");

    assert_eq!(&encrypted[0..4], &MAGIC);
    assert_eq!(encrypted[4], VERSION_CHACHA20_POLY1305);

    let decrypted =
        EncryptedArtifact::decrypt(&encrypted, &key).expect("decryption should succeed");
    assert_eq!(decrypted, wasm_payload);
}

#[test]
fn test_roundtrip_empty_payload() {
    let key = [0x33u8; 32];
    let empty_payload: &[u8] = &[];

    let encrypted = EncryptedArtifact::encrypt(empty_payload, &key).expect("encrypt empty");
    assert_eq!(encrypted.len(), HEADER_LEN);

    let decrypted = EncryptedArtifact::decrypt(&encrypted, &key).expect("decrypt empty");
    assert!(decrypted.is_empty());
}

#[test]
fn test_roundtrip_large_payload() {
    let key = [0x99u8; 32];
    // 512 KB realistic mock wasm module
    let mut large_wasm = Vec::with_capacity(512 * 1024);
    large_wasm.extend_from_slice(b"\0asm\x01\x00\x00\x00");
    for i in 0..(512 * 1024 - 8) {
        large_wasm.push((i % 251) as u8);
    }

    let encrypted = EncryptedArtifact::encrypt(&large_wasm, &key).expect("large encrypt");
    assert_eq!(encrypted.len(), HEADER_LEN + large_wasm.len());

    let decrypted = EncryptedArtifact::decrypt(&encrypted, &key).expect("large decrypt");
    assert_eq!(decrypted.len(), large_wasm.len());
    assert_eq!(decrypted, large_wasm);
}

#[test]
fn test_distinct_nonces_per_encryption() {
    let key = [0xa5u8; 32];
    let payload = b"\0asm\x01\x00\x00\x00hello_same_payload";

    let enc1 = EncryptedArtifact::encrypt(payload, &key).unwrap();
    let enc2 = EncryptedArtifact::encrypt(payload, &key).unwrap();

    // Ciphertext and Nonces MUST differ even with identical key and plaintext
    assert_ne!(enc1, enc2);
    let nonce1 = &enc1[5..17];
    let nonce2 = &enc2[5..17];
    assert_ne!(nonce1, nonce2);

    // Both decrypt to identical plaintext
    assert_eq!(EncryptedArtifact::decrypt(&enc1, &key).unwrap(), payload);
    assert_eq!(EncryptedArtifact::decrypt(&enc2, &key).unwrap(), payload);
}

#[test]
fn test_envelope_parse_and_to_bytes() {
    let key = [0x12u8; 32];
    let payload = b"\0asm\x01\x00\x00\x00payload_check";

    let enc_bytes = EncryptedArtifact::encrypt(payload, &key).unwrap();
    let artifact = EncryptedArtifact::parse(&enc_bytes).unwrap();

    assert_eq!(artifact.version, CURRENT_VERSION);
    assert_eq!(artifact.nonce.len(), 12);
    assert_eq!(artifact.tag.len(), 16);
    assert_eq!(artifact.ciphertext.len(), payload.len());

    let re_serialized = artifact.to_bytes();
    assert_eq!(re_serialized, enc_bytes);
}
