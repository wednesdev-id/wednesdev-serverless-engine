use wednes_artifact_crypto::{CryptoError, EncryptedArtifact, CURRENT_VERSION, HEADER_LEN};

#[test]
fn test_payload_too_short() {
    let key = [0x44u8; 32];

    // 0 bytes
    let err = EncryptedArtifact::decrypt(&[], &key).unwrap_err();
    assert_eq!(
        err,
        CryptoError::PayloadTooShort {
            expected: HEADER_LEN,
            actual: 0
        }
    );

    // 4 bytes
    let err = EncryptedArtifact::decrypt(b"WENC", &key).unwrap_err();
    assert_eq!(
        err,
        CryptoError::PayloadTooShort {
            expected: HEADER_LEN,
            actual: 4
        }
    );

    // 32 bytes (1 byte short of HEADER_LEN = 33)
    let buffer = vec![0u8; 32];
    let err = EncryptedArtifact::decrypt(&buffer, &key).unwrap_err();
    assert_eq!(
        err,
        CryptoError::PayloadTooShort {
            expected: HEADER_LEN,
            actual: 32
        }
    );
}

#[test]
fn test_invalid_magic_bytes() {
    let key = [0x55u8; 32];
    let mut payload = EncryptedArtifact::encrypt(b"\0asm\x01\x00\x00\x00", &key).unwrap();

    // Corrupt magic header from 'WENC' to 'WASH'
    payload[0] = b'W';
    payload[1] = b'A';
    payload[2] = b'S';
    payload[3] = b'H';

    let err = EncryptedArtifact::decrypt(&payload, &key).unwrap_err();
    assert_eq!(err, CryptoError::InvalidMagic(*b"WASH"));
}

#[test]
fn test_unsupported_version_mismatch() {
    let key = [0x66u8; 32];
    let mut payload = EncryptedArtifact::encrypt(b"\0asm\x01\x00\x00\x00", &key).unwrap();

    // Version is at offset 4. Corrupt to 0x99.
    payload[4] = 0x99;

    let err = EncryptedArtifact::decrypt(&payload, &key).unwrap_err();
    assert_eq!(
        err,
        CryptoError::UnsupportedVersion {
            expected: CURRENT_VERSION,
            actual: 0x99
        }
    );

    // Also test version 0x00
    payload[4] = 0x00;
    let err2 = EncryptedArtifact::decrypt(&payload, &key).unwrap_err();
    assert_eq!(
        err2,
        CryptoError::UnsupportedVersion {
            expected: CURRENT_VERSION,
            actual: 0x00
        }
    );
}

#[test]
fn test_wire_format_layout() {
    let key = [0x77u8; 32];
    let wasm = b"\0asm\x01\x00\x00\x00demo";
    let encrypted = EncryptedArtifact::encrypt(wasm, &key).unwrap();

    // Format: [Magic: 4 bytes] [Version: 1 byte] [Nonce: 12 bytes] [Tag: 16 bytes] [Ciphertext: N bytes]
    assert_eq!(&encrypted[0..4], b"WENC");
    assert_eq!(encrypted[4], 0x01);
    assert_eq!(encrypted.len(), 4 + 1 + 12 + 16 + wasm.len());
}
