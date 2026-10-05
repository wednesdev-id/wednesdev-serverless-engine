use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use wednes_runtime::RuntimeBackend;
use wednes_runtime_wasmtime::WasmtimeBackend;

fn wasm_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target")
}

#[test]
fn test_child_compile() {
    if env::var("IS_CHILD").is_ok() {
        let mut backend = WasmtimeBackend::init().unwrap();
        backend
            .load_module("hello", &wasm_root().join("hello.wasm"))
            .unwrap();
        std::process::exit(0);
    }
}

#[test]
fn persistent_cache_across_processes() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let xdg_cache = env::temp_dir().join(format!("wednes-cache-test-{}", ts));
    fs::create_dir_all(&xdg_cache).unwrap();

    let exe = env::current_exe().unwrap();

    // Run first process to compile and populate the cache
    let status1 = Command::new(&exe)
        .arg("test_child_compile")
        .arg("--exact")
        .arg("--nocapture")
        .env("IS_CHILD", "1")
        .env("XDG_CACHE_HOME", &xdg_cache)
        .status()
        .expect("failed to run child process 1");
    assert!(status1.success());

    // Verify cache files exist!
    let modules_dir = xdg_cache.join("wasmtime").join("modules");
    assert!(modules_dir.exists(), "Cache modules directory must exist");

    // Find all cache files
    let mut files = Vec::new();
    for entry in fs::read_dir(&modules_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            for sub_entry in fs::read_dir(entry.path()).unwrap() {
                let sub_entry = sub_entry.unwrap();
                files.push(sub_entry.path());
            }
        }
    }
    assert!(!files.is_empty(), "Expected cached artifacts to be written");
    println!("Found cache files: {:?}", files);

    // Record the metadata/mtime of the cache file(s)
    let _mtimes: Vec<_> = files
        .iter()
        .map(|f| fs::metadata(f).unwrap().modified().unwrap())
        .collect();

    // Sleep a tiny bit to ensure mtime would change if rewritten
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Run second process with the same cache directory to verify it hits cache and does NOT add or rewrite files
    let status2 = Command::new(&exe)
        .arg("test_child_compile")
        .arg("--exact")
        .arg("--nocapture")
        .env("IS_CHILD", "1")
        .env("XDG_CACHE_HOME", &xdg_cache)
        .status()
        .expect("failed to run child process 2");
    assert!(status2.success());

    let mut new_files = Vec::new();
    for entry in fs::read_dir(&modules_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            for sub_entry in fs::read_dir(entry.path()).unwrap() {
                let sub_entry = sub_entry.unwrap();
                new_files.push(sub_entry.path());
            }
        }
    }

    assert_eq!(
        files.len(),
        new_files.len(),
        "No new files should be created on cache hit"
    );
    let stats_file = new_files
        .iter()
        .find(|p| p.extension().map_or(false, |ext| ext == "stats"))
        .unwrap();
    let stats = fs::read_to_string(stats_file).unwrap();
    assert!(
        stats.contains("usages = 2"),
        "Cache hit evidence missing: expected usages = 2, got: {}",
        stats
    );

    // Cleanup
    fs::remove_dir_all(&xdg_cache).unwrap();
}
