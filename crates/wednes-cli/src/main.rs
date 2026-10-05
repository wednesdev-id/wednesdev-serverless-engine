use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use wednes_core::{manifest::Manifest, registry::FileRegistry};
use wednes_gateway::{create_router, FnConfig, Scheduler};
use wednes_runtime::RuntimeBackend;
use wednes_runtime_wasmtime::WasmtimeBackend;

#[derive(Parser, Debug)]
#[command(author, version, about = "Wednes Engine Function Daemon")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    // Backward compat fallback
    #[arg(short, long, default_value = "127.0.0.1:8080", global = true)]
    listen: String,
    #[arg(short, long, global = true)]
    wasm: Option<PathBuf>,
    #[arg(short, long, default_value = "hello", global = true)]
    id: String,

    #[arg(long, default_value_t = 100, global = true)]
    global_concurrency: usize,
    #[arg(long, default_value_t = 1200, global = true)]
    memory_budget_mb: usize,
    #[arg(long, default_value_t = 16, global = true)]
    default_concurrency: usize,
    #[arg(long, global = true)]
    log_json: bool,
    #[arg(long, global = true)]
    http3: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init {
        name: String,
    },
    Dev,
    Deploy {
        #[arg(short, long)]
        manifest: PathBuf,
        #[arg(short, long)]
        registry: PathBuf,
    },
    Run {
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        listen: String,
        #[arg(short, long)]
        registry: PathBuf,
        #[arg(long, default_value_t = 100)]
        global_concurrency: usize,
        #[arg(long, default_value_t = 1200)]
        memory_budget_mb: usize,
        #[arg(long, default_value_t = 16)]
        default_concurrency: usize,
        #[arg(long)]
        http3: bool,
    },
}

