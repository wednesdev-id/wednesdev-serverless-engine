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
}

#[derive(Subcommand, Debug)]
enum Commands {
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
    },
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
            let listener = tokio::net::TcpListener::bind(&args.listen).await?;
            info!("wednesd HTTP gateway listening on {}", args.listen);
            axum::serve(listener, app).await?;
            Ok(())
        }
    }
}
