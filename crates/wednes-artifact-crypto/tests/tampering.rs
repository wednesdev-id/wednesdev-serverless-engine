use wednes_artifact_crypto::{CipherAlgorithm, CryptoError, EncryptedArtifact, HEADER_LEN};

#[test]
fn test_invalid_key_fails_authentication() {
    let key_correct = [0x11u8; 32];
    let key_wrong = [0x22u8; 32];
    let wasm = b"\0asm\x01\x00\x00\x00auth_test";

    let encrypted = EncryptedArtifact::encrypt(wasm, &key_correct).unwrap();

    // Decrypting with wrong key must fail with AuthenticationFailed
    let err = EncryptedArtifact::decrypt(&encrypted, &key_wrong).unwrap_err();
    assert_eq!(err, CryptoError::AuthenticationFailed);
}

#[test]
fn test_tampered_ciphertext_fails_authentication() {
    let key = [0x33u8; 32];
    let wasm = b"\0asm\x01\x00\x00\x00tamper_test";

    let mut encrypted = EncryptedArtifact::encrypt(wasm, &key).unwrap();

    // Ciphertext starts at HEADER_LEN (offset 33). Flip bit in first byte of ciphertext.
    let ciphertext_offset = HEADER_LEN;
    encrypted[ciphertext_offset] ^= 0x01;

    let err = EncryptedArtifact::decrypt(&encrypted, &key).unwrap_err();
    assert_eq!(err, CryptoError::AuthenticationFailed);
}

#[test]
fn test_tampered_tag_fails_authentication() {
    let key = [0x44u8; 32];
    let wasm = b"\0asm\x01\x00\x00\x00tag_test";

    let mut encrypted = EncryptedArtifact::encrypt(wasm, &key).unwrap();

    // Tag is located at offsets 17..33. Flip bit in first byte of tag.
    let tag_offset = 4 + 1 + 12; // 17
    encrypted[tag_offset] ^= 0x80;

    let err = EncryptedArtifact::decrypt(&encrypted, &key).unwrap_err();
    assert_eq!(err, CryptoError::AuthenticationFailed);

    // Also flip bit in last byte of tag
    let mut encrypted2 = EncryptedArtifact::encrypt(wasm, &key).unwrap();
    encrypted2[tag_offset + 15] ^= 0x01;
    let err2 = EncryptedArtifact::decrypt(&encrypted2, &key).unwrap_err();
    assert_eq!(err2, CryptoError::AuthenticationFailed);
}

#[test]
fn test_tampered_nonce_fails_authentication() {
    let key = [0x55u8; 32];
    let wasm = b"\0asm\x01\x00\x00\x00nonce_test";

    let mut encrypted = EncryptedArtifact::encrypt(wasm, &key).unwrap();

    // Nonce is located at offsets 5..17. Flip bit in first byte of nonce.
    let nonce_offset = 4 + 1; // 5
    encrypted[nonce_offset] ^= 0x04;

    let err = EncryptedArtifact::decrypt(&encrypted, &key).unwrap_err();
    assert_eq!(err, CryptoError::AuthenticationFailed);
}

#[test]
fn test_truncated_ciphertext_fails_authentication() {
    let key = [0x66u8; 32];
    let wasm = b"\0asm\x01\x00\x00\x00truncation_test";

    let encrypted = EncryptedArtifact::encrypt(wasm, &key).unwrap();

    // Truncate last byte of ciphertext
    let truncated = &encrypted[..encrypted.len() - 1];

    let err = EncryptedArtifact::decrypt(truncated, &key).unwrap_err();
    assert_eq!(err, CryptoError::AuthenticationFailed);
}

#[test]
fn test_chacha_invalid_key_and_tampering() {
    let key_correct = [0x77u8; 32];
    let key_wrong = [0x88u8; 32];
    let wasm = b"\0asm\x01\x00\x00\x00chacha_tamper";

    let mut encrypted = EncryptedArtifact::encrypt_with_cipher(
        wasm,
        &key_correct,
        CipherAlgorithm::ChaCha20Poly1305,
    )
    .unwrap();

    // Wrong key
    let err = EncryptedArtifact::decrypt(&encrypted, &key_wrong).unwrap_err();
    assert_eq!(err, CryptoError::AuthenticationFailed);

    // Tampered ciphertext
    encrypted[HEADER_LEN] ^= 0x02;
    let err2 = EncryptedArtifact::decrypt(&encrypted, &key_correct).unwrap_err();
    assert_eq!(err2, CryptoError::AuthenticationFailed);
}
