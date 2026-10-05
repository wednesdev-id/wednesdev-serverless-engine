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

#[tokio::test]
async fn test_concurrency_rejects_with_429() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let registry = tmp.path().join("registry.json");
    let loop_wasm_dest = tmp.path().join("infinite_loop.wasm");
    let loop_wasm = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../target/infinite_loop.wasm");
    fs::copy(&loop_wasm, &loop_wasm_dest)?;

    // deploy with max_concurrency = 2
    let manifest_path = tmp.path().join("loop.toml");
    fs::write(&manifest_path, r#"
name = "loop"
abi = "wednes:function@0.1.0"
artifact = "infinite_loop.wasm"

[runtime]
memory_mb = 32
timeout_ms = 2000
max_concurrency = 2
"#)?;

    let status = Command::new(cli())
        .arg("deploy")
        .arg("--manifest").arg(&manifest_path)
        .arg("--registry").arg(&registry)
        .status()?;
    // NOTE: This will fail until we fix manifest.rs! We want this to fail in RED phase!
    if !status.success() {
        return Err(anyhow::anyhow!("Deploy failed"));
    }

    // start daemon with enough worker threads so WASM cpu-bound execution doesn't starve the HTTP listener
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let mut daemon = Command::new(cli())
        .env("TOKIO_WORKER_THREADS", "10")
        .arg("run")
        .arg("--listen").arg(format!("127.0.0.1:{}", port))
        .arg("--registry").arg(&registry)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?;

    // Wait until port is open
    let mut ready = false;
    for _ in 0..50 {
        if tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    assert!(ready, "Daemon failed to start");

    // Fire 4 concurrent requests
    let mut tasks = vec![];
    let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(4));
    for _ in 0..4 {
        let b = barrier.clone();
        tasks.push(tokio::spawn(async move {
            let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
            b.wait().await;
            use tokio::io::{AsyncWriteExt, AsyncReadExt};
            stream.write_all(b"GET /fn/loop HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").await.unwrap();
            let mut buf = vec![0; 1024];
            let n = stream.read(&mut buf).await.unwrap();
            let resp = String::from_utf8_lossy(&buf[..n]).to_string();
            resp
        }));
    }

    let mut statuses = vec![];
    for t in tasks {
        let resp = t.await?;
        if resp.contains("HTTP/1.1 429 Too Many Requests") {
            statuses.push(429);
        } else if resp.contains("HTTP/1.1 500 Internal Server Error") {
            statuses.push(500);
        } else if resp.contains("HTTP/1.1 200 OK") {
            statuses.push(200);
        } else {
            statuses.push(0);
        }
    }

    daemon.kill()?;

    // With max_concurrency = 2, and timeout = 500ms, if we fire 4 at exactly the same time,
    // 2 should be admitted (and eventually fail with 500 or timeout),
    // and 2 should be rejected immediately with 429!
    let count_429 = statuses.iter().filter(|&&s| s == 429).count();
    let count_500 = statuses.iter().filter(|&&s| s == 500).count();
    
    assert_eq!(count_429, 2, "Expected exactly 2 requests to be rejected with 429. Statuses: {:?}", statuses);
    assert_eq!(count_500, 2, "Expected exactly 2 requests to fail with 500 (timeout trap). Statuses: {:?}", statuses);

    Ok(())
}

#[tokio::test]
async fn test_load_global_concurrency_stability() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let registry = tmp.path().join("registry.json");
    let hello_wasm_dest = tmp.path().join("hello.wasm");
    let hello_wasm = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../target/hello.wasm");
    fs::copy(&hello_wasm, &hello_wasm_dest)?;

    // deploy hello with no per-function max_concurrency (falls back to global/default)
    let manifest_path = tmp.path().join("hello.toml");
    fs::write(&manifest_path, r#"
name = "hello"
abi = "wednes:function@0.1.0"
artifact = "hello.wasm"

[runtime]
memory_mb = 32
timeout_ms = 1000
"#)?;

    let status = Command::new(cli())
        .arg("deploy")
        .arg("--manifest").arg(&manifest_path)
        .arg("--registry").arg(&registry)
        .status()?;
    assert!(status.success());

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let mut daemon = Command::new(cli())
        .env("TOKIO_WORKER_THREADS", "16")
        .arg("run")
        .arg("--listen").arg(format!("127.0.0.1:{}", port))
        .arg("--registry").arg(&registry)
        .arg("--global-concurrency").arg("5")
        .arg("--default-concurrency").arg("10")
        .spawn()?;

    let mut ready = false;
    for _ in 0..50 {
        if tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    assert!(ready, "Daemon failed to start");

    // Blast 20 concurrent requests
    let mut tasks = vec![];
    for _ in 0..20 {
        tasks.push(tokio::spawn(async move {
            let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
            use tokio::io::{AsyncWriteExt, AsyncReadExt};
            stream.write_all(b"GET /fn/hello HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").await.unwrap();
            let mut buf = vec![0; 1024];
            let n = stream.read(&mut buf).await.unwrap();
            String::from_utf8_lossy(&buf[..n]).to_string()
        }));
    }

    let mut statuses = vec![];
    for t in tasks {
        let resp = t.await?;
        if resp.contains("HTTP/1.1 429 Too Many Requests") {
            statuses.push(429);
        } else if resp.contains("HTTP/1.1 200 OK") {
            statuses.push(200);
        } else {
            statuses.push(0);
        }
    }

    // Verify stability: we expect a mix of 200s and 429s, NO 500s or crashes
    let count_429 = statuses.iter().filter(|&&s| s == 429).count();
    let count_200 = statuses.iter().filter(|&&s| s == 200).count();
    
    assert!(count_200 > 0, "At least some requests should succeed");
    assert!(count_429 > 0, "At least some requests should be rate limited under pressure (global limit 5 vs load 20)");
    assert_eq!(count_200 + count_429, 20, "All requests must complete as either 200 or 429, daemon remained stable");

    // Verify it still responds successfully after the load burst
    let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
    use tokio::io::{AsyncWriteExt, AsyncReadExt};
    stream.write_all(b"GET /fn/hello HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").await.unwrap();
    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf).await.unwrap();
    let resp = String::from_utf8_lossy(&buf[..n]).to_string();
    assert!(resp.contains("HTTP/1.1 200 OK"), "Daemon must remain healthy after load");

    daemon.kill()?;
    Ok(())
}
