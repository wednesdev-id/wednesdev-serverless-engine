use wednes_core::{
    manifest::{Manifest, RuntimeConfig},
    Request,
};
use wednes_runtime::RuntimeBackend;
use wednes_runtime_wasmtime::WasmtimeBackend;

fn req() -> Request {
    Request {
        method: "GET".into(),
        path: "/".into(),
        headers: vec![],
        body: vec![],
    }
}

fn wasm_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target")
}

#[tokio::test]
async fn configured_limits_and_cached_execution() {
    let root = wasm_root();
    let mut b = WasmtimeBackend::init().unwrap();
    let mut m = Manifest {
        name: "hello".into(),
        abi: "wednes:function@0.1.0".into(),
        artifact: "hello.wasm".into(),
        runtime: Some(RuntimeConfig {
            memory_mb: Some(32),
            timeout_ms: Some(100),
            max_concurrency: None,
        }),
        capabilities: None,
    };
    b.load_manifest(&m, &root.join("hello.wasm")).unwrap();
    // Repeated invocations from cache
    for _ in 0..3 {
        assert_eq!(b.execute("hello", req()).await.unwrap().status, 200);
    }

    // Low memory limit should make the function fail
    m.runtime.as_mut().unwrap().memory_mb = Some(1);
    b.load_manifest(&m, &root.join("hello.wasm")).unwrap();
    assert!(b.execute("hello", req()).await.is_err());

    // Short timeout should trap infinite loop
    m.runtime.as_mut().unwrap().memory_mb = Some(32);
    b.load_manifest(&m, &root.join("infinite_loop.wasm"))
        .unwrap();
    let now = std::time::Instant::now();
    assert!(b.execute("hello", req()).await.is_err());
    assert!(
        now.elapsed().as_millis() < 500,
        "timeout too slow: {}ms",
        now.elapsed().as_millis()
    );
}

#[test]
fn empty_component_fails_at_precompile() {
    let mut b = WasmtimeBackend::init().unwrap();
    let p = std::env::temp_dir().join(format!("empty-{}.wat", std::process::id()));
    std::fs::write(&p, "(component)").unwrap();
    // It should fail at precompile because it lacks required handle export
    assert!(b.precompile("empty", &p).is_err());
    std::fs::remove_file(p).unwrap();
}
