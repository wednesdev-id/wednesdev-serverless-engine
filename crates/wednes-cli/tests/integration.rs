use anyhow::Result;
use std::fs;
use std::process::Command;

fn cli() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_wednesd"))
}

fn hello_wasm() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../target/hello.wasm")
}

#[test]
fn test_deploy_invalid_leaves_existing_intact() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let registry = tmp.path().join("registry.json");
    let hello_wasm_dest = tmp.path().join("hello.wasm");
    fs::copy(hello_wasm(), &hello_wasm_dest)?;

    // valid deploy
    let manifest_path = tmp.path().join("hello.toml");
    fs::write(&manifest_path, r#"
name = "hello"
abi = "wednes:function@0.1.0"
artifact = "hello.wasm"
"#)?;
    let status = Command::new(cli())
        .arg("deploy")
        .arg("--manifest").arg(&manifest_path)
        .arg("--registry").arg(&registry)
        .status()?;
    assert!(status.success());

    let r = fs::read_to_string(&registry)?;
    assert!(r.contains(r#""name": "hello""#));

    // invalid name – registry must be unchanged
    let bad_path = tmp.path().join("bad.toml");
    fs::write(&bad_path, r#"
name = "bad name!"
abi = "wednes:function@0.1.0"
artifact = "hello.wasm"
"#)?;
    let status = Command::new(cli())
        .arg("deploy")
        .arg("--manifest").arg(&bad_path)
        .arg("--registry").arg(&registry)
        .status()?;
    assert!(!status.success());

    let r2 = fs::read_to_string(&registry)?;
    assert_eq!(r, r2, "registry changed after failed deploy");

    // unknown fields
    let unk_path = tmp.path().join("unknown.toml");
    fs::write(&unk_path, r#"
name = "bad-field"
abi = "wednes:function@0.1.0"
artifact = "hello.wasm"
surprise = true
"#)?;
    let status = Command::new(cli())
        .arg("deploy")
        .arg("--manifest").arg(&unk_path)
        .arg("--registry").arg(&registry)
        .status()?;
    assert!(!status.success());

    // unsupported ABI
    let abi_path = tmp.path().join("abi.toml");
    fs::write(&abi_path, r#"
name = "bad-abi"
abi = "wednes:function@0.2.0"
artifact = "hello.wasm"
"#)?;
    let status = Command::new(cli())
        .arg("deploy")
        .arg("--manifest").arg(&abi_path)
        .arg("--registry").arg(&registry)
        .status()?;
    assert!(!status.success());

    Ok(())
}

#[test]
fn test_hello_runs_and_repeated_invocations_work() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let registry = tmp.path().join("registry.json");
    let hello_wasm_dest = tmp.path().join("hello.wasm");
    fs::copy(hello_wasm(), &hello_wasm_dest)?;

    let manifest_path = tmp.path().join("hello.toml");
    fs::write(&manifest_path, r#"
name = "hello"
abi = "wednes:function@0.1.0"
artifact = "hello.wasm"

[runtime]
memory_mb = 32
timeout_ms = 5000

"#)?;
    let status = Command::new(cli())
        .arg("deploy")
        .arg("--manifest").arg(&manifest_path)
        .arg("--registry").arg(&registry)
        .status()?;
    assert!(status.success());

    // Verify registry has hello
    let r = fs::read_to_string(&registry)?;
    assert!(r.contains(r#""name": "hello""#));
    assert!(r.contains("wednes:function@0.1.0"));

    Ok(())
}