fn setup_http3(listen: &str) -> Result<()> {
    info!("Generating self-signed TLS certificate for HTTP/3 QUIC on {}", listen);
    // ponytail: HTTP/3 requires TLS 1.3 over QUIC. axum/hyper ecosystem requires custom quinn/h3 integration for full HTTP/3 loop.
    // skipped: QUIC listener bind & HTTP/3 multiplexing, add when h3 crate stabilizes with hyper 1.0/axum 0.7.
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let _cert_der = cert.cert.der();
    info!("Self-signed cert generated. TLS barrier: native Axum 0.7 lacks QUIC/HTTP3 natively, custom h3 loop required.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.log_json {
        tracing_subscriber::fmt().json().init();
    } else {
        tracing_subscriber::fmt::init();
    }

    match args.command {
        Some(Commands::Init { name }) => {
            let project_path = std::env::current_dir()?.join(&name);
            fs::create_dir_all(&project_path)?;

            let cargo_toml = format!(
                r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wednes-sdk = {{ version = "*" }}
"#
            );
            fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

            let manifest = format!(
                r#"[manifest]
name = "{name}"
abi = "wednes:function@0.1.0"
artifact = "target/wasm32-wasip1/release/{name}.wasm"

[runtime]
memory_mb = 128
max_concurrency = 10
"#
            );
            fs::write(project_path.join("wednes.toml"), manifest)?;

            let src_dir = project_path.join("src");
            fs::create_dir_all(&src_dir)?;
            let lib_rs = r#"use wednes_sdk::{export, bindings::Guest, Request, Response};

struct Component;

impl Guest for Component {
    fn handle(_req: Request) -> Response {
        Response::json(200, "{\"message\": \"hello from wednes\"}")
    }
}

export!(Component);
"#;
            fs::write(src_dir.join("lib.rs"), lib_rs)?;
            info!("Initialized {} successfully", name);
            Ok(())
        }
        Some(Commands::Dev) => {
            // Find project name from Cargo.toml or wednes.toml
            let cargo_toml_str = fs::read_to_string("Cargo.toml").unwrap_or_default();
            let name = cargo_toml_str
                .lines()
                .find(|l| l.starts_with("name = "))
                .and_then(|l| l.split('=').nth(1))
                .unwrap_or(r#""dev""#)
                .trim()
                .trim_matches('"');

            let wasm_name = name.replace('-', "_");
            let wasm_path = format!("target/wasm32-wasip1/release/{}.wasm", wasm_name);

            use notify::{Event, RecursiveMode, Watcher};
            use std::process::Command;
            use std::sync::mpsc::channel;

            let (tx, rx) = channel();
            let mut watcher = notify::recommended_watcher(tx)?;
            watcher.watch(std::path::Path::new("src"), RecursiveMode::Recursive)?;

            let mut server: Option<std::process::Child> = None;

            let mut rebuild = || {
                info!("Compiling...");
                let status = Command::new("cargo")
                    .args(["build", "--target", "wasm32-wasip1", "--release"])
                    .status();
                match status {
                    Ok(s) if s.success() => {
                        if let Some(mut child) = server.take() {
                            let _ = child.kill();
                            let _ = child.wait();
                        }
                        if let Ok(exe) = std::env::current_exe() {
                            info!("Starting server...");
                            server = Command::new(exe)
                                .args(["--id", name, "--wasm", &wasm_path])
                                .spawn()
                                .ok();
                        }
                    }
                    _ => {
                        info!("Build failed");
                    }
                }
            };

            rebuild();

            for res in rx {
                match res {
                    Ok(Event { kind, .. }) => {
                        if kind.is_modify() || kind.is_create() || kind.is_remove() {
                            rebuild();
                        }
                    }
                    Err(e) => info!("watch error: {:?}", e),
                }
            }
            Ok(())
        }
        Some(Commands::Deploy { manifest, registry }) => {
            let m_bytes = fs::read(&manifest)?;
            let manifest_str = String::from_utf8(m_bytes)?;
            let m: Manifest = toml::from_str(&manifest_str)?;

            // Validate manifest limits etc
            m.validate()?;

            // Precompile the component and cache it
            info!("Precompiling artifact: {}", m.artifact);
            let mut backend = WasmtimeBackend::init()?;
            let artifact_path = manifest
                .parent()
                .unwrap_or(std::path::Path::new(""))
                .join(&m.artifact);
            if !artifact_path.exists() {
                bail!("Artifact not found: {:?}", artifact_path);
            }
            backend.precompile(&m.name, &artifact_path)?;

            // Register
            let mut reg = FileRegistry::open(&registry)?;
            reg.register(m.clone())?;
            info!("Deployed function '{}' to registry {:?}", m.name, registry);
            Ok(())
        }
        Some(Commands::Run {
            listen,
            registry,
            global_concurrency,
            memory_budget_mb,
            default_concurrency,
            http3,
        }) => {
            let reg = FileRegistry::open(&registry)?;
            let mut backend = WasmtimeBackend::init()?;
            let mut fn_configs = HashMap::new();
            for (id, manifest) in reg.data.functions.iter() {
                // Here we rely on Wasmtime's cache since we call load_module with the artifact path
                let artifact_path = registry
                    .parent()
                    .unwrap_or(std::path::Path::new(""))
                    .join(&manifest.artifact);
                backend.load_manifest(manifest, &artifact_path)?;
                let conc = manifest
                    .runtime
                    .as_ref()
                    .and_then(|r| r.max_concurrency)
                    .unwrap_or(default_concurrency as u32) as usize;
                let mem = manifest
                    .runtime
                    .as_ref()
                    .and_then(|r| r.memory_mb)
                    .unwrap_or(32) as usize;
                fn_configs.insert(
                    id.clone(),
                    FnConfig {
                        concurrency: conc,
                        memory_mb: mem,
                    },
                );
                info!("Loaded function '{}'", id);
            }
            let scheduler = Arc::new(Scheduler::new(
                global_concurrency,
                memory_budget_mb,
                fn_configs,
                default_concurrency,
            ));
            let runtime: Arc<dyn RuntimeBackend> = Arc::new(backend);
            let app = create_router(runtime, scheduler);
            if http3 || args.http3 {
                setup_http3(&listen)?;
            }
            let listener = tokio::net::TcpListener::bind(&listen).await?;
            info!("wednesd HTTP gateway listening on {}", listen);
            axum::serve(listener, app).await?;
            Ok(())
        }
        None => {
            // Backward compat
            info!("Starting wednesd on {} (legacy mode)", args.listen);
            let mut backend = WasmtimeBackend::init()?;
            if let Some(wasm_path) = args.wasm {
                info!("Loading function '{}' from {:?}", args.id, wasm_path);
                backend.load_module(&args.id, &wasm_path)?;
            }
            let scheduler = Arc::new(Scheduler::new(
                args.global_concurrency,
                args.memory_budget_mb,
                HashMap::new(),
                args.default_concurrency,
            ));
            let runtime: Arc<dyn RuntimeBackend> = Arc::new(backend);
            let app = create_router(runtime, scheduler);
            if args.http3 {
                setup_http3(&args.listen)?;
            }
            let listener = tokio::net::TcpListener::bind(&args.listen).await?;
            info!("wednesd HTTP gateway listening on {}", args.listen);
            axum::serve(listener, app).await?;
            Ok(())
        }
    }
}
